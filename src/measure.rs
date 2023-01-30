use std::fmt::Display;

use crate::distr_plot::make_distr_plots;
use crate::duration::Duration;
use crate::experiment::Experiment;
use crate::experiment_map::ExperimentMap;
use crate::math::number::Number;
use crate::math::numbers::Numbers;
use crate::math::stats::Stats;
use crate::mem_usage::MemUsage;
use crate::render_stats::render_stats;
use crate::run_log::RunLog;

pub(crate) trait Measure {
    type Number: Number;
    type NumberDisplay: Display + Copy;

    fn number_to_display(&self, number: Self::Number) -> Self::NumberDisplay;

    fn name(&self) -> &str;
    fn id(&self) -> &str;
    fn numbers(test: &Experiment) -> &Numbers<Self::Number>;
}

pub struct WallTime;

impl Measure for WallTime {
    type Number = Duration;
    type NumberDisplay = Duration;

    fn number_to_display(&self, number: Self::Number) -> Self::NumberDisplay {
        number
    }

    fn name(&self) -> &str {
        "Time (in seconds)"
    }

    fn id(&self) -> &str {
        "wall-time"
    }

    fn numbers(test: &Experiment) -> &Numbers<Self::Number> {
        &test.durations
    }
}

pub struct MaxRss;

impl Measure for MaxRss {
    type Number = MemUsage;
    type NumberDisplay = u64;

    fn number_to_display(&self, number: Self::Number) -> Self::NumberDisplay {
        number.mib()
    }

    fn name(&self) -> &str {
        "Max RSS (in megabytes)"
    }

    fn id(&self) -> &str {
        "max-rss"
    }

    fn numbers(test: &Experiment) -> &Numbers<Self::Number> {
        &test.mem_usages
    }
}

pub trait MeasureDyn {
    fn name(&self) -> &str;
    fn make_distr_plots(
        &self,
        tests: &ExperimentMap<Experiment>,
        width: usize,
    ) -> anyhow::Result<ExperimentMap<String>>;
    fn display_stats(&self, tests: &ExperimentMap<Experiment>) -> ExperimentMap<String>;
    fn render_stats(
        &self,
        tests: &ExperimentMap<Experiment>,
        include_distr: bool,
    ) -> anyhow::Result<String>;
    fn write_raw(&self, tests: &ExperimentMap<Experiment>, log: &mut RunLog) -> anyhow::Result<()>;
}

impl<M: Measure> MeasureDyn for M {
    fn name(&self) -> &str {
        self.name()
    }

    fn make_distr_plots(
        &self,
        tests: &ExperimentMap<Experiment>,
        width: usize,
    ) -> anyhow::Result<ExperimentMap<String>> {
        make_distr_plots(tests, width, Self::numbers)
    }

    fn display_stats(&self, tests: &ExperimentMap<Experiment>) -> ExperimentMap<String> {
        let stats: ExperimentMap<_> = tests.map(|t| {
            Self::numbers(t)
                .stats()
                .unwrap()
                .map(|n| self.number_to_display(n))
        });
        Stats::display_stats_new(&stats)
    }

    fn render_stats(
        &self,
        tests: &ExperimentMap<Experiment>,
        include_distr: bool,
    ) -> anyhow::Result<String> {
        render_stats(tests, include_distr, self, Self::numbers)
    }

    fn write_raw(&self, tests: &ExperimentMap<Experiment>, log: &mut RunLog) -> anyhow::Result<()> {
        log.write_raw(
            self.id(),
            &tests.values().map(Self::numbers).collect::<Vec<_>>(),
        )
    }
}

pub struct AllMeasures(pub Vec<Box<dyn MeasureDyn>>);

impl AllMeasures {
    pub fn render_stats(
        &self,
        tests: &ExperimentMap<Experiment>,
        include_distr: bool,
    ) -> anyhow::Result<String> {
        let mut s = String::new();
        for (i, measure) in self.0.iter().enumerate() {
            if i != 0 {
                s.push_str("\n");
            }
            s.push_str(&measure.render_stats(tests, include_distr)?);
        }
        Ok(s)
    }

    pub fn write_raw(
        &self,
        tests: &ExperimentMap<Experiment>,
        log: &mut RunLog,
    ) -> anyhow::Result<()> {
        for measure in &self.0 {
            measure.write_raw(tests, log)?;
        }
        Ok(())
    }
}
