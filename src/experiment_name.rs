use std::fmt;

use crate::ansi;

#[derive(Clone, Copy, Debug)]
pub enum ExperimentName {
    A,
    B,
    C,
    D,
    E,
}

impl ExperimentName {
    pub fn name(&self) -> &str {
        match self {
            ExperimentName::A => "A",
            ExperimentName::B => "B",
            ExperimentName::C => "C",
            ExperimentName::D => "D",
            ExperimentName::E => "E",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            ExperimentName::A => ansi::RED,
            ExperimentName::B => ansi::GREEN,
            ExperimentName::C => ansi::BLUE,
            ExperimentName::D => ansi::MAGENTA,
            ExperimentName::E => ansi::CYAN,
        }
    }

    pub fn name_colored(&self) -> String {
        format!("{}{}{}", self.color(), self.name(), ansi::RESET)
    }
}

impl fmt::Display for ExperimentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
