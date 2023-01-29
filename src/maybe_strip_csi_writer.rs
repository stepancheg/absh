use crate::ansi::strip_csi;
use std::fmt;
use std::fmt::Write;
use std::io;

pub struct MaybeStripCsiWriter<W: io::Write> {
    pub inner: W,
    pub strip: bool,
}

impl<W: io::Write> Write for MaybeStripCsiWriter<W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.strip {
            write!(self.inner, "{}", strip_csi(s)).map_err(|_| fmt::Error)
        } else {
            write!(self.inner, "{}", s).map_err(|_| fmt::Error)
        }
    }
}
