/// Map small integer to a value.
pub struct LinearMap<A> {
    values: Vec<Option<A>>,
}

impl<A> Default for LinearMap<A> {
    fn default() -> LinearMap<A> {
        LinearMap { values: Vec::new() }
    }
}

impl<A> LinearMap<A> {
    pub fn get(&self, index: usize) -> Option<&A> {
        self.values.get(index).and_then(|v| v.as_ref())
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut A> {
        self.values.get_mut(index).and_then(|v| v.as_mut())
    }

    pub fn insert(&mut self, index: usize, value: A) {
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &A)> {
        self.values
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.as_ref().map(|v| (i, v)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut A)> {
        self.values
            .iter_mut()
            .enumerate()
            .filter_map(|(i, v)| v.as_mut().map(|v| (i, v)))
    }

    pub fn keys(&self) -> impl Iterator<Item = usize> + '_ {
        self.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &A> + '_ {
        self.iter().map(|(_, v)| v)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut A> + '_ {
        self.iter_mut().map(|(_, v)| v)
    }

    pub fn count(&self) -> usize {
        self.iter().count()
    }

    pub fn map<'a, B>(&'a self, mut f: impl FnMut(&'a A) -> B) -> LinearMap<B> {
        let mut map = LinearMap::default();
        for (k, v) in self.iter() {
            map.insert(k, f(v));
        }
        map
    }

    pub fn zip<'a, B>(
        &'a self,
        other: &'a LinearMap<B>,
    ) -> impl Iterator<Item = (usize, &A, &B)> + 'a {
        self.iter().zip(other.iter()).map(|((ka, va), (kb, vb))| {
            assert_eq!(ka, kb);
            (ka, va, vb)
        })
    }
}
