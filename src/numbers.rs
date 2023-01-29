use crate::number::Number;
use crate::stats::stats;
use crate::stats::Stats;

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
pub struct Numbers<T: Number> {
    raw: Vec<T>,
    sorted: Vec<T>,
}

impl<T: Number> Numbers<T> {
    pub fn push(&mut self, d: T) {
        self.raw.push(d.clone());
        let idx = self.sorted.binary_search(&d).unwrap_or_else(|x| x);
        self.sorted.insert(idx, d);
    }

    pub fn clear(&mut self) {
        self.raw.clear();
        self.sorted.clear();
    }

    pub fn raw(&self) -> &[T] {
        &self.raw
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn med(&self) -> T {
        if self.len() % 2 == 0 {
            let xy: T =
                self.sorted[self.len() / 2 - 1].clone() + self.sorted[self.len() / 2].clone();
            xy.div_usize(2)
        } else {
            self.sorted[self.len() / 2].clone()
        }
    }

    pub fn min(&self) -> T {
        self.sorted[0].clone()
    }

    pub fn max(&self) -> T {
        self.sorted.last().unwrap().clone()
    }

    pub fn sum(&self) -> T {
        self.raw.iter().cloned().sum()
    }

    pub fn mean(&self) -> T {
        if self.len() == 0 {
            T::default()
        } else {
            self.sum().div_usize(self.len())
        }
    }

    pub fn std(&self) -> T {
        assert!(self.len() >= 2);
        let mean = self.mean();
        let s_2 = self
            .raw
            .iter()
            .map(|d| (d.as_f64() - mean.as_f64()) * (d.as_f64() - mean.as_f64()))
            .sum::<f64>()
            / ((self.len() - 1) as f64);
        let std_seconds = f64::sqrt(s_2);

        T::from_f64(std_seconds)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = T> + 'a {
        self.raw.iter().cloned()
    }

    pub fn distr(&self, n: usize, min: T, max: T) -> Distr {
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

    pub fn stats(&self) -> Stats<T> {
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

    use crate::numbers::Number;
    use crate::numbers::Numbers;

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
        ds.push(TestNumber(30));
        ds.push(TestNumber(50));
        ds.push(TestNumber(20));
        ds.push(TestNumber(30));
        assert_eq!(TestNumber(20), ds.min());
        assert_eq!(TestNumber(50), ds.max());
        ds.push(TestNumber(60));
        assert_eq!(TestNumber(60), ds.max());
        ds.push(TestNumber(10));
        assert_eq!(TestNumber(10), ds.min());
    }

    #[test]
    fn distr_1() {
        let mut ds = Numbers::default();
        ds.push(TestNumber(10));
        assert_eq!(&[1], &ds.distr(1, TestNumber(0), TestNumber(10)).counts[..]);
        assert_eq!(
            &[1],
            &ds.distr(1, TestNumber(10), TestNumber(20)).counts[..]
        );
    }

    #[test]
    fn distr_2() {
        let mut ds = Numbers::default();
        ds.push(TestNumber(10));
        ds.push(TestNumber(14));
        ds.push(TestNumber(16));
        ds.push(TestNumber(17));
        ds.push(TestNumber(20));
        assert_eq!(
            &[2, 3],
            &ds.distr(2, TestNumber(10), TestNumber(20)).counts[..]
        );
    }

    #[test]
    fn sum() {
        let mut ds = Numbers::default();
        ds.push(TestNumber(10));
        ds.push(TestNumber(20));
        assert_eq!(TestNumber(30), ds.sum());
    }

    #[test]
    fn mean() {
        let mut ds = Numbers::default();

        assert_eq!(TestNumber::default(), ds.mean());

        ds.push(TestNumber(10));
        ds.push(TestNumber(30));
        assert_eq!(TestNumber(20), ds.mean());
    }

    #[test]
    fn std() {
        let mut ds = Numbers::default();
        ds.push(TestNumber(11));
        ds.push(TestNumber(13));
        ds.push(TestNumber(15));

        assert_eq!(TestNumber(2), ds.std())
    }
}
