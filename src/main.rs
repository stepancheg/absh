use std::convert::TryInto;
use std::fmt::Write as _;
use std::io::Write;
use std::time::Instant;

use structopt::StructOpt;

use absh::ansi;
use absh::plot_halves_u64;
use absh::plot_u64;
use absh::sh::spawn_sh;
use absh::student::t_table;
use absh::student::TWO_SIDED_95;
use absh::test_name::TestName;
use absh::Duration;
use absh::MemUsage;
use absh::Number;
use absh::Numbers;
use absh::PlotHighlight;
use absh::RunLog;
use absh::Stats;
use rand::prelude::SliceRandom;
use wait4::Wait4;

struct Test {
    name: TestName,
    warmup: String,
    run: String,
    durations: Numbers<Duration>,
    mem_usages: Numbers<MemUsage>,
}

impl Test {
    fn plot_highlights(&self) -> PlotHighlight {
        PlotHighlight {
            non_zero: format!("{}", self.name.color().to_owned()),
            zero: format!("{}", ansi::WHITE_BG),
            reset: ansi::RESET.to_owned(),
        }
    }

    fn plot_halves_highlights(&self) -> PlotHighlight {
        PlotHighlight {
            non_zero: format!("{}", self.name.color().to_owned()),
            zero: "".to_owned(),
            reset: ansi::RESET.to_owned(),
        }
    }
}

#[derive(StructOpt, Debug)]
struct Opts {
    #[structopt(short = "a", help = "A variant shell script")]
    a: String,
    #[structopt(short = "b", help = "B variant shell script")]
    b: Option<String>,
    #[structopt(short = "c", help = "C variant shell script")]
    c: Option<String>,
    #[structopt(short = "d", help = "D variant shell script")]
    d: Option<String>,
    #[structopt(short = "e", help = "E variant shell script")]
    e: Option<String>,
    #[structopt(short = "A", long = "a-warmup", help = "A variant warmup shell script")]
    aw: Option<String>,
    #[structopt(short = "B", long = "b-warmup", help = "B variant warmup shell script")]
    bw: Option<String>,
    #[structopt(short = "C", long = "c-warmup", help = "C variant warmup shell script")]
    cw: Option<String>,
    #[structopt(short = "D", long = "d-warmup", help = "D variant warmup shell script")]
    dw: Option<String>,
    #[structopt(short = "E", long = "e-warmup", help = "E variant warmup shell script")]
    ew: Option<String>,
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
    #[structopt(short = "m", long = "mem", help = "Also measure max resident set size")]
    mem: bool,
}

