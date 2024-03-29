use std::convert::TryInto;
use std::fmt::Write as _;
use std::time::Instant;

use absh::ansi;
use absh::duration::Duration;
use absh::experiment::Experiment;
use absh::experiment_map::ExperimentMap;
use absh::experiment_name::ExperimentName;
use absh::measure::key::MeasureKey;
use absh::measure::map::MeasureMap;
use absh::measure::tr::AllMeasures;
use absh::measure::tr::MaxRss;
use absh::measure::tr::MeasureDyn;
use absh::measure::tr::WallTime;
use absh::mem_usage::MemUsage;
use absh::run_log::RunLog;
use absh::sh::spawn_sh;
use clap::Parser;
use rand::prelude::SliceRandom;
use wait4::Wait4;

/// A/B testing for shell scripts.
#[derive(clap::Parser, Debug)]
struct Opts {
    /// A variant shell script.
    #[clap(short)]
    a: String,
    /// B variant shell script.
    #[clap(short)]
    b: Option<String>,
    /// C variant shell script.
    #[clap(short)]
    c: Option<String>,
    /// D variant shell script.
    #[clap(short)]
    d: Option<String>,
    /// E variant shell script.
    #[clap(short)]
    e: Option<String>,
    /// A variant warmup shell script.
    #[clap(short = 'A', long = "a-warmup")]
    aw: Option<String>,
    /// B variant warmup shell script.
    #[clap(short = 'B', long = "b-warmup")]
    bw: Option<String>,
    /// C variant warmup shell script.
    #[clap(short = 'C', long = "c-warmup")]
    cw: Option<String>,
    /// D variant warmup shell script.
    #[clap(short = 'D', long = "d-warmup")]
    dw: Option<String>,
    /// E variant warmup shell script.
    #[clap(short = 'E', long = "e-warmup")]
    ew: Option<String>,
    /// Randomise test execution order.
    #[clap(short = 'r')]
    random_order: bool,
    /// Ignore the results of the first iteration.
    #[clap(short = 'i')]
    ignore_first: bool,
    /// Stop after n successful iterations (run forever if not specified).
    #[clap(short = 'n')]
    iterations: Option<u32>,
    /// Also measure max resident set size.
    #[clap(short = 'm', long)]
    mem: bool,
    /// Test is considered failed if it takes longer than this many seconds.
    #[clap(long)]
    max_time: Option<u32>,
}

fn run_test(log: &mut RunLog, test: &mut Experiment, opts: &Opts) -> anyhow::Result<()> {
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
    opts: &Opts,
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
    let opts: Opts = Opts::parse();

    let mut log = RunLog::open();

    let mut experiments = ExperimentMap::default();
    experiments.insert(
        ExperimentName::A,
        Experiment {
            name: ExperimentName::A,
            warmup: opts.aw.clone().unwrap_or(String::new()),
            run: opts.a.clone(),
            measures: MeasureMap::new_all_default(),
        },
    );

    fn parse_opt_test(
        tests: &mut ExperimentMap<Experiment>,
        name: ExperimentName,
        run: &Option<String>,
        warmup: &Option<String>,
    ) {
        if let Some(run) = run.clone() {
            tests.insert(
                name,
                Experiment {
                    name,
                    warmup: warmup.clone().unwrap_or(String::new()),
                    run,
                    measures: MeasureMap::new_all_default(),
                },
            );
        }
    }
    parse_opt_test(&mut experiments, ExperimentName::B, &opts.b, &opts.bw);
    parse_opt_test(&mut experiments, ExperimentName::C, &opts.c, &opts.cw);
    parse_opt_test(&mut experiments, ExperimentName::D, &opts.d, &opts.dw);
    parse_opt_test(&mut experiments, ExperimentName::E, &opts.e, &opts.ew);

    eprintln!("Writing absh data to {}/", log.name().display());
    if let Some(last) = log.last() {
        eprintln!("Log symlink is {}", last.display());
    }

    log.write_args()?;

    writeln!(log.log_only(), "random_order: {}", opts.random_order)?;
    for (n, t) in experiments.iter_mut() {
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
