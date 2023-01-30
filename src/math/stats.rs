use std::fmt;

use crate::math::number::Number;
use crate::math::numbers::Numbers;
use crate::measure::Measure;

pub struct Stats<T: Number> {
    pub count: u64,
    pub mean: T,
    pub med: T,
    pub min: T,
    pub max: T,
    pub std: T,
}

impl<T: Number> Stats<T> {
    fn se(&self) -> T {
        T::from_f64(self.std.as_f64() / f64::sqrt((self.count - 1) as f64))
    }

    /// sigma^2
    pub fn sigma_sq(&self) -> f64 {
        let millis = self.std.as_f64();
        millis * millis
    }

    pub(crate) fn display_stats<M>(stats: &[Stats<T>], m: &M) -> Vec<String>
    where
        M: Measure<Number = T>,
    {
        struct MultiWriter<'s, M: Measure> {
            vec: Vec<String>,
            m: &'s M,
            stats: &'s [Stats<M::Number>],
        }

        impl<'s, M: Measure> MultiWriter<'s, M> {
            fn append_n<'d, D: fmt::Display + 'd>(
                &mut self,
                v: impl Fn(&'d M, &Stats<M::Number>) -> D,
            ) -> fmt::Result
            where
                's: 'd,
            {
                use std::fmt::Write;
                let values: Vec<String> = self
                    .stats
                    .iter()
                    .map(|s| v(self.m, s).to_string())
                    .collect();
                let max_len = values.iter().map(|s| s.len()).max().unwrap();
                for (r, s) in self.vec.iter_mut().zip(&values) {
                    write!(r, "{:>width$}", s, width = max_len)?;
                }
                Ok(())
            }

            fn append_str(&mut self, s: &str) -> fmt::Result {
                for r in &mut self.vec {
                    r.push_str(s);
                }
                Ok(())
            }

            fn append_column<'d, D: fmt::Display + 'd>(
                &mut self,
                name: &str,
                v: impl Fn(&'d M, &Stats<M::Number>) -> D,
            ) -> fmt::Result
            where
                's: 'd,
            {
                if !self.vec[0].is_empty() {
                    self.append_str(" ")?;
                }
                self.append_str(name)?;
                self.append_n(v)?;
                Ok(())
            }

            fn append_stats(&mut self) -> fmt::Result {
                self.append_column("n=", |_m, s| s.count)?;
                self.append_column("mean=", |m, s| m.number_display_for_stats(s.mean))?;
                self.append_column("std=", |m, s| m.number_display_for_stats(s.std))?;
                self.append_column("se=", |m, s| m.number_display_for_stats(s.se()))?;
                self.append_column("min=", |m, s| m.number_display_for_stats(s.min))?;
                self.append_column("max=", |m, s| m.number_display_for_stats(s.max))?;
                self.append_column("med=", |m, s| m.number_display_for_stats(s.med))?;
                Ok(())
            }
        }

        let mut w = MultiWriter::<M> {
            vec: vec![String::new(); stats.len()],
            stats,
            m,
        };
        w.append_stats().unwrap();
        w.vec
    }
}

pub(crate) fn stats<T: Number>(numbers: &Numbers<T>) -> Option<Stats<T>> {
    assert!(numbers.len() >= 2);

    Some(Stats {
        count: numbers.len() as u64,
        mean: numbers.mean()?,
        med: numbers.med()?,
        min: numbers.min()?,
        max: numbers.max()?,
        std: numbers.std()?,
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
        assert_eq!(4, stats.se());
    }
}
