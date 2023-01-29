use crate::ansi;
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub enum TestName {
    A,
    B,
    C,
    D,
    E,
}

impl TestName {
    pub fn name(&self) -> &str {
        match self {
            TestName::A => "A",
            TestName::B => "B",
            TestName::C => "C",
            TestName::D => "D",
            TestName::E => "E",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            TestName::A => ansi::RED,
            TestName::B => ansi::GREEN,
            TestName::C => ansi::BLUE,
            TestName::D => ansi::MAGENTA,
            TestName::E => ansi::CYAN,
        }
    }

    pub fn name_colored(&self) -> String {
        format!("{}{}{}", self.color(), self.name(), ansi::RESET)
    }
}

impl fmt::Display for TestName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