fn run_test(log: &mut absh::RunLog, test: &mut Test) {
    writeln!(log.both_log_and_stderr()).unwrap();
    writeln!(
        log.both_log_and_stderr(),
        "running test: {}",
        test.name.name_colored()
    )
    .unwrap();
    let warmup_lines = test.warmup.lines().collect::<Vec<_>>();
    if !warmup_lines.is_empty() {
        writeln!(log.both_log_and_stderr(), "running warmup script:").unwrap();
        for line in &warmup_lines {
            writeln!(log.both_log_and_stderr(), "    {}", line).unwrap();
        }
    }

    let mut process = spawn_sh(&test.warmup);
    let status = process.wait4().unwrap();
    if !status.status.success() {
        writeln!(
            log.both_log_and_stderr(),
            "warmup failed: {}",
            status.status
        )
        .unwrap();
        return;
    }

    writeln!(log.both_log_and_stderr(), "running script:").unwrap();
    let lines = test.run.lines().collect::<Vec<_>>();
    for line in &lines {
        writeln!(log.both_log_and_stderr(), "    {}", line).unwrap();
    }

    let start = Instant::now();

    let mut process = spawn_sh(&test.run);
    let status = process.wait4().unwrap();
    if !status.status.success() {
        writeln!(
            log.both_log_and_stderr(),
            "script failed: {}",
            status.status
        )
        .unwrap();
        return;
    }

    let duration = Duration::from_nanos(start.elapsed().as_nanos().try_into().unwrap());
    assert!(status.rusage.maxrss != 0, "maxrss not available");
    let max_rss = MemUsage::from_bytes(status.rusage.maxrss);

    writeln!(
        log.both_log_and_stderr(),
        "{} finished in {:3} s, max rss {} MiB",
        test.name.name_colored(),
        duration,
        max_rss.mib(),
    )
    .unwrap();

    test.durations.push(duration);
    test.mem_usages.push(max_rss);
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

fn make_distr_plots<N: Number>(
    tests: &[Test],
    width: usize,
    numbers: impl Fn(&Test) -> &Numbers<N>,
) -> Vec<String> {
    let min = tests.iter().map(|t| numbers(t).min()).min().unwrap();
    let max = tests.iter().map(|t| numbers(t).max()).max().unwrap();

    let distr_halves: Vec<_> = tests
        .iter()
        .map(|t| (t, numbers(t).distr(width * 2, min.clone(), max.clone())))
        .collect();

    let distr: Vec<_> = tests
        .iter()
        .map(|t| (t, numbers(t).distr(width, min.clone(), max.clone())))
        .collect();

    let max_height_halves = distr_halves
        .iter()
        .map(|(_, d)| d.max())
        .max()
        .unwrap()
        .clone();
    let max_height = distr.iter().map(|(_, d)| d.max()).max().unwrap().clone();

    let distr_plots = distr
        .iter()
        .map(|(t, d)| plot_u64(&d.counts, max_height, &t.plot_highlights()))
        .collect();

    let distr_halves_plots = distr_halves
        .iter()
        .map(|(t, d)| plot_halves_u64(&d.counts, max_height_halves, &t.plot_halves_highlights()))
        .collect();

    if max_height_halves <= 2 {
        distr_halves_plots
    } else {
        distr_plots
    }
}

fn print_stats<N: Number>(
    tests: &[Test],
    log: &mut RunLog,
    name: &str,
    numbers: impl Fn(&Test) -> &Numbers<N>,
) {
    let test_color = |t: &Test| t.name.color();

    let stats: Vec<_> = tests.iter().map(|t| numbers(t).stats()).collect();
    let durations: Vec<_> = tests.iter().map(|t| numbers(t)).collect();

    let stats_str: Vec<_> = Stats::display_stats(&stats);

    let stats_width = stats_str.iter().map(|s| s.len()).max().unwrap();

    let distr_plots = make_distr_plots(&tests, stats_width - 8, numbers);

    writeln!(log.both_log_and_stderr(), "").unwrap();
    writeln!(log.both_log_and_stderr(), "{}:", name).unwrap();
    for index in 0..tests.len() {
        let test = &tests[index];
        let stats = &stats_str[index];
        writeln!(
            log.both_log_and_stderr(),
            "{color}{name}{reset}: {stats}",
            name = test.name,
            color = test_color(test),
            reset = ansi::RESET,
        )
        .unwrap();
    }
    for index in 0..tests.len() {
        let test = &tests[index];
        let plot = &distr_plots[index];
        writeln!(
            log.stderr_only(),
            "{color}{name}{reset}: distr=[{plot}]",
            name = test.name,
            color = test_color(test),
            reset = ansi::RESET,
        )
        .unwrap();
    }

    if tests.len() >= 2 {
        for b_index in 1..tests.len() {
            let degrees_of_freedom =
                u64::min(stats[0].count as u64 - 1, stats[b_index].count as u64 - 1);
            let t_star = t_table(degrees_of_freedom, TWO_SIDED_95);

            // Half of a confidence interval
            let conf_h = t_star
                * f64::sqrt(
                    stats[0].sigma_sq() / (stats[0].count - 1) as f64
                        + stats[b_index].sigma_sq() / (stats[b_index].count - 1) as f64,
                );

            // Quarter of a confidence interval
            let conf_q = conf_h / 2.0;

            let b_a_min =
                (stats[b_index].mean.as_f64() - conf_q) / (stats[0].mean.as_f64() + conf_q);
            let b_a_max =
                (stats[b_index].mean.as_f64() + conf_q) / (stats[0].mean.as_f64() - conf_q);

            writeln!(
                log.both_log_and_stderr(),
                "{b_name}/{a_name}: {b_a:.3} {b_a_min:.3}..{b_a_max:.3} (95% conf)",
                b_name = tests[b_index].name,
                a_name = tests[0].name,
                b_a = stats[b_index].mean.as_f64() / stats[0].mean.as_f64(),
                b_a_min = b_a_min,
                b_a_max = b_a_max,
            )
            .unwrap();
        }
    }

    log.write_raw(&durations).unwrap();
}

fn main() {
    let opts: Opts = Opts::from_args();

    let mut log = RunLog::open();

    let mut tests = Vec::new();
    tests.push(Test {
        name: TestName::A,
        warmup: opts.aw.clone().unwrap_or(String::new()),
        run: opts.a.clone(),
        durations: Numbers::default(),
        mem_usages: Numbers::default(),
    });

    fn parse_opt_test(
        tests: &mut Vec<Test>,
        name: TestName,
        run: &Option<String>,
        warmup: &Option<String>,
    ) {
        if let Some(run) = run.clone() {
            tests.push(Test {
                name,
                warmup: warmup.clone().unwrap_or(String::new()),
                run,
                durations: Numbers::default(),
                mem_usages: Numbers::default(),
            });
        }
    }
    parse_opt_test(&mut tests, TestName::B, &opts.b, &opts.bw);
    parse_opt_test(&mut tests, TestName::C, &opts.c, &opts.cw);
    parse_opt_test(&mut tests, TestName::D, &opts.d, &opts.dw);
    parse_opt_test(&mut tests, TestName::E, &opts.e, &opts.ew);

    eprintln!("Writing absh data to {}/", log.name().display());
    if let Some(last) = log.last() {
        eprintln!("Log symlink is {}", last.display());
    }

    writeln!(log.log_only(), "random_order: {}", opts.random_order).unwrap();
    for t in &mut tests {
        writeln!(log.log_only(), "{}.run: {}", t.name, t.run).unwrap();
        if !t.warmup.is_empty() {
            writeln!(log.log_only(), "{}.warmup: {}", t.name, t.warmup).unwrap();
        }
    }

    if opts.ignore_first {
        run_pair(&mut log, &opts, &mut tests);

        for test in &mut tests {
            test.durations.clear();
            test.mem_usages.clear();
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
            yellow = ansi::YELLOW,
            reset = ansi::RESET,
        )
        .unwrap();
        writeln!(
            log.both_log_and_stderr(),
            "{yellow}Results might be skewed.{reset}",
            yellow = ansi::YELLOW,
            reset = ansi::RESET,
        )
        .unwrap();
        writeln!(
            log.both_log_and_stderr(),
            "{yellow}Use `-i` command line flag to ignore the first iteration.{reset}",
            yellow = ansi::YELLOW,
            reset = ansi::RESET,
        )
        .unwrap();
    }

    loop {
        run_pair(&mut log, &opts, &mut tests);

        let min_duration_len = tests.iter_mut().map(|t| t.durations.len()).min().unwrap();
        if Some(min_duration_len) == opts.iterations.map(|n| n as usize) {
            break;
        }

        if min_duration_len < 2 {
            continue;
        }

        print_stats(&tests, &mut log, "Time (in seconds)", |t| &t.durations);
        if opts.mem {
            print_stats(&tests, &mut log, "Max RSS (in megabytes)", |t| {
                &t.mem_usages
            });
        }
    }
}
