use std::convert::TryInto;
use std::fmt::Write as _;
use std::io::Write;
use std::time::Instant;

use structopt::StructOpt;

use absh::ansi;
use absh::ansi::RESET;
use absh::plot_halves_u64;
use absh::plot_u64;
use absh::sh::spawn_sh;
use absh::t_table;
use absh::Duration;
use absh::Numbers;
use absh::PlotHighlight;
use absh::Stats;
use absh::TWO_SIDED_95;
use rand::prelude::SliceRandom;

struct Test {
    name: &'static str,
    warmup: String,
    run: String,
    color_if_tty: &'static str,
    durations: Numbers<Duration>,
}

impl Test {
    fn color(&self, is_tty: bool) -> &'static str {
        match is_tty {
            true => self.color_if_tty,
            false => "",
        }
    }

    fn plot_highlights(&self, is_tty: bool) -> PlotHighlight {
        match is_tty {
            true => PlotHighlight {
                non_zero: format!("{}", self.color_if_tty.to_owned()),
                zero: format!("{}", ansi::WHITE_BG),
                reset: RESET.to_owned(),
            },
            false => PlotHighlight::no(),
        }
    }

    fn plot_halves_highlights(&self, is_tty: bool) -> PlotHighlight {
        match is_tty {
            true => PlotHighlight {
                non_zero: format!("{}", self.color_if_tty.to_owned()),
                zero: "".to_owned(),
                reset: RESET.to_owned(),
            },
            false => PlotHighlight::no(),
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
    #[structopt(short = "A", long = "a-warmup", help = "A variant warmup shell script")]
    aw: Option<String>,
    #[structopt(short = "B", long = "b-warmup", help = "B variant warmup shell script")]
    bw: Option<String>,
    #[structopt(short = "C", long = "c-warmup", help = "C variant warmup shell script")]
    cw: Option<String>,
    #[structopt(short = "D", long = "d-warmup", help = "D variant warmup shell script")]
    dw: Option<String>,
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

fn run_pair(log: &mut absh::RunLog, opts: &Opts, tests: &mut [Test]) {
    let mut indices: Vec<usize> = (0..tests.len()).collect();
    if opts.random_order {
        indices.shuffle(&mut rand::thread_rng());
    }
    for &index in &indices {
        run_test(log, &mut tests[index]);
    }
}

fn make_distr_plots(tests: &[Test], width: usize, is_tty: bool) -> Vec<String> {
    let min = tests.iter().map(|t| t.durations.min()).min().unwrap();
    let max = tests.iter().map(|t| t.durations.max()).max().unwrap();

    let distr_halves: Vec<_> = tests
        .iter()
        .map(|t| (t, t.durations.distr(width * 2, min, max)))
        .collect();

    let distr: Vec<_> = tests
        .iter()
        .map(|t| (t, t.durations.distr(width, min, max)))
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
        .map(|(t, d)| plot_u64(&d.counts, max_height, &t.plot_highlights(is_tty)))
        .collect();

    let distr_halves_plots = distr_halves
        .iter()
        .map(|(t, d)| {
            plot_halves_u64(
                &d.counts,
                max_height_halves,
                &t.plot_halves_highlights(is_tty),
            )
        })
        .collect();

    if max_height_halves <= 2 {
        distr_halves_plots
    } else {
        distr_plots
    }
}

fn main() {
    let opts: Opts = Opts::from_args();

    let mut log = absh::RunLog::open();

    let mut tests = Vec::new();
    tests.push(Test {
        name: "A",
        warmup: opts.aw.clone().unwrap_or(String::new()),
        run: opts.a.clone(),
        color_if_tty: ansi::RED,
        durations: Numbers::default(),
    });
    if let Some(b) = opts.b.clone() {
        tests.push(Test {
            name: "B",
            warmup: opts.bw.clone().unwrap_or(String::new()),
            run: b,
            color_if_tty: ansi::GREEN,
            durations: Numbers::default(),
        });
    }
    if let Some(c) = opts.c.clone() {
        tests.push(Test {
            name: "C",
            warmup: opts.cw.clone().unwrap_or(String::new()),
            run: c,
            color_if_tty: ansi::BLUE,
            durations: Numbers::default(),
        });
    }
    if let Some(d) = opts.d.clone() {
        tests.push(Test {
            name: "D",
            warmup: opts.dw.clone().unwrap_or(String::new()),
            run: d,
            color_if_tty: ansi::MAGENTA,
            durations: Numbers::default(),
        });
    }

    let is_tty = !cfg!(windows) && atty::is(atty::Stream::Stderr);
    let (yellow, reset) = match is_tty {
        true => (ansi::YELLOW, ansi::RESET),
        false => ("", ""),
    };

    let test_color = |t: &Test| t.color(is_tty);

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
        run_pair(&mut log, &opts, &mut tests);

        let min_duration_len = tests.iter_mut().map(|t| t.durations.len()).min().unwrap();
        if Some(min_duration_len) == opts.iterations.map(|n| n as usize) {
            break;
        }

        if min_duration_len < 2 {
            continue;
        }

        let stats: Vec<_> = tests.iter_mut().map(|t| t.durations.stats()).collect();
        let durations: Vec<_> = tests.iter().map(|t| &t.durations).collect();

        let stats_str: Vec<_> = stats
            .iter()
            .map(|s: &Stats<Duration>| s.to_string())
            .collect();

        let stats_width = stats_str.iter().map(|s| s.len()).max().unwrap();

        let distr_plots = make_distr_plots(&tests, stats_width - 8, is_tty);

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
                "{color}{name}{reset}: distr=[{plot}]",
                name = test.name,
                color = test_color(test),
                reset = reset,
                plot = distr_plot,
            );
        }

        if tests.len() >= 2 {
            for b_index in 1..tests.len() {
                let degrees_of_freedom =
                    u64::min(stats[0].count as u64 - 1, stats[b_index].count as u64 - 1);
                let t_star = t_table(degrees_of_freedom, TWO_SIDED_95);

                // Half of a confidence interval
                let conf_h = t_star
                    * f64::sqrt(
                        stats[0].var_millis_sq() / (stats[0].count - 1) as f64
                            + stats[b_index].var_millis_sq() / (stats[b_index].count - 1) as f64,
                    );

                // Quarter of a confidence interval
                let conf_q = conf_h / 2.0;

                let b_a_min = (stats[b_index].mean.millis_f64() - conf_q)
                    / (stats[0].mean.millis_f64() + conf_q);
                let b_a_max = (stats[b_index].mean.millis_f64() + conf_q)
                    / (stats[0].mean.millis_f64() - conf_q);

                writeln!(
                    log.both_log_and_stderr(),
                    "{b_name}/{a_name}: {b_a:.3} {b_a_min:.3}..{b_a_max:.3} (95% conf)",
                    b_name = tests[b_index].name,
                    a_name = tests[0].name,
                    b_a = stats[b_index].mean / stats[0].mean,
                    b_a_min = b_a_min,
                    b_a_max = b_a_max,
                )
                .unwrap();
            }
        }

        log.write_raw(&durations).unwrap();
    }
}
