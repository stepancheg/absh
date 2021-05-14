use crate::Duration;

#[derive(Default)]
pub struct Durations {
    raw: Vec<Duration>,
    sorted: Vec<Duration>,
}

impl Durations {
    pub fn push(&mut self, d: Duration) {
        self.raw.push(d);
        let idx = self.sorted.binary_search(&d).unwrap_or_else(|x| x);
        self.sorted.insert(idx, d);
        // self.sorted.push(d);
        // self.sorted.sort();
    }

    pub fn raw(&self) -> &[Duration] {
        &self.raw
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    pub fn med(&self) -> Duration {
        if self.len() % 2 == 0 {
            (self.sorted[self.len() / 2 - 1] + self.sorted[self.len() / 2]) / 2
        } else {
            self.sorted[self.len() / 2]
        }
    }

    pub fn min(&self) -> Duration {
        self.sorted[0]
    }

    pub fn max(&self) -> Duration {
        self.sorted.last().unwrap().clone()
    }

    pub fn sum(&self) -> Duration {
        self.raw.iter().sum()
    }

    pub fn mean(&self) -> Duration {
        if self.len() == 0 {
            Duration::default()
        } else {
            self.sum() / self.len()
        }
    }

    pub fn std(&self) -> Duration {
        assert!(self.len() >= 2);
        let mean = self.mean();
        let s_2 = self
            .raw
            .iter()
            .map(|d| {
                (d.seconds_f64() - mean.seconds_f64()) * (d.seconds_f64() - mean.seconds_f64())
            })
            .sum::<f64>()
            / ((self.len() - 1) as f64);
        let std_seconds = f64::sqrt(s_2);

        Duration::from_seconds_f64(std_seconds)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Duration> + 'a {
        self.raw.iter().cloned()
    }

    pub fn distr(&self, n: usize, min: Duration, max: Duration) -> Vec<usize> {
        let mut r = vec![0; n];
        if min != max {
            for d in &self.raw {
                let bucket = (((*d - min).nanos() as f64) / ((max - min).nanos() as f64)
                    * ((n - 1) as f64))
                    .round() as usize;
                r[bucket.clamp(0, n - 1)] += 1;
            }
        }
        r
    }
}

#[cfg(test)]
mod test {
    use crate::Duration;
    use crate::Durations;

    #[test]
    fn push() {
        let mut ds = Durations::default();
        ds.push(Duration::from_millis(30));
        ds.push(Duration::from_millis(50));
        ds.push(Duration::from_millis(20));
        ds.push(Duration::from_millis(30));
        assert_eq!(Duration::from_millis(20), ds.min());
        assert_eq!(Duration::from_millis(50), ds.max());
        ds.push(Duration::from_millis(60));
        assert_eq!(Duration::from_millis(60), ds.max());
        ds.push(Duration::from_millis(10));
        assert_eq!(Duration::from_millis(10), ds.min());
    }

    #[test]
    fn distr_1() {
        let mut ds = Durations::default();
        ds.push(Duration::from_millis(10));
        assert_eq!(
            &[1],
            &ds.distr(1, Duration::from_millis(0), Duration::from_millis(10))[..]
        );
        assert_eq!(
            &[1],
            &ds.distr(1, Duration::from_millis(10), Duration::from_millis(20))[..]
        );
    }

    #[test]
    fn distr_2() {
        let mut ds = Durations::default();
        ds.push(Duration::from_millis(10));
        ds.push(Duration::from_millis(14));
        ds.push(Duration::from_millis(16));
        ds.push(Duration::from_millis(17));
        ds.push(Duration::from_millis(20));
        assert_eq!(
            &[2, 3],
            &ds.distr(2, Duration::from_millis(10), Duration::from_millis(20))[..]
        );
    }

    #[test]
    fn sum() {
        let mut ds = Durations::default();
        ds.push(Duration::from_millis(10));
        ds.push(Duration::from_millis(20));
        assert_eq!(Duration::from_millis(30), ds.sum());
    }

    #[test]
    fn mean() {
        let mut ds = Durations::default();

        assert_eq!(Duration::default(), ds.mean());

        ds.push(Duration::from_millis(10));
        ds.push(Duration::from_millis(30));
        assert_eq!(Duration::from_millis(20), ds.mean());
    }

    #[test]
    fn std() {
        let mut ds = Durations::default();
        ds.push(Duration::from_millis(11));
        ds.push(Duration::from_millis(13));
        ds.push(Duration::from_millis(15));

        assert_eq!(Duration::from_millis(2), ds.std())
    }
}
