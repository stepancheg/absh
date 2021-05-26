use std::cmp;
use std::fmt;
use std::fmt::Write as _;
use std::io::Write;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::time::Instant;

use structopt::StructOpt;

use absh::ansi;
use absh::plot;
use absh::plot_halves;
use absh::t_table;
use absh::Duration;
use absh::Durations;
use absh::TWO_SIDED_95;
use std::convert::TryInto;

struct Test {
    name: &'static str,
    warmup: String,
    run: String,
}

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(short = "a", help = "A variant shell script")]
    a: String,
    #[structopt(short = "b", help = "B variant shell script")]
    b: String,
    #[structopt(short = "A", long = "a-warmup", help = "A variant warmup shell script")]
    aw: Option<String>,
    #[structopt(short = "B", long = "b-warmup", help = "B variant warmup shell script")]
    bw: Option<String>,
    #[structopt(
        short = "r",
        long = "random-order",
        help = "Randomise test execution order"
    )]
    random_order: bool,
    #[structopt(
        short = "i",
        long = "ignore-first",
        help = "Ignore the results of the first iteration"
    )]
    ignore_first: bool,
    #[structopt(
        short = "n",
        long = "iterations",
        help = "Stop after n successful iterations (run forever if not specified)"
    )]
    iterations: Option<u32>,
}

fn spawn_sh(script: &str) -> Child {
    Command::new("/bin/sh")
        .args(&["-ec", &script])
        .stdin(Stdio::null())
        .spawn()
        .expect("launch /bin/sh")
}

fn run_test(log: &mut absh::RunLog, test: &Test, durations: &mut Durations) {
    writeln!(log.both_log_and_stderr()).unwrap();
    writeln!(log.both_log_and_stderr(), "running test: {}", test.name).unwrap();
    let warmup_lines = test.warmup.lines().collect::<Vec<_>>();
    if !warmup_lines.is_empty() {
        writeln!(log.both_log_and_stderr(), "running warmup script:").unwrap();
        for line in &warmup_lines {
            writeln!(log.both_log_and_stderr(), "    {}", line).unwrap();
        }
    }

    let mut process = spawn_sh(&test.warmup);
    let status = process.wait().unwrap();
    if !status.success() {
        writeln!(log.both_log_and_stderr(), "warmup failed: {}", status).unwrap();
        return;
    }

    writeln!(log.both_log_and_stderr(), "running script:").unwrap();
    let lines = test.run.lines().collect::<Vec<_>>();
    for line in &lines {
        writeln!(log.both_log_and_stderr(), "    {}", line).unwrap();
    }

    let start = Instant::now();

    let mut process = spawn_sh(&test.run);
    let status = process.wait().unwrap();
    if !status.success() {
        writeln!(log.both_log_and_stderr(), "script failed: {}", status).unwrap();
        return;
    }

    let duration = Duration::from_nanos(start.elapsed().as_nanos().try_into().unwrap());

    writeln!(
        log.both_log_and_stderr(),
        "{} finished in {}",
        test.name,
        duration
    )
    .unwrap();

    durations.push(duration);
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
    fn var_millis_sq(&self) -> f64 {
        let millis = self.std.millis_f64();
        millis * millis
    }

    fn se(&self) -> Duration {
        Duration::from_nanos((self.std.nanos() as f64 / f64::sqrt((self.count - 1) as f64)) as u64)
    }
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = self.max;
        write!(
            f,
            "n={n} mean={mean} std={std} se={se} min={min} max={max} med={med}",
            n = self.count,
            mean = self.mean,
            std = self.std,
            se = self.se(),
            min = self.min,
            max = self.max,
            med = self.med,
        )
    }
}

fn stats(durations: &mut Durations) -> Stats {
    assert!(durations.len() >= 2);

    Stats {
        count: durations.len() as u64,
        mean: durations.mean(),
        med: durations.med(),
        min: durations.min(),
        max: durations.max(),
        std: durations.std(),
    }
}

fn run_pair(
    log: &mut absh::RunLog,
    opts: &Opts,
    a: &Test,
    b: &Test,
    mut a_durations: &mut Durations,
    mut b_durations: &mut Durations,
) {
    if !opts.random_order || rand::random() {
        run_test(log, &b, &mut b_durations);
        run_test(log, &a, &mut a_durations);
    } else {
        run_test(log, &a, &mut a_durations);
        run_test(log, &b, &mut b_durations);
    }
}

fn make_two_distr(
    a_durations: &Durations,
    b_durations: &Durations,
    width: usize,
) -> (String, String) {
    let min = cmp::min(a_durations.min(), b_durations.min());
    let max = cmp::max(a_durations.max(), b_durations.max());

    let a_distr_halves = a_durations.distr(width * 2, min, max);
    let b_distr_halves = b_durations.distr(width * 2, min, max);

    let a_distr = a_durations.distr(width, min, max);
    let b_distr = b_durations.distr(width, min, max);

    let max_height_halves = cmp::max(
        a_distr_halves.iter().max().unwrap(),
        b_distr_halves.iter().max().unwrap(),
    )
    .clone();
    let max_height = cmp::max(a_distr.iter().max().unwrap(), b_distr.iter().max().unwrap()).clone();

    let a_distr = a_distr.iter().map(|&v| v as f64).collect::<Vec<_>>();
    let b_distr = b_distr.iter().map(|&v| v as f64).collect::<Vec<_>>();

    let a_distr_halves = a_distr_halves.iter().map(|&v| v as f64).collect::<Vec<_>>();
    let b_distr_halves = b_distr_halves.iter().map(|&v| v as f64).collect::<Vec<_>>();

    let a_distr_plot = plot(&a_distr, 0.0, max_height as f64);
    let b_distr_plot = plot(&b_distr, 0.0, max_height as f64);

    let a_distr_halves_plot = plot_halves(&a_distr_halves, 0.0, max_height_halves as f64);
    let b_distr_halves_plot = plot_halves(&b_distr_halves, 0.0, max_height_halves as f64);

    if max_height_halves <= 2 {
        (a_distr_halves_plot, b_distr_halves_plot)
    } else {
        (a_distr_plot, b_distr_plot)
    }
}

