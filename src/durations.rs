use crate::numbers::Distr;
use crate::numbers::Numbers;
use crate::Duration;

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
            &ds.distr(1, Duration::from_millis(0), Duration::from_millis(10))
                .counts[..]
        );
        assert_eq!(
            &[1],
            &ds.distr(1, Duration::from_millis(10), Duration::from_millis(20))
                .counts[..]
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
            &ds.distr(2, Duration::from_millis(10), Duration::from_millis(20))
                .counts[..]
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
