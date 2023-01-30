use std::ops::Index;
use std::ops::IndexMut;

use crate::experiment_name::ExperimentName;
use crate::linear_map::LinearMap;

/// Map from experiment name.
pub struct ExperimentMap<A> {
    values: LinearMap<A>,
}

impl<A> Default for ExperimentMap<A> {
    fn default() -> ExperimentMap<A> {
        ExperimentMap {
            values: LinearMap::default(),
        }
    }
}

impl<A> ExperimentMap<A> {
    pub fn get(&self, exp: ExperimentName) -> Option<&A> {
        self.values.get(exp.index())
    }

    pub fn get_mut(&mut self, exp: ExperimentName) -> Option<&mut A> {
        self.values.get_mut(exp.index())
    }

    pub fn insert(&mut self, exp: ExperimentName, value: A) {
        self.values.insert(exp.index(), value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (ExperimentName, &A)> {
        self.values
            .iter()
            .map(|(i, v)| (ExperimentName::from_index(i), v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (ExperimentName, &mut A)> {
        self.values
            .iter_mut()
            .map(|(i, v)| (ExperimentName::from_index(i), v))
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
        self.values.count()
    }

    pub fn map<'a, B>(&'a self, f: impl FnMut(&'a A) -> B) -> ExperimentMap<B> {
        ExperimentMap {
            values: self.values.map(f),
        }
    }

    pub fn zip<'a, B>(
        &'a self,
        other: &'a ExperimentMap<B>,
    ) -> impl Iterator<Item = (ExperimentName, &A, &B)> + 'a {
        self.values
            .zip(&other.values)
            .map(|(k, a, b)| (ExperimentName::from_index(k), a, b))
    }
}

impl<A> Index<ExperimentName> for ExperimentMap<A> {
    type Output = A;

    fn index(&self, exp: ExperimentName) -> &A {
        self.get(exp).unwrap()
    }
}

impl<A> IndexMut<ExperimentName> for ExperimentMap<A> {
    fn index_mut(&mut self, exp: ExperimentName) -> &mut A {
        self.get_mut(exp).unwrap()
    }
}