fn main() {
    let opts: Opts = Opts::from_args();

    let mut log = absh::RunLog::open();

    let a = Test {
        name: "A",
        warmup: opts.aw.clone().unwrap_or(String::new()),
        run: opts.a.clone(),
    };
    let b = Test {
        name: "B",
        warmup: opts.bw.clone().unwrap_or(String::new()),
        run: opts.b.clone(),
    };

    let mut a_durations = Durations::default();
    let mut b_durations = Durations::default();

    let is_tty = !cfg!(windows) && atty::is(atty::Stream::Stderr);
    let (green, red, yellow, reset) = match is_tty {
        true => (ansi::GREEN, ansi::RED, ansi::YELLOW, ansi::RESET),
        false => ("", "", "", ""),
    };

    eprintln!("Writing absh data to {}/", log.name().display());
    if let Some(last) = log.last() {
        eprintln!("Log symlink is {}", last.display());
    }

    writeln!(&mut log, "random_order: {}", opts.random_order).unwrap();
    for t in &[&a, &b] {
        writeln!(&mut log, "{}.run: {}", t.name, t.run).unwrap();
        if !t.warmup.is_empty() {
            writeln!(&mut log, "{}.warmup: {}", t.name, t.warmup).unwrap();
        }
    }

    if opts.ignore_first {
        run_pair(
            &mut log,
            &opts,
            &a,
            &b,
            &mut Durations::default(),
            &mut Durations::default(),
        );

        writeln!(log.both_log_and_stderr(), "").unwrap();
        writeln!(
            log.both_log_and_stderr(),
            "Ignoring first run pair results."
        )
        .unwrap();
        writeln!(log.both_log_and_stderr(), "Now collecting the results.").unwrap();
        writeln!(
            log.both_log_and_stderr(),
            "Statistics will be printed after the second successful iteration."
        )
        .unwrap();
    } else {
        writeln!(log.both_log_and_stderr(), "").unwrap();
        writeln!(
            log.both_log_and_stderr(),
            "{yellow}First run pair results will be used in statistics.{reset}",
            yellow = yellow,
            reset = reset
        )
        .unwrap();
        writeln!(
            log.both_log_and_stderr(),
            "{yellow}Results might be skewed.{reset}",
            yellow = yellow,
            reset = reset
        )
        .unwrap();
        writeln!(
            log.both_log_and_stderr(),
            "{yellow}Use `-i` command line flag to ignore the first iteration.{reset}",
            yellow = yellow,
            reset = reset
        )
        .unwrap();
    }

    loop {
        if Some(cmp::min(a_durations.len(), b_durations.len()))
            == opts.iterations.map(|n| n as usize)
        {
            break;
        }

        run_pair(&mut log, &opts, &a, &b, &mut a_durations, &mut b_durations);
        if a_durations.len() < 2 || b_durations.len() < 2 {
            continue;
        }
        let a_stats = stats(&mut a_durations);
        let b_stats = stats(&mut b_durations);

        let a_stats_str = a_stats.to_string();
        let b_stats_str = b_stats.to_string();

        let stats_width = cmp::max(a_stats_str.len(), b_stats_str.len());

        let (a_distr_plot, b_distr_plot) =
            make_two_distr(&a_durations, &b_durations, stats_width - 8);

        writeln!(log.both_log_and_stderr(), "").unwrap();
        writeln!(
            log.both_log_and_stderr(),
            "{color}A{reset}: {stats}",
            color = red,
            reset = reset,
            stats = a_stats
        )
        .unwrap();
        writeln!(
            log.both_log_and_stderr(),
            "{color}B{reset}: {stats}",
            color = green,
            reset = reset,
            stats = b_stats
        )
        .unwrap();
        eprintln!(
            "{color}A{reset}: distr=[{color}{plot}{reset}]",
            color = red,
            reset = reset,
            plot = a_distr_plot
        );
        eprintln!(
            "{color}B{reset}: distr=[{color}{plot}{reset}]",
            color = green,
            reset = reset,
            plot = b_distr_plot
        );

        let degrees_of_freedom = u64::min(a_stats.count as u64 - 1, b_stats.count as u64 - 1);
        let t_star = t_table(degrees_of_freedom, TWO_SIDED_95);

        // Half of a confidence interval
        let conf_h = t_star
            * f64::sqrt(
                a_stats.var_millis_sq() / (a_stats.count - 1) as f64
                    + b_stats.var_millis_sq() / (b_stats.count - 1) as f64,
            );

        // Quarter of a confidence interval
        let conf_q = conf_h / 2.0;

        let b_a_min = (b_stats.mean.millis_f64() - conf_q) / (a_stats.mean.millis_f64() + conf_q);
        let b_a_max = (b_stats.mean.millis_f64() + conf_q) / (a_stats.mean.millis_f64() - conf_q);

        writeln!(
            log.both_log_and_stderr(),
            "B/A: {:.3} {:.3}..{:.3} (95% conf)",
            b_stats.mean / a_stats.mean,
            b_a_min,
            b_a_max,
        )
        .unwrap();

        log.write_raw(&a_durations, &b_durations).unwrap();
    }
}
