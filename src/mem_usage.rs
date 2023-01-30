use std::fmt;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Sub;

use crate::math::number::Number;

#[derive(Copy, Clone, Default, PartialOrd, Ord, PartialEq, Eq)]
pub struct MemUsage {
    bytes: u64,
}

impl MemUsage {
    pub fn from_bytes(bytes: u64) -> MemUsage {
        MemUsage { bytes }
    }

    pub fn mib(&self) -> u64 {
        self.bytes >> 20
    }

    pub fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl Add for MemUsage {
    type Output = MemUsage;

    fn add(self, rhs: Self) -> Self::Output {
        MemUsage {
            bytes: self.bytes + rhs.bytes,
        }
    }
}

impl Sub for MemUsage {
    type Output = MemUsage;

    fn sub(self, rhs: Self) -> Self::Output {
        MemUsage {
            bytes: self.bytes.checked_sub(rhs.bytes).unwrap(),
        }
    }
}

impl Sum for MemUsage {
    fn sum<I: Iterator<Item = MemUsage>>(iter: I) -> MemUsage {
        MemUsage {
            bytes: iter.map(|m| m.bytes).sum(),
        }
    }
}

impl fmt::Display for MemUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.bytes)
    }
}

impl Number for MemUsage {
    fn div_usize(&self, rhs: usize) -> Self {
        MemUsage {
            bytes: self.bytes / (rhs as u64),
        }
    }

    fn mul_usize(&self, rhs: usize) -> Self {
        MemUsage {
            bytes: self.bytes.checked_mul(rhs as u64).unwrap(),
        }
    }

    fn as_f64(&self) -> f64 {
        self.bytes as f64
    }

    fn from_f64(f: f64) -> Self {
        MemUsage { bytes: f as u64 }
    }
}
