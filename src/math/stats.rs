use std::fmt;
use std::fmt::Display;

use crate::experiment_map::ExperimentMap;
use crate::math::number::Number;
use crate::math::numbers::Numbers;

pub struct Stats<A> {
    pub count: u64,
    pub mean: A,
    pub med: A,
    pub min: A,
    pub max: A,
    pub std: A,
    pub se: A,
}

impl<A> Stats<A> {
    pub fn map<B>(self, mut f: impl FnMut(A) -> B) -> Stats<B> {
        Stats {
            count: self.count,
            mean: f(self.mean),
            med: f(self.med),
            min: f(self.min),
            max: f(self.max),
            std: f(self.std),
            se: f(self.se),
        }
    }
}

impl<T: Number> Stats<T> {
    /// sigma^2
    pub fn sigma_sq(&self) -> f64 {
        let millis = self.std.as_f64();
        millis * millis
    }
}

impl<A: Display + Copy> Stats<A> {
    pub(crate) fn display_stats_new(stats: &ExperimentMap<Stats<A>>) -> ExperimentMap<String> {
        struct MultiWriter<'s, A> {
            vec: ExperimentMap<String>,
            stats: &'s ExperimentMap<Stats<A>>,
        }

        impl<'s, A: Display + Copy> MultiWriter<'s, A> {
            fn append_n<D: Display>(&mut self, v: impl Fn(&Stats<A>) -> D) -> fmt::Result {
                use std::fmt::Write;
                let values: ExperimentMap<String> = self.stats.map(|s| v(s).to_string());
                let max_len = values.values().map(|s| s.len()).max().unwrap();
                for (r, s) in self.vec.values_mut().zip(values.values()) {
                    write!(r, "{:>width$}", s, width = max_len)?;
                }
                Ok(())
            }

            fn append_str(&mut self, s: &str) -> fmt::Result {
                for r in self.vec.values_mut() {
                    r.push_str(s);
                }
                Ok(())
            }

            fn append_column<'d, D: Display + 'd>(
                &mut self,
                name: &str,
                v: impl Fn(&Stats<A>) -> D,
            ) -> fmt::Result
            where
                's: 'd,
            {
                if !self.vec.values().next().unwrap().is_empty() {
                    self.append_str(" ")?;
                }
                self.append_str(name)?;
                self.append_n(v)?;
                Ok(())
            }

            fn append_stats(&mut self) -> fmt::Result {
                self.append_column("n=", |s| s.count)?;
                self.append_column("mean=", |s| s.mean)?;
                self.append_column("std=", |s| s.std)?;
                self.append_column("se=", |s| s.se)?;
                self.append_column("min=", |s| s.min)?;
                self.append_column("max=", |s| s.max)?;
                self.append_column("med=", |s| s.med)?;
                Ok(())
            }
        }

        let mut w = MultiWriter {
            vec: stats.map(|_| String::new()),
            stats,
        };
        w.append_stats().unwrap();
        w.vec
    }
}

pub(crate) fn stats<T: Number>(numbers: &Numbers<T>) -> Option<Stats<T>> {
    assert!(numbers.len() >= 2);

    let std = numbers.std()?;
    let se = T::from_f64(std.as_f64() / f64::sqrt((numbers.len() - 1) as f64));
    Some(Stats {
        count: numbers.len() as u64,
        mean: numbers.mean()?,
        med: numbers.med()?,
        min: numbers.min()?,
        max: numbers.max()?,
        std,
        se,
    })
}

#[cfg(test)]
mod test {
    use crate::math::numbers::Numbers;
    use crate::math::stats::stats;

    #[test]
    fn se() {
        let mut numbers = Numbers::default();
        numbers.push(10u64);
        numbers.push(20u64);
        numbers.push(30u64);
        numbers.push(30u64);
        numbers.push(30u64);
        let stats = stats(&numbers).unwrap();
        assert_eq!(4, stats.se);
    }
}
