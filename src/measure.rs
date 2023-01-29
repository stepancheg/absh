use crate::distr_plot::make_distr_plots;
use crate::duration::Duration;
use crate::mem_usage::MemUsage;
use crate::number::Number;
use crate::numbers::Numbers;
use crate::render_stats::render_stats;
use crate::run_log::RunLog;
use crate::stats::Stats;
use crate::test::Test;

pub(crate) trait Measure {
    type Number: Number;
    fn name(&self) -> &str;
    fn id(&self) -> &str;
    fn numbers(test: &Test) -> &Numbers<Self::Number>;
}

pub struct WallTime;

impl Measure for WallTime {
    type Number = Duration;

    fn name(&self) -> &str {
        "Time (in seconds)"
    }

    fn id(&self) -> &str {
        "wall-time"
    }

    fn numbers(test: &Test) -> &Numbers<Self::Number> {
        &test.durations
    }
}

pub struct MaxRss;

impl Measure for MaxRss {
    type Number = MemUsage;

    fn name(&self) -> &str {
        "Max RSS (in megabytes)"
    }

    fn id(&self) -> &str {
        "max-rss"
    }

    fn numbers(test: &Test) -> &Numbers<Self::Number> {
        &test.mem_usages
    }
}

pub trait MeasureDyn {
    fn name(&self) -> &str;
    fn make_distr_plots(&self, tests: &[Test], width: usize) -> anyhow::Result<Vec<String>>;
    fn display_stats(&self, tests: &[Test]) -> Vec<String>;
    fn render_stats(&self, tests: &[Test], include_distr: bool) -> anyhow::Result<String>;
    fn write_raw(&self, tests: &[Test], log: &mut RunLog) -> anyhow::Result<()>;
}

impl<M: Measure> MeasureDyn for M {
    fn name(&self) -> &str {
        self.name()
    }

    fn make_distr_plots(&self, tests: &[Test], width: usize) -> anyhow::Result<Vec<String>> {
        make_distr_plots(tests, width, Self::numbers)
    }

    fn display_stats(&self, tests: &[Test]) -> Vec<String> {
        let stats: Vec<_> = tests.iter().map(|t| Self::numbers(t).stats()).collect();
        Stats::display_stats(&stats)
    }

    fn render_stats(&self, tests: &[Test], include_distr: bool) -> anyhow::Result<String> {
        render_stats(tests, include_distr, self, Self::numbers)
    }

    fn write_raw(&self, tests: &[Test], log: &mut RunLog) -> anyhow::Result<()> {
        log.write_raw(
            self.id(),
            &tests.iter().map(Self::numbers).collect::<Vec<_>>(),
        )
    }
}

pub struct AllMeasures(pub Vec<Box<dyn MeasureDyn>>);

impl AllMeasures {
    pub fn render_stats(&self, tests: &[Test], include_distr: bool) -> anyhow::Result<String> {
        let mut s = String::new();
        for (i, measure) in self.0.iter().enumerate() {
            if i != 0 {
                s.push_str("\n");
            }
            s.push_str(&measure.render_stats(tests, include_distr)?);
        }
        Ok(s)
    }

    pub fn write_raw(&self, tests: &[Test], log: &mut RunLog) -> anyhow::Result<()> {
        for measure in &self.0 {
            measure.write_raw(tests, log)?;
        }
        Ok(())
    }
}
