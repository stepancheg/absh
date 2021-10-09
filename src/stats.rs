use crate::number::Number;
use crate::numbers::Numbers;
use std::fmt;
use std::slice;

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

    pub fn display_stats(stats: &[Stats<T>]) -> Vec<String> {
        struct MultiWriter<'s, N: Number> {
            vec: Vec<String>,
            stats: &'s [Stats<N>],
        }

        impl<'s, N: Number> MultiWriter<'s, N> {
            fn append_n<D: fmt::Display>(&mut self, v: impl Fn(&Stats<N>) -> D) -> fmt::Result {
                use std::fmt::Write;
                let values: Vec<String> = self.stats.iter().map(|s| v(s).to_string()).collect();
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

            fn append_column<D: fmt::Display>(
                &mut self,
                name: &str,
                v: impl Fn(&Stats<N>) -> D,
            ) -> fmt::Result {
                if !self.vec[0].is_empty() {
                    self.append_str(" ")?;
                }
                self.append_str(name)?;
                self.append_n(v)?;
                Ok(())
            }

            fn append_stats(&mut self) -> fmt::Result {
                self.append_column("n=", |s| s.count)?;
                self.append_column("mean=", |s| s.mean.display_for_stats())?;
                self.append_column("std=", |s| s.std.display_for_stats())?;
                self.append_column("se=", |s| s.se().display_for_stats())?;
                self.append_column("min=", |s| s.min.display_for_stats())?;
                self.append_column("max=", |s| s.max.display_for_stats())?;
                self.append_column("med=", |s| s.med.display_for_stats())?;
                Ok(())
            }
        }

        let mut w = MultiWriter {
            vec: vec![String::new(); stats.len()],
            stats,
        };
        w.append_stats().unwrap();
        w.vec
    }
}

impl<T: Number> fmt::Display for Stats<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strings = Stats::display_stats(slice::from_ref(self));
        assert!(strings.len() == 1);
        f.write_str(&strings[0])
    }
}

pub(crate) fn stats<T: Number>(numbers: &Numbers<T>) -> Stats<T> {
    assert!(numbers.len() >= 2);

    Stats {
        count: numbers.len() as u64,
        mean: numbers.mean(),
        med: numbers.med(),
        min: numbers.min(),
        max: numbers.max(),
        std: numbers.std(),
    }
}

#[cfg(test)]
mod test {
    use crate::numbers::Numbers;
    use crate::stats::stats;

    #[test]
    fn se() {
        let mut numbers = Numbers::default();
        numbers.push(10u64);
        numbers.push(20u64);
        numbers.push(30u64);
        numbers.push(30u64);
        numbers.push(30u64);
        let stats = stats(&numbers);
        assert_eq!(4, stats.se());
    }
}
