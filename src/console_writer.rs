use crate::ansi::strip_csi;
use std::fmt;

#[derive(Debug)]
pub struct ConsoleWriter {
    color: bool,
}

fn is_tty() -> bool {
    !cfg!(windows) && atty::is(atty::Stream::Stderr)
}

impl ConsoleWriter {
    pub fn auto() -> ConsoleWriter {
        ConsoleWriter { color: is_tty() }
    }
}

impl fmt::Write for ConsoleWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.color {
            eprint!("{}", s);
        } else {
            eprint!("{}", strip_csi(s));
        }
        Ok(())
    }
}
