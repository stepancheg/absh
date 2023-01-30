use crate::experiment_name::ExperimentName;

/// Map from experiment name.
pub struct ExperimentMap<A> {
    values: Vec<Option<A>>,
}

impl<A> Default for ExperimentMap<A> {
    fn default() -> ExperimentMap<A> {
        ExperimentMap { values: Vec::new() }
    }
}

impl<A> ExperimentMap<A> {
    pub fn get(&self, exp: ExperimentName) -> Option<&A> {
        self.values.get(exp.index()).and_then(|v| v.as_ref())
    }

    pub fn get_mut(&mut self, exp: ExperimentName) -> Option<&mut A> {
        self.values.get_mut(exp.index()).and_then(|v| v.as_mut())
    }

    pub fn insert(&mut self, exp: ExperimentName, value: A) {
        let index = exp.index();
        if index >= self.values.len() {
            self.values.resize_with(index + 1, || None);
        }
        self.values[index] = Some(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (ExperimentName, &A)> {
        self.values
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.as_ref().map(|v| (ExperimentName::from_index(i), v)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (ExperimentName, &mut A)> {
        self.values
            .iter_mut()
            .enumerate()
            .filter_map(|(i, v)| v.as_mut().map(|v| (ExperimentName::from_index(i), v)))
    }

    pub fn keys(&self) -> impl Iterator<Item = ExperimentName> + '_ {
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

    pub fn map<'a, B>(&'a self, mut f: impl FnMut(&'a A) -> B) -> ExperimentMap<B> {
        let mut map = ExperimentMap::default();
        for (k, v) in self.iter() {
            map.insert(k, f(v));
        }
        map
    }

    pub fn zip<'a, B>(
        &'a self,
        other: &'a ExperimentMap<B>,
    ) -> impl Iterator<Item = (ExperimentName, &A, &B)> + 'a {
        self.iter().zip(other.iter()).map(|((ka, va), (kb, vb))| {
            assert_eq!(ka, kb);
            (ka, va, vb)
        })
    }
}
