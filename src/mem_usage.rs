use std::fmt;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Sub;

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
