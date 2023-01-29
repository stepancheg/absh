use std::convert::TryInto;
use std::fmt::Write as _;
use std::time::Instant;

use absh::ansi;
use absh::measure::MaxRss;
use absh::measure::MeasureDyn;
use absh::measure::WallTime;
use absh::sh::spawn_sh;
use absh::student::t_table;
use absh::student::TWO_SIDED_95;
use absh::test::Test;
use absh::test_name::TestName;
use absh::Duration;
use absh::MemUsage;
use absh::Number;
use absh::Numbers;
use absh::RunLog;
use clap::Parser;
use rand::prelude::SliceRandom;
use wait4::Wait4;

#[derive(clap::Parser, Debug)]
#[command(about = "A/B testing for shell scripts")]
struct Opts {
    #[clap(short, help = "A variant shell script")]
    a: String,
    #[clap(short, help = "B variant shell script")]
    b: Option<String>,
    #[clap(short, help = "C variant shell script")]
    c: Option<String>,
    #[clap(short, help = "D variant shell script")]
    d: Option<String>,
    #[clap(short, help = "E variant shell script")]
    e: Option<String>,
    #[clap(short = 'A', long = "a-warmup", help = "A variant warmup shell script")]
    aw: Option<String>,
    #[clap(short = 'B', long = "b-warmup", help = "B variant warmup shell script")]
    bw: Option<String>,
    #[clap(short = 'C', long = "c-warmup", help = "C variant warmup shell script")]
    cw: Option<String>,
    #[clap(short = 'D', long = "d-warmup", help = "D variant warmup shell script")]
    dw: Option<String>,
    #[clap(short = 'E', long = "e-warmup", help = "E variant warmup shell script")]
    ew: Option<String>,
    #[clap(short = 'r', help = "Randomise test execution order")]
    random_order: bool,
    #[clap(short = 'i', help = "Ignore the results of the first iteration")]
    ignore_first: bool,
    #[clap(
        short = 'n',
        help = "Stop after n successful iterations (run forever if not specified)"
    )]
    iterations: Option<u32>,
    #[clap(short = 'm', long, help = "Also measure max resident set size")]
    mem: bool,
}

fn run_test(log: &mut absh::RunLog, test: &mut Test) -> anyhow::Result<()> {
    writeln!(log.both_log_and_stderr())?;
    writeln!(
        log.both_log_and_stderr(),
        "running test: {}",
        test.name.name_colored()
    )?;
    let warmup_lines = test.warmup.lines().collect::<Vec<_>>();
    if !warmup_lines.is_empty() {
        writeln!(log.both_log_and_stderr(), "running warmup script:")?;
        for line in &warmup_lines {
            writeln!(log.both_log_and_stderr(), "    {}", line)?;
        }
    }

    let mut process = spawn_sh(&test.warmup);
    let status = process.wait4()?;
    if !status.status.success() {
        writeln!(
            log.both_log_and_stderr(),
            "warmup failed: {}",
            status.status
        )?;
        return Ok(());
    }

    writeln!(log.both_log_and_stderr(), "running script:")?;
    let lines = test.run.lines().collect::<Vec<_>>();
    for line in &lines {
        writeln!(log.both_log_and_stderr(), "    {}", line)?;
    }

    let start = Instant::now();

    let mut process = spawn_sh(&test.run);
    let status = process.wait4()?;
    if !status.status.success() {
        writeln!(
            log.both_log_and_stderr(),
            "script failed: {}",
            status.status
        )?;
        return Ok(());
    }

    let duration = Duration::from_nanos(start.elapsed().as_nanos().try_into()?);
    assert!(status.rusage.maxrss != 0, "maxrss not available");
    let max_rss = MemUsage::from_bytes(status.rusage.maxrss);

    writeln!(
        log.both_log_and_stderr(),
        "{} finished in {:3} s, max rss {} MiB",
        test.name.name_colored(),
        duration,
        max_rss.mib(),
    )?;

    test.durations.push(duration);
    test.mem_usages.push(max_rss);
    Ok(())
}

fn run_pair(log: &mut absh::RunLog, opts: &Opts, tests: &mut [Test]) -> anyhow::Result<()> {
    let mut indices: Vec<usize> = (0..tests.len()).collect();
    if opts.random_order {
        indices.shuffle(&mut rand::thread_rng());
    }
    for &index in &indices {
        run_test(log, &mut tests[index])?;
    }
    Ok(())
}

