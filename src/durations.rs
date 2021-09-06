use crate::numbers::Distr;
use crate::numbers::Numbers;
use crate::stats::stats;
use crate::Duration;
use crate::Stats;

#[derive(Default)]
pub struct Durations {
    numbers: Numbers<Duration>,
}

impl Durations {
    pub fn push(&mut self, d: Duration) {
        self.numbers.push(d)
    }

    pub fn clear(&mut self) {
        self.numbers.clear()
    }

    pub fn raw(&self) -> &[Duration] {
        self.numbers.raw()
    }

    pub fn len(&self) -> usize {
        self.numbers.len()
    }

    pub fn med(&self) -> Duration {
        self.numbers.med()
    }

    pub fn min(&self) -> Duration {
        self.numbers.min()
    }

    pub fn max(&self) -> Duration {
        self.numbers.max()
    }

    pub fn sum(&self) -> Duration {
        self.numbers.sum()
    }

    pub fn mean(&self) -> Duration {
        self.numbers.mean()
    }

    pub fn std(&self) -> Duration {
        self.numbers.std()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Duration> + 'a {
        self.numbers.iter()
    }

    pub fn distr(&self, n: usize, min: Duration, max: Duration) -> Distr {
        self.numbers.distr(n, min, max)
    }

    pub fn stats(&self) -> Stats<Duration> {
        stats(&self.numbers)
    }
}
