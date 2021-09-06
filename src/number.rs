use std::fmt;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Sub;

pub trait Number:
    Clone + Ord + Add<Output = Self> + Sub<Output = Self> + Sum + Default + fmt::Display
{
    fn div_usize(&self, rhs: usize) -> Self;

    fn as_f64(&self) -> f64;

    fn from_f64(f: f64) -> Self;
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
