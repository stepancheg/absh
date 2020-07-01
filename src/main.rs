use std::process::Child;
use std::process::Command;
use std::process::Stdio;

use std::fmt;
use std::time::Instant;
use structopt::StructOpt;

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

struct Duration {
    millis: u64,
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:03}", self.millis / 1000, self.millis % 1000)
    }
}

fn run_test(test: &Test, duration_millis: &mut Vec<Duration>) {
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
    avg: Duration,
    std: Duration,
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "N={}, r={}+-{}", self.count, self.avg, self.std)
    }
}

fn stats(durations: &[Duration]) -> Stats {
    assert!(durations.len() >= 3);
    let durations = &durations[1..];
    let sum: f64 = durations.iter().map(|d| d.millis as f64).sum();
    let avg: f64 = sum / durations.len() as f64;
    let s_2 = durations
        .iter()
        .map(|d| (d.millis as f64 - avg) * (d.millis as f64 - avg))
        .sum::<f64>()
        / ((durations.len() - 1) as f64);
    let std = f64::sqrt(s_2);
    Stats {
        count: durations.len() as u64,
        avg: Duration { millis: avg as u64 },
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

    loop {
        run_test(&b, &mut b_durations);
        run_test(&a, &mut a_durations);
        if a_durations.len() < 3 || b_durations.len() < 3 {
            continue;
        }
        let a_stats = stats(&a_durations);
        let b_stats = stats(&b_durations);
        eprintln!("A: {}", a_stats);
        eprintln!("B: {}", b_stats);
        eprintln!("B/A: {:.3}", (b_stats.avg.millis as f64) / (a_stats.avg.millis as f64));
    }
}
