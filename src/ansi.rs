use once_cell::sync::Lazy;
use regex::Regex;

/// ANSI color.
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl AnsiColor {
    pub fn fg(&self) -> &'static str {
        match self {
            AnsiColor::Black => "\x1B[30m",
            AnsiColor::Red => "\x1B[31m",
            AnsiColor::Green => "\x1B[32m",
            AnsiColor::Yellow => "\x1B[33m",
            AnsiColor::Blue => "\x1B[34m",
            AnsiColor::Magenta => "\x1B[35m",
            AnsiColor::Cyan => "\x1B[36m",
            AnsiColor::White => "\x1B[37m",
        }
    }

    pub fn bg(&self) -> &'static str {
        match self {
            AnsiColor::Black => "\x1B[40m",
            AnsiColor::Red => "\x1B[41m",
            AnsiColor::Green => "\x1B[42m",
            AnsiColor::Yellow => "\x1B[43m",
            AnsiColor::Blue => "\x1B[44m",
            AnsiColor::Magenta => "\x1B[45m",
            AnsiColor::Cyan => "\x1B[46m",
            AnsiColor::White => "\x1B[47m",
        }
    }
}

/// Reset color
pub const RESET: &str = "\x1B[0m";

// https://en.wikipedia.org/wiki/ANSI_escape_code#CSIsection
pub fn strip_csi(s: &str) -> String {
    static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("\x1b\\[[0-9]+[a-zA-Z]").unwrap());
    REGEX.replace_all(s, "").into_owned()
}

#[cfg(test)]
mod test {
    use crate::ansi::strip_csi;

    #[test]
    fn test() {
        assert_eq!("A\nB\n", strip_csi("\x1B[32mA\x1B[0m\n\x1B[31mB\x1B[0m\n"));
    }
}
