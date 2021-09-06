use std::iter::Sum;
use std::ops::Add;
use std::ops::Div;
use std::ops::Sub;

pub trait Number:
    Clone + Ord + Add<Output = Self> + Sub<Output = Self> + Div<usize, Output = Self> + Sum + Default
{
    fn as_f64(&self) -> f64;

    fn from_f64(f: f64) -> Self;
}
