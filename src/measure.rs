use crate::distr_plot::make_distr_plots;
use crate::render_stats::render_stats;
use crate::stats::Stats;
use crate::test::Test;
use crate::Duration;
use crate::MemUsage;
use crate::Number;
use crate::Numbers;

pub trait Measure {
    type Number: Number;
    fn name(&self) -> &str;
    fn numbers(test: &Test) -> &Numbers<Self::Number>;
}

pub struct WallTime;

impl Measure for WallTime {
    type Number = Duration;

    fn name(&self) -> &str {
        "Time (in seconds)"
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

    fn numbers(test: &Test) -> &Numbers<Self::Number> {
        &test.mem_usages
    }
}

pub trait MeasureDyn {
    fn name(&self) -> &str;
    fn make_distr_plots(&self, tests: &[Test], width: usize) -> anyhow::Result<Vec<String>>;
    fn display_stats(&self, tests: &[Test]) -> Vec<String>;
    fn render_stats(&self, tests: &[Test], include_distr: bool) -> anyhow::Result<String>;
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
}
