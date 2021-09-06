use std::iter::Sum;
use std::ops::Add;
use std::ops::Div;
use std::ops::Sub;

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

pub(crate) trait Number:
    Clone + Ord + Add<Output = Self> + Sub<Output = Self> + Div<usize, Output = Self> + Sum + Default
{
    fn as_f64(&self) -> f64;

    fn from_f64(f: f64) -> Self;
}

#[derive(Default)]
pub(crate) struct Numbers<T: Number> {
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
            xy / 2
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
            self.sum() / self.len()
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
}
