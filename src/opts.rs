use clap::builder::styling;
use clap::builder::Styles;

use crate::experiment::Experiment;
use crate::experiment_map::ExperimentMap;
use crate::experiment_name::ExperimentName;
use crate::measure::map::MeasureMap;

pub(crate) fn clap_styles() -> Styles {
    let heading = styling::AnsiColor::Yellow.on_default().bold();
    Styles::styled()
        .header(heading)
        .usage(heading)
        .literal(styling::AnsiColor::Green.on_default())
        .placeholder(styling::AnsiColor::Cyan.on_default())
}

/// A/B testing for shell scripts.
#[derive(clap::Parser, Debug)]
#[clap(styles = clap_styles())]
pub struct AbshOpts {
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
    pub random_order: bool,
    /// Ignore the results of the first iteration.
    #[clap(short = 'i')]
    pub ignore_first: bool,
    /// Stop after n successful iterations (run forever if not specified).
    #[clap(short = 'n')]
    pub iterations: Option<u32>,
    /// Also measure max resident set size.
    #[clap(short = 'm', long)]
    pub mem: bool,
    /// Test is considered failed if it takes longer than this many seconds.
    #[clap(long)]
    pub max_time: Option<u32>,
}

impl AbshOpts {
    pub fn experiments(&self) -> ExperimentMap<Experiment> {
        let mut experiments = ExperimentMap::default();
        experiments.insert(
            ExperimentName::A,
            Experiment {
                name: ExperimentName::A,
                warmup: self.aw.clone().unwrap_or(String::new()),
                run: self.a.clone(),
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
        parse_opt_test(&mut experiments, ExperimentName::B, &self.b, &self.bw);
        parse_opt_test(&mut experiments, ExperimentName::C, &self.c, &self.cw);
        parse_opt_test(&mut experiments, ExperimentName::D, &self.d, &self.dw);
        parse_opt_test(&mut experiments, ExperimentName::E, &self.e, &self.ew);
        experiments
    }
}
