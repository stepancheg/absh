use std::convert::TryInto;
use std::fmt;
use std::fmt::Write as _;
use std::io::Write;
use std::time::Instant;

use structopt::StructOpt;

use absh::ansi;
use absh::plot;
use absh::plot_halves;
use absh::sh::spawn_sh;
use absh::t_table;
use absh::Duration;
use absh::Durations;
use absh::TWO_SIDED_95;
use rand::prelude::SliceRandom;

struct Test {
    name: &'static str,
    warmup: String,
    run: String,
    color_if_tty: &'static str,
    durations: Durations,
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

fn run_test(log: &mut absh::RunLog, test: &mut Test) {
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

    test.durations.push(duration);
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

fn run_pair(log: &mut absh::RunLog, opts: &Opts, tests: &mut [Test]) {
    let mut indices: Vec<usize> = (0..tests.len()).collect();
    if opts.random_order {
        indices.shuffle(&mut rand::thread_rng());
    }
    for &index in &indices {
        run_test(log, &mut tests[index]);
    }
}

fn make_two_distr(durations: &[&Durations], width: usize) -> Vec<String> {
    let min = durations.iter().map(|d| d.min()).min().unwrap();
    let max = durations.iter().map(|d| d.max()).max().unwrap();

    let distr_halves: Vec<_> = durations
        .iter()
        .map(|d| d.distr(width * 2, min, max))
        .collect();

    let distr: Vec<_> = durations.iter().map(|d| d.distr(width, min, max)).collect();

    let max_height_halves = distr_halves
        .iter()
        .map(|h| h.iter().max().unwrap())
        .max()
        .unwrap()
        .clone();
    let max_height = distr
        .iter()
        .map(|h| h.iter().max().unwrap())
        .max()
        .unwrap()
        .clone();

    let a_distr = distr[0].iter().map(|&v| v as f64).collect::<Vec<_>>();
    let b_distr = distr[1].iter().map(|&v| v as f64).collect::<Vec<_>>();

    let a_distr_halves = distr_halves[0]
        .iter()
        .map(|&v| v as f64)
        .collect::<Vec<_>>();
    let b_distr_halves = distr_halves[1]
        .iter()
        .map(|&v| v as f64)
        .collect::<Vec<_>>();

    let a_distr_plot = plot(&a_distr, 0.0, max_height as f64);
    let b_distr_plot = plot(&b_distr, 0.0, max_height as f64);

    let a_distr_halves_plot = plot_halves(&a_distr_halves, 0.0, max_height_halves as f64);
    let b_distr_halves_plot = plot_halves(&b_distr_halves, 0.0, max_height_halves as f64);

    if max_height_halves <= 2 {
        vec![a_distr_halves_plot, b_distr_halves_plot]
    } else {
        vec![a_distr_plot, b_distr_plot]
    }
}

fn main() {
    let opts: Opts = Opts::from_args();

    let mut log = absh::RunLog::open();

    let a = Test {
        name: "A",
        warmup: opts.aw.clone().unwrap_or(String::new()),
        run: opts.a.clone(),
        color_if_tty: ansi::RED,
        durations: Durations::default(),
    };
    let b = Test {
        name: "B",
        warmup: opts.bw.clone().unwrap_or(String::new()),
        run: opts.b.clone(),
        color_if_tty: ansi::GREEN,
        durations: Durations::default(),
    };

    let mut tests = vec![a, b];

    let is_tty = !cfg!(windows) && atty::is(atty::Stream::Stderr);
    let (yellow, reset) = match is_tty {
        true => (ansi::YELLOW, ansi::RESET),
        false => ("", ""),
    };

    let test_color = match is_tty {
        true => |t: &Test| t.color_if_tty,
        false => |_: &Test| "",
    };

    eprintln!("Writing absh data to {}/", log.name().display());
    if let Some(last) = log.last() {
        eprintln!("Log symlink is {}", last.display());
    }

    writeln!(&mut log, "random_order: {}", opts.random_order).unwrap();
    for t in &mut tests {
        writeln!(&mut log, "{}.run: {}", t.name, t.run).unwrap();
        if !t.warmup.is_empty() {
            writeln!(&mut log, "{}.warmup: {}", t.name, t.warmup).unwrap();
        }
    }

    if opts.ignore_first {
        run_pair(&mut log, &opts, &mut tests);

        for test in &mut tests {
            test.durations.clear();
        }

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
        let min_duration_len = tests.iter_mut().map(|t| t.durations.len()).min().unwrap();
        if Some(min_duration_len) == opts.iterations.map(|n| n as usize) {
            break;
        }

        run_pair(&mut log, &opts, &mut tests);
        if min_duration_len < 2 {
            continue;
        }

        let stats: Vec<_> = tests.iter_mut().map(|t| stats(&mut t.durations)).collect();
        let durations: Vec<_> = tests.iter().map(|t| &t.durations).collect();

        let stats_str: Vec<_> = stats.iter().map(|s: &Stats| s.to_string()).collect();

        let stats_width = stats_str.iter().map(|s| s.len()).max().unwrap();

        let distr_plots = make_two_distr(&durations, stats_width - 8);

        writeln!(log.both_log_and_stderr(), "").unwrap();
        for index in 0..tests.len() {
            let test = &tests[index];
            let stats = &stats[index];
            writeln!(
                log.both_log_and_stderr(),
                "{color}{name}{reset}: {stats}",
                name = test.name,
                color = test_color(test),
                reset = reset,
                stats = stats,
            )
            .unwrap();
        }
        for index in 0..tests.len() {
            let test = &tests[index];
            let distr_plot = &distr_plots[index];
            eprintln!(
                "{color}{name}{reset}: distr=[{color}{plot}{reset}]",
                name = test.name,
                color = test_color(test),
                reset = reset,
                plot = distr_plot,
            );
        }

        let degrees_of_freedom = u64::min(stats[0].count as u64 - 1, stats[1].count as u64 - 1);
        let t_star = t_table(degrees_of_freedom, TWO_SIDED_95);

        // Half of a confidence interval
        let conf_h = t_star
            * f64::sqrt(
                stats[0].var_millis_sq() / (stats[0].count - 1) as f64
                    + stats[1].var_millis_sq() / (stats[1].count - 1) as f64,
            );

        // Quarter of a confidence interval
        let conf_q = conf_h / 2.0;

        let b_a_min = (stats[1].mean.millis_f64() - conf_q) / (stats[0].mean.millis_f64() + conf_q);
        let b_a_max = (stats[1].mean.millis_f64() + conf_q) / (stats[0].mean.millis_f64() - conf_q);

        writeln!(
            log.both_log_and_stderr(),
            "B/A: {:.3} {:.3}..{:.3} (95% conf)",
            stats[1].mean / stats[0].mean,
            b_a_min,
            b_a_max,
        )
        .unwrap();

        log.write_raw(&durations).unwrap();
    }
}
