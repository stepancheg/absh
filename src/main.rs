use std::convert::TryInto;
use std::fmt::Write as _;
use std::time::Instant;

use absh::ansi;
use absh::ansi::AnsiColor;
use absh::duration::Duration;
use absh::experiment::Experiment;
use absh::experiment_map::ExperimentMap;
use absh::experiment_name::ExperimentName;
use absh::measure::key::MeasureKey;
use absh::measure::tr::AllMeasures;
use absh::measure::tr::MaxRss;
use absh::measure::tr::MeasureDyn;
use absh::measure::tr::WallTime;
use absh::mem_usage::MemUsage;
use absh::opts::AbshOpts;
use absh::run_log::RunLog;
use absh::sh::spawn_sh;
use clap::Parser;
use rand::prelude::SliceRandom;
use wait4::Wait4;

fn run_test(log: &mut RunLog, test: &mut Experiment, opts: &AbshOpts) -> anyhow::Result<()> {
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

    let duration = Duration::from_nanos(start.elapsed().as_nanos().try_into()?);

    if !status.status.success() {
        writeln!(
            log.both_log_and_stderr(),
            "script failed: {}",
            status.status
        )?;
        return Ok(());
    }
    if let Some(max_time_s) = opts.max_time {
        if duration.seconds_f64() > max_time_s as f64 {
            writeln!(
                log.both_log_and_stderr(),
                "script took too long: {} s",
                duration.seconds_f64() as u64
            )?;
            return Ok(());
        }
    }

    if status.rusage.maxrss == 0 {
        return Err(anyhow::anyhow!("maxrss not available"));
    }
    let max_rss = MemUsage::from_bytes(status.rusage.maxrss);

    writeln!(
        log.both_log_and_stderr(),
        "{} finished in {:3} s, max rss {} MiB",
        test.name.name_colored(),
        duration,
        max_rss.mib(),
    )?;

    test.measures[MeasureKey::WallTime].push(duration.nanos());
    test.measures[MeasureKey::MaxRss].push(max_rss.bytes());
    Ok(())
}

fn run_pair(
    log: &mut RunLog,
    opts: &AbshOpts,
    tests: &mut ExperimentMap<Experiment>,
) -> anyhow::Result<()> {
    let mut indices: Vec<ExperimentName> = tests.keys().collect();
    if opts.random_order {
        indices.shuffle(&mut rand::thread_rng());
    }
    for &index in &indices {
        run_test(log, tests.get_mut(index).unwrap(), opts)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opts: AbshOpts = AbshOpts::parse();

    let mut experiments = opts.experiments();

    let mut log = RunLog::open();

    eprintln!("Writing absh data to {}/", log.name().display());
    if let Some(last) = log.last() {
        eprintln!("Log symlink is {}", last.display());
    }

    log.write_args()?;

    writeln!(log.log_only(), "random_order: {}", opts.random_order)?;
    for (n, t) in experiments.iter() {
        writeln!(log.log_only(), "{}.run: {}", n, t.run)?;
        if !t.warmup.is_empty() {
            writeln!(log.log_only(), "{}.warmup: {}", n, t.warmup)?;
        }
    }

    if opts.ignore_first {
        run_pair(&mut log, &opts, &mut experiments)?;

        for (_n, test) in experiments.iter_mut() {
            for numbers in test.measures.values_mut() {
                numbers.clear();
            }
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
            yellow = AnsiColor::Yellow.fg(),
            reset = ansi::RESET,
        )?;
        writeln!(
            log.both_log_and_stderr(),
            "{yellow}Results might be skewed.{reset}",
            yellow = AnsiColor::Yellow.fg(),
            reset = ansi::RESET,
        )?;
        writeln!(
            log.both_log_and_stderr(),
            "{yellow}Use `-i` command line flag to ignore the first iteration.{reset}",
            yellow = AnsiColor::Yellow.fg(),
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
        run_pair(&mut log, &opts, &mut experiments)?;

        let min_count = experiments.values_mut().map(|t| t.runs()).min().unwrap();
        if Some(min_count) == opts.iterations.map(|n| n as usize) {
            break;
        }

        if min_count < 2 {
            continue;
        }

        writeln!(log.both_log_and_stderr(), "")?;

        let graph_full = measures.render_stats(&experiments, true)?;
        let graph_short = measures.render_stats(&experiments, false)?;

        write!(log.stderr_only(), "{}", graph_full)?;
        write!(log.log_only(), "{}", graph_short,)?;

        log.write_graph(&graph_full)?;

        measures.write_raw(&experiments, &mut log)?;
    }

    Ok(())
}
