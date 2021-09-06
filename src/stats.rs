use crate::number::Number;
use crate::numbers::Numbers;
use crate::Duration;
use std::fmt;

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
}

impl Stats<Duration> {
    /// sigma^2
    pub fn var_millis_sq(&self) -> f64 {
        let millis = self.std.millis_f64();
        millis * millis
    }
}

impl fmt::Display for Stats<Duration> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = self.max;
        write!(
            f,
            "n={n} mean={mean} std={std} se={se} min={min} max={max} med={med}",
            n = self.count,
            mean = self.mean,
            std = self.std,
            se = self.se(),
            min = self.min,
            max = self.max,
            med = self.med,
        )
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
