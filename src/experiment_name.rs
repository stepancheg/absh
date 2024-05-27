use std::fmt;

use crate::ansi;
use crate::ansi::AnsiColor;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExperimentName {
    A,
    B,
    C,
    D,
    E,
}

impl ExperimentName {
    pub(crate) fn all() -> [ExperimentName; 5] {
        [
            ExperimentName::A,
            ExperimentName::B,
            ExperimentName::C,
            ExperimentName::D,
            ExperimentName::E,
        ]
    }

    pub fn index(&self) -> usize {
        match self {
            ExperimentName::A => 0,
            ExperimentName::B => 1,
            ExperimentName::C => 2,
            ExperimentName::D => 3,
            ExperimentName::E => 4,
        }
    }

    pub fn from_index(index: usize) -> ExperimentName {
        match index {
            0 => ExperimentName::A,
            1 => ExperimentName::B,
            2 => ExperimentName::C,
            3 => ExperimentName::D,
            4 => ExperimentName::E,
            _ => panic!("Invalid index: {}", index),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ExperimentName::A => "A",
            ExperimentName::B => "B",
            ExperimentName::C => "C",
            ExperimentName::D => "D",
            ExperimentName::E => "E",
        }
    }

    pub(crate) fn lower(&self) -> &str {
        match self {
            ExperimentName::A => "a",
            ExperimentName::B => "b",
            ExperimentName::C => "c",
            ExperimentName::D => "d",
            ExperimentName::E => "e",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            ExperimentName::A => AnsiColor::Red.fg(),
            ExperimentName::B => AnsiColor::Green.fg(),
            ExperimentName::C => AnsiColor::Blue.fg(),
            ExperimentName::D => AnsiColor::Magenta.fg(),
            ExperimentName::E => AnsiColor::Cyan.fg(),
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
