use std::fmt;
use std::fmt::Formatter;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Sub;

pub trait Number:
    Clone + Ord + Add<Output = Self> + Sub<Output = Self> + Sum + Default + fmt::Display
{
    fn div_usize(&self, rhs: usize) -> Self;

    fn as_f64(&self) -> f64;

    fn from_f64(f: f64) -> Self;

    fn fmt_for_stats(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }

    fn display_for_stats(&self) -> NumberDisplayForStats<Self> {
        NumberDisplayForStats(self.clone())
    }
}

pub struct NumberDisplayForStats<N: Number>(N);

impl<N: Number> fmt::Display for NumberDisplayForStats<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt_for_stats(f)
    }
}

impl Number for u64 {
    fn div_usize(&self, rhs: usize) -> Self {
        *self / (rhs as u64)
    }

    fn as_f64(&self) -> f64 {
        *self as f64
    }

    fn from_f64(f: f64) -> Self {
        f as u64
    }
}
