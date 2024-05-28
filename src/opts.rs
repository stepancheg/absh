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
/// In scripts, `@ABSH_P` placeholder is replaced with
/// the current experiment name (`a`, `b`...).
#[derive(clap::Parser, Debug, Default)]
#[clap(styles = clap_styles(), verbatim_doc_comment)]
pub struct AbshOpts {
    /// A variant shell script.
    #[clap(short, value_name = "SCRIPT")]
    a: String,
    /// B variant shell script.
    #[clap(short, value_name = "SCRIPT")]
    b: Option<String>,
    /// C variant shell script.
    #[clap(short, value_name = "SCRIPT")]
    c: Option<String>,
    /// D variant shell script.
    #[clap(short, value_name = "SCRIPT")]
    d: Option<String>,
    /// Warmup script to run before each test.
    #[clap(
        short,
        long,
        conflicts_with_all = &["aw", "bw", "cw", "dw"],
        value_name = "SCRIPT",
    )]
    warmup: Option<String>,
    /// A variant warmup shell script, used unless `--warmup` is specified.
    #[clap(short = 'A', long = "a-warmup", value_name = "SCRIPT")]
    aw: Option<String>,
    /// B variant warmup shell script, used unless `--warmup` is specified.
    #[clap(short = 'B', long = "b-warmup", value_name = "SCRIPT")]
    bw: Option<String>,
    /// C variant warmup shell script, used unless `--warmup` is specified.
    #[clap(short = 'C', long = "c-warmup", value_name = "SCRIPT")]
    cw: Option<String>,
    /// D variant warmup shell script, used unless `--warmup` is specified.
    #[clap(short = 'D', long = "d-warmup", value_name = "SCRIPT")]
    dw: Option<String>,
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
    #[clap(long, value_name = "SECONDS")]
    pub max_time: Option<u32>,
}

impl AbshOpts {
    fn make_script(script: &str, experiment: ExperimentName) -> String {
        script.replace("@ABSH_P", &experiment.lower())
    }

    fn run_for_experiment(&self, experiment: ExperimentName) -> Option<String> {
        match experiment {
            ExperimentName::A => Some(Self::make_script(&self.a, ExperimentName::A)),
            ExperimentName::B => Some(Self::make_script(self.b.as_ref()?, ExperimentName::B)),
            ExperimentName::C => Some(Self::make_script(self.c.as_ref()?, ExperimentName::C)),
            ExperimentName::D => Some(Self::make_script(self.d.as_ref()?, ExperimentName::D)),
            ExperimentName::E => None,
        }
    }

    fn warmup_for_experiment(&self, experiment: ExperimentName) -> String {
        let letter_warmup: Option<&str> = match experiment {
            ExperimentName::A => self.aw.as_deref(),
            ExperimentName::B => self.bw.as_deref(),
            ExperimentName::C => self.cw.as_deref(),
            ExperimentName::D => self.dw.as_deref(),
            ExperimentName::E => None,
        };
        let warmup = letter_warmup.or(self.warmup.as_deref()).unwrap_or_default();
        Self::make_script(warmup, experiment)
    }

    fn experiment(&self, experiment: ExperimentName) -> Option<Experiment> {
        Some(Experiment {
            name: experiment,
            run: self.run_for_experiment(experiment)?,
            warmup: self.warmup_for_experiment(experiment),
            measures: MeasureMap::new_all_default(),
        })
    }

    pub fn experiments(&self) -> ExperimentMap<Experiment> {
        let mut experiments = ExperimentMap::default();
        for experiment_name in ExperimentName::all() {
            if let Some(experiment) = self.experiment(experiment_name) {
                experiments.insert(experiment_name, experiment);
            }
        }
        experiments
    }
}

#[cfg(test)]
mod tests {
    use crate::experiment_name::ExperimentName;
    use crate::opts::AbshOpts;

    #[test]
    fn test_absh_p_substituted() {
        let experiment_map = AbshOpts {
            a: "echo 1 @ABSH_P".to_owned(),
            b: Some("echo 2 @ABSH_P".to_owned()),
            aw: Some("echo 3 @ABSH_P".to_owned()),
            warmup: Some("echo 4 @ABSH_P".to_owned()),
            ..AbshOpts::default()
        }
        .experiments();
        assert_eq!(
            "echo 1 a",
            experiment_map.get(ExperimentName::A).unwrap().run
        );
        assert_eq!(
            "echo 2 b",
            experiment_map.get(ExperimentName::B).unwrap().run
        );
        assert_eq!(
            "echo 3 a",
            experiment_map.get(ExperimentName::A).unwrap().warmup
        );
        assert_eq!(
            "echo 4 b",
            experiment_map.get(ExperimentName::B).unwrap().warmup
        );
    }
}
