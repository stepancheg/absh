use crate::ansi;
use crate::bars::PlotHighlight;
use crate::duration::Duration;
use crate::mem_usage::MemUsage;
use crate::numbers::Numbers;
use crate::test_name::TestName;

pub struct Test {
    pub name: TestName,
    pub warmup: String,
    pub run: String,
    pub durations: Numbers<Duration>,
    pub mem_usages: Numbers<MemUsage>,
}

impl Test {
    pub fn plot_highlights(&self) -> PlotHighlight {
        PlotHighlight {
            non_zero: format!("{}", self.name.color().to_owned()),
            zero: format!("{}", ansi::WHITE_BG),
            reset: ansi::RESET.to_owned(),
        }
    }

    pub fn plot_halves_highlights(&self) -> PlotHighlight {
        PlotHighlight {
            non_zero: format!("{}", self.name.color().to_owned()),
            zero: "".to_owned(),
            reset: ansi::RESET.to_owned(),
        }
    }
}
