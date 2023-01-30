use std::fmt;
use std::iter::Sum;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Sub;

use crate::math::number::Number;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Debug)]
pub struct Duration {
    nanos: u64,
}

impl Number for Duration {
    fn div_usize(&self, rhs: usize) -> Self {
        Duration {
            nanos: self.nanos / (rhs as u64),
        }
    }

    fn mul_usize(&self, rhs: usize) -> Self {
        Duration {
            nanos: self.nanos.checked_mul(rhs as u64).unwrap(),
        }
    }

    fn as_f64(&self) -> f64 {
        self.seconds_f64()
    }

    fn from_f64(f: f64) -> Self {
        Duration::from_seconds_f64(f)
    }
}

impl Duration {
    pub fn from_nanos(nanos: u64) -> Duration {
        Duration { nanos }
    }

    pub fn from_nanos_f64(nanos: f64) -> Duration {
        Self::from_nanos(nanos as u64)
    }

    pub fn from_millis(millis: u64) -> Duration {
        Self::from_nanos(millis.checked_mul(1_000_000).unwrap())
    }

    pub fn from_seconds_f64(seconds: f64) -> Duration {
        Self::from_nanos_f64(seconds * 1_000_000_000.0)
    }

    pub fn nanos(&self) -> u64 {
        self.nanos
    }

    pub fn millis(&self) -> u64 {
        self.nanos / 1_000_000
    }

    pub fn millis_f64(&self) -> f64 {
        self.nanos as f64 / 1_000_000.0
    }

    pub fn seconds_f64(&self) -> f64 {
        self.nanos as f64 / 1_000_000_000.0
    }
}

impl Sub for Duration {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration {
            nanos: self.nanos.checked_sub(rhs.nanos).unwrap(),
        }
    }
}

impl Add for Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Duration {
            nanos: self.nanos.checked_add(rhs.nanos).unwrap(),
        }
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Self) {
        self.nanos = self.nanos.checked_add(rhs.nanos).unwrap();
    }
}

impl Div<u64> for Duration {
    type Output = Duration;

    fn div(self, rhs: u64) -> Self::Output {
        Duration {
            nanos: self.nanos / rhs,
        }
    }
}

impl Div<usize> for Duration {
    type Output = Duration;

    fn div(self, rhs: usize) -> Self::Output {
        self / (rhs as u64)
    }
}

impl Div<i32> for Duration {
    type Output = Duration;

    fn div(self, rhs: i32) -> Self::Output {
        assert!(rhs > 0);
        self / (rhs as u64)
    }
}

impl Div<Duration> for Duration {
    type Output = f64;

    fn div(self, rhs: Duration) -> Self::Output {
        self.nanos as f64 / rhs.nanos as f64
    }
}

impl Sum for Duration {
    fn sum<I: Iterator<Item = Duration>>(iter: I) -> Self {
        let mut sum = Duration::default();
        for d in iter {
            sum += d;
        }
        sum
    }
}

impl<'a> Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item = &'a Duration>>(iter: I) -> Self {
        let mut sum = Duration::default();
        for d in iter {
            sum += *d;
        }
        sum
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:03}", self.millis() / 1000, self.millis() % 1000)
    }
}
