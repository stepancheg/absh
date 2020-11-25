use std::process::Child;
use std::process::Command;
use std::process::Stdio;

use std::fmt;
use std::ops::Add;
use std::ops::Div;
use std::ops::Sub;
use std::time::Instant;
use structopt::StructOpt;

use absh::t_table;
use absh::TWO_SIDED_95;

struct Test {
    name: &'static str,
    warmup: String,
    run: String,
}

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(short = "a", long = "a")]
    a: String,
    #[structopt(short = "b", long = "b")]
    b: String,
    #[structopt(short = "A", long = "a-warmup", default_value = "")]
    aw: String,
    #[structopt(short = "B", long = "b-warmup", default_value = "")]
    bw: String,
}

fn spawn_sh(script: &str) -> Child {
    Command::new("/bin/sh")
        .args(&["-ec", &script])
        .stdin(Stdio::null())
        .spawn()
        .expect("launch /bin/sh")
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct Duration {
    millis: u64,
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration {
            millis: self.millis - rhs.millis,
        }
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Duration {
            millis: self.millis + rhs.millis,
        }
    }
}

impl Div<u64> for Duration {
    type Output = Duration;

    fn div(self, rhs: u64) -> Self::Output {
        Duration {
            millis: self.millis / rhs,
        }
    }
}

impl Div<Duration> for Duration {
    type Output = f64;

    fn div(self, rhs: Duration) -> Self::Output {
        self.millis as f64 / rhs.millis as f64
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:03}", self.millis / 1000, self.millis % 1000)
    }
}

fn run_test(test: &Test, duration_millis: &mut Vec<Duration>) {
    eprintln!();
    eprintln!("running test: {}", test.name);
    let warmup_lines = test.warmup.lines().collect::<Vec<_>>();
    if !warmup_lines.is_empty() {
        eprintln!("running warmup script:");
        for line in &warmup_lines {
            eprintln!("    {}", line);
        }
    }

    let mut process = spawn_sh(&test.warmup);
    let status = process.wait().unwrap();
    if !status.success() {
        eprintln!("warmup failed: {}", status);
        return;
    }

    eprintln!("running script:");
    let lines = test.run.lines().collect::<Vec<_>>();
    for line in &lines {
        eprintln!("    {}", line);
    }

    let start = Instant::now();

    let mut process = spawn_sh(&test.run);
    let status = process.wait().unwrap();
    if !status.success() {
        eprintln!("script failed: {}", status);
        return;
    }

    let duration = Duration {
        millis: start.elapsed().as_millis() as u64,
    };

    eprintln!("{} finished in {}", test.name, duration);

    duration_millis.push(duration);
}

struct Stats {
    count: u64,
    mean: Duration,
    med: Duration,
    min: Duration,
    max: Duration,
    std: Duration,
}

impl Stats {
    /// sigma^2
    fn var(&self) -> f64 {
        let millis = self.std.millis as f64;
        millis * millis
    }

    fn se(&self) -> Duration {
        Duration {
            millis: (self.std.millis as f64 / f64::sqrt((self.count - 1) as f64)) as u64,
        }
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = self.max;
        write!(
            f,
            "n={n} mean={mean} std={std} se={se} min={min} med={med}",
            n = self.count,
            mean = self.mean,
            std = self.std,
            se = self.se(),
            min = self.min,
            med = self.med,
        )
    }
}

fn stats(durations: &mut [Duration]) -> Stats {
    assert!(durations.len() >= 3);
    let durations = &mut durations[1..];

    // sort to obtain min/median
    durations.sort();

    let sum: f64 = durations.iter().map(|d| d.millis as f64).sum();
    let avg: f64 = sum / durations.len() as f64;
    let s_2 = durations
        .iter()
        .map(|d| (d.millis as f64 - avg) * (d.millis as f64 - avg))
        .sum::<f64>()
        / ((durations.len() - 1) as f64);
    let std = f64::sqrt(s_2);

    let med = if durations.len() % 2 == 0 {
        (durations[durations.len() / 2 - 1] + durations[durations.len() / 2]) / 2
    } else {
        durations[durations.len() / 2]
    };

    Stats {
        count: durations.len() as u64,
        mean: Duration { millis: avg as u64 },
        med,
        min: durations[0],
        max: durations.last().unwrap().clone(),
        std: Duration { millis: std as u64 },
    }
}

fn main() {
    let ops = Opts::from_args();

    let a = Test {
        name: "A",
        warmup: ops.aw,
        run: ops.a,
    };
    let b = Test {
        name: "B",
        warmup: ops.bw,
        run: ops.b,
    };

    let mut a_durations = Vec::new();
    let mut b_durations = Vec::new();

    let is_tty = !cfg!(windows) && atty::is(atty::Stream::Stderr);
    let (green, red, reset) = match is_tty {
        true => ("\x1B[32m", "\x1B[31m", "\x1B[0m"),
        false => ("", "", ""),
    };

    loop {
        run_test(&b, &mut b_durations);
        run_test(&a, &mut a_durations);
        if a_durations.len() < 3 || b_durations.len() < 3 {
            continue;
        }
        let a_stats = stats(&mut a_durations);
        let b_stats = stats(&mut b_durations);
        eprintln!();
        eprintln!("{}A{}: {}", red, reset, a_stats);
        eprintln!("{}B{}: {}", green, reset, b_stats);

        let degrees_of_freedom = u64::min(a_stats.count as u64 - 1, b_stats.count as u64 - 1);
        let t_star = t_table(degrees_of_freedom, TWO_SIDED_95);

        // Half of a confidence interval
        let conf_h = t_star
            * f64::sqrt(
                a_stats.var() / (a_stats.count - 1) as f64
                    + b_stats.var() / (b_stats.count - 1) as f64,
            );

        // Quarter of a confidence interval
        let conf_q = conf_h / 2.0;

        let b_a_min = (b_stats.mean.millis as f64 - conf_q) / (a_stats.mean.millis as f64 + conf_q);
        let b_a_max = (b_stats.mean.millis as f64 + conf_q) / (a_stats.mean.millis as f64 - conf_q);

        eprintln!(
            "B/A: {:.3} {:.3}..{:.3} (95% conf)",
            b_stats.mean / a_stats.mean,
            b_a_min,
            b_a_max
        );
    }
}