fn render_stats<N: Number>(
    tests: &[Test],
    include_distr: bool,
    measure: &dyn MeasureDyn,
    numbers: impl Fn(&Test) -> &Numbers<N>,
) -> anyhow::Result<String> {
    let mut r = String::new();

    let stats: Vec<_> = tests.iter().map(|t| numbers(t).stats()).collect();

    let stats_str: Vec<_> = measure.display_stats(tests);

    let stats_width = stats_str.iter().map(|s| s.len()).max().unwrap();

    let distr_plots = measure.make_distr_plots(&tests, stats_width - 8)?;

    writeln!(r, "{}:", measure.name())?;
    for index in 0..tests.len() {
        let test = &tests[index];
        let stats = &stats_str[index];
        writeln!(
            r,
            "{color}{name}{reset}: {stats}",
            name = test.name,
            color = test.name.color(),
            reset = ansi::RESET,
        )?;
    }
    for index in 0..tests.len() {
        let test = &tests[index];
        let plot = &distr_plots[index];
        if include_distr {
            writeln!(
                r,
                "{color}{name}{reset}: distr=[{plot}]",
                name = test.name,
                color = test.name.color(),
                reset = ansi::RESET,
            )?;
        }
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
                r,
                "{b_name}/{a_name}: {b_a:.3} {b_a_min:.3}..{b_a_max:.3} (95% conf)",
                b_name = tests[b_index].name,
                a_name = tests[0].name,
                b_a = stats[b_index].mean.as_f64() / stats[0].mean.as_f64(),
                b_a_min = b_a_min,
                b_a_max = b_a_max,
            )?;
        }
    }

    Ok(r)
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();

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

    log.write_args()?;

    writeln!(log.log_only(), "random_order: {}", opts.random_order)?;
    for t in &mut tests {
        writeln!(log.log_only(), "{}.run: {}", t.name, t.run)?;
        if !t.warmup.is_empty() {
            writeln!(log.log_only(), "{}.warmup: {}", t.name, t.warmup)?;
        }
    }

    if opts.ignore_first {
        run_pair(&mut log, &opts, &mut tests)?;

        for test in &mut tests {
            test.durations.clear();
            test.mem_usages.clear();
        }

        writeln!(log.both_log_and_stderr(), "")?;
        writeln!(
            log.both_log_and_stderr(),
            "Ignoring first run pair results."
        )?;
        writeln!(log.both_log_and_stderr(), "Now collecting the results.")?;
        writeln!(
            log.both_log_and_stderr(),
            "Statistics will be printed after the second successful iteration."
        )?;
    } else {
        writeln!(log.both_log_and_stderr(), "")?;
        writeln!(
            log.both_log_and_stderr(),
            "{yellow}First run pair results will be used in statistics.{reset}",
            yellow = ansi::YELLOW,
            reset = ansi::RESET,
        )?;
        writeln!(
            log.both_log_and_stderr(),
            "{yellow}Results might be skewed.{reset}",
            yellow = ansi::YELLOW,
            reset = ansi::RESET,
        )?;
        writeln!(
            log.both_log_and_stderr(),
            "{yellow}Use `-i` command line flag to ignore the first iteration.{reset}",
            yellow = ansi::YELLOW,
            reset = ansi::RESET,
        )?;
    }

    loop {
        run_pair(&mut log, &opts, &mut tests)?;

        let min_duration_len = tests.iter_mut().map(|t| t.durations.len()).min().unwrap();
        if Some(min_duration_len) == opts.iterations.map(|n| n as usize) {
            break;
        }

        if min_duration_len < 2 {
            continue;
        }

        writeln!(log.both_log_and_stderr(), "")?;

        let mut graph_full = render_stats(&tests, true, &WallTime, |t| &t.durations)?;
        let mut graph_short = render_stats(&tests, false, &WallTime, |t| &t.durations)?;

        if opts.mem {
            graph_full.push_str("\n");
            graph_short.push_str("\n");
            graph_full.push_str(&render_stats(&tests, true, &MaxRss, |t| &t.mem_usages)?);
            graph_short.push_str(&render_stats(&tests, false, &MaxRss, |t| &t.mem_usages)?);
        }

        write!(log.stderr_only(), "{}", graph_full)?;
        write!(log.log_only(), "{}", graph_short,)?;

        log.write_graph(&graph_full)?;

        let durations: Vec<_> = tests.iter().map(|t| &t.durations).collect();
        log.write_raw(&durations)?;
    }

    Ok(())
}
