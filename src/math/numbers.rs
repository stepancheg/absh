use crate::math::number::Number;
use crate::math::sorted::NumbersSorted;
use crate::math::stats::stats;
use crate::math::stats::Stats;

pub struct Distr {
    pub counts: Vec<u64>,
}

impl Distr {
    pub fn max(&self) -> u64 {
        self.counts.iter().max().cloned().unwrap_or(0)
    }

    pub fn to_f64(&self) -> Vec<f64> {
        self.counts.iter().map(|&c| c as f64).collect()
    }
}

#[derive(Default)]
pub struct Numbers {
    raw: Vec<u64>,
    sorted: Vec<u64>,
}

impl Numbers {
    pub fn push(&mut self, d: u64) {
        self.raw.push(d.clone());
        let idx = self.sorted.binary_search(&d).unwrap_or_else(|x| x);
        self.sorted.insert(idx, d);
    }

    pub fn clear(&mut self) {
        self.raw.clear();
        self.sorted.clear();
    }

    pub fn raw(&self) -> &[u64] {
        &self.raw
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn med(&self) -> Option<u64> {
        self.sorted().med()
    }

    pub fn min(&self) -> Option<u64> {
        self.sorted().min()
    }

    pub fn max(&self) -> Option<u64> {
        self.sorted().max()
    }

    pub fn sum(&self) -> u64 {
        self.sorted().sum()
    }

    pub fn mean(&self) -> Option<u64> {
        self.sorted().mean()
    }

    pub fn std(&self) -> Option<u64> {
        self.sorted().std()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = u64> + 'a {
        self.raw.iter().cloned()
    }

    pub fn sorted(&self) -> NumbersSorted<u64> {
        NumbersSorted(&self.sorted)
    }

    pub fn distr(&self, n: usize, min: u64, max: u64) -> Distr {
        let mut counts = vec![0; n];
        if min != max {
            for d in &self.raw {
                let bucket = (((d.clone() - min.clone()).as_f64())
                    / ((max.clone() - min.clone()).as_f64())
                    * ((n - 1) as f64))
                    .round() as usize;
                counts[bucket.clamp(0, n - 1)] += 1;
            }
        }
        Distr { counts }
    }

    pub fn stats(&self) -> Option<Stats<u64>> {
        stats(self)
    }
}

#[cfg(test)]
mod test {
    use std::fmt;
    use std::iter::Sum;
    use std::ops::Add;
    use std::ops::Div;
    use std::ops::Sub;

    use crate::math::numbers::Number;
    use crate::math::numbers::Numbers;

    #[derive(Copy, Clone, Default, PartialOrd, Eq, PartialEq, Ord, Debug)]
    struct TestNumber(u64);

    impl fmt::Display for TestNumber {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            fmt::Display::fmt(&self.0, f)
        }
    }

    impl Add for TestNumber {
        type Output = TestNumber;

        fn add(self, rhs: TestNumber) -> Self::Output {
            TestNumber(self.0 + rhs.0)
        }
    }

    impl Sub for TestNumber {
        type Output = Self;

        fn sub(self, rhs: Self) -> Self::Output {
            assert!(self.0 >= rhs.0);
            TestNumber(self.0 - rhs.0)
        }
    }

    impl Div<usize> for TestNumber {
        type Output = TestNumber;

        fn div(self, rhs: usize) -> Self::Output {
            TestNumber(self.0 / rhs as u64)
        }
    }

    impl Sum for TestNumber {
        fn sum<I: Iterator<Item = Self>>(iter: I) -> TestNumber {
            TestNumber(iter.map(|n| n.0).sum())
        }
    }

    impl Number for TestNumber {
        fn div_usize(&self, rhs: usize) -> Self {
            TestNumber(self.0 / (rhs as u64))
        }

        fn mul_usize(&self, rhs: usize) -> Self {
            TestNumber(self.0.checked_mul(rhs as u64).unwrap())
        }

        fn as_f64(&self) -> f64 {
            (self.0 as f64) * 1000.0
        }

        fn from_f64(f: f64) -> Self {
            TestNumber((f / 1000.0) as u64)
        }
    }

    #[test]
    fn push() {
        let mut ds = Numbers::default();
        ds.push(30);
        ds.push(50);
        ds.push(20);
        ds.push(30);
        assert_eq!(20, ds.min().unwrap());
        assert_eq!(50, ds.max().unwrap());
        ds.push(60);
        assert_eq!(60, ds.max().unwrap());
        ds.push(10);
        assert_eq!(10, ds.min().unwrap());
    }

    #[test]
    fn distr_1() {
        let mut ds = Numbers::default();
        ds.push(10);
        assert_eq!(&[1], &ds.distr(1, 0, 10).counts[..]);
        assert_eq!(&[1], &ds.distr(1, 10, 20).counts[..]);
    }

    #[test]
    fn distr_2() {
        let mut ds = Numbers::default();
        ds.push(10);
        ds.push(14);
        ds.push(16);
        ds.push(17);
        ds.push(20);
        assert_eq!(&[2, 3], &ds.distr(2, 10, 20).counts[..]);
    }

    #[test]
    fn sum() {
        let mut ds = Numbers::default();
        ds.push(10);
        ds.push(20);
        assert_eq!(30, ds.sum());
    }

    #[test]
    fn mean() {
        let mut ds = Numbers::default();

        assert_eq!(None, ds.mean());

        ds.push(10);
        ds.push(30);
        assert_eq!(20, ds.mean().unwrap());
    }

    #[test]
    fn std() {
        let mut ds = Numbers::default();
        ds.push(11);
        ds.push(13);
        ds.push(15);

        assert_eq!(2, ds.std().unwrap())
    }
}
