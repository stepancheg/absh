use std::convert::TryInto;
use std::fmt::Write as _;
use std::time::Instant;

use absh::ansi;
use absh::duration::Duration;
use absh::measure::AllMeasures;
use absh::measure::MaxRss;
use absh::measure::MeasureDyn;
use absh::measure::WallTime;
use absh::mem_usage::MemUsage;
use absh::numbers::Numbers;
use absh::run_log::RunLog;
use absh::sh::spawn_sh;
use absh::test::Test;
use absh::test_name::TestName;
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

fn run_test(log: &mut RunLog, test: &mut Test) -> anyhow::Result<()> {
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

    let mut process = spawn_sh(&test.warmup)?;
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

    let mut process = spawn_sh(&test.run)?;
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

fn run_pair(log: &mut RunLog, opts: &Opts, tests: &mut [Test]) -> anyhow::Result<()> {
    let mut indices: Vec<usize> = (0..tests.len()).collect();
    if opts.random_order {
        indices.shuffle(&mut rand::thread_rng());
    }
    for &index in &indices {
        run_test(log, &mut tests[index])?;
    }
    Ok(())
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

    let mut measures: Vec<Box<dyn MeasureDyn>> = Vec::new();
    measures.push(Box::new(WallTime));
    if opts.mem {
        measures.push(Box::new(MaxRss));
    }
    let measures = AllMeasures(measures);

    loop {
        run_pair(&mut log, &opts, &mut tests)?;

        let min_count = tests.iter().map(|t| t.runs()).min().unwrap();
        if Some(min_count) == opts.iterations.map(|n| n as usize) {
            break;
        }

        if min_count < 2 {
            continue;
        }

        writeln!(log.both_log_and_stderr(), "")?;

        let graph_full = measures.render_stats(&tests, true)?;
        let graph_short = measures.render_stats(&tests, false)?;

        write!(log.stderr_only(), "{}", graph_full)?;
        write!(log.log_only(), "{}", graph_short,)?;

        log.write_graph(&graph_full)?;

        let durations: Vec<_> = tests.iter().map(|t| &t.durations).collect();
        log.write_raw(&durations)?;
    }

    Ok(())
}
