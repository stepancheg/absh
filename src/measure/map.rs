use std::ops::Index;
use std::ops::IndexMut;

use crate::linear_map::LinearMap;
use crate::measure::key::MeasureKey;

/// Map measure key to a value.
pub struct MeasureMap<A> {
    values: LinearMap<A>,
}

impl<A> Default for MeasureMap<A> {
    fn default() -> MeasureMap<A> {
        MeasureMap {
            values: LinearMap::default(),
        }
    }
}

impl<A> MeasureMap<A> {
    pub fn new_all_default() -> MeasureMap<A>
    where
        A: Default,
    {
        let mut map = MeasureMap::default();
        for key in MeasureKey::ALL {
            map.insert(*key, A::default());
        }
        map
    }

    pub fn get(&self, key: MeasureKey) -> Option<&A> {
        self.values.get(key.index())
    }

    pub fn get_mut(&mut self, key: MeasureKey) -> Option<&mut A> {
        self.values.get_mut(key.index())
    }

    pub fn insert(&mut self, key: MeasureKey, value: A) {
        self.values.insert(key.index(), value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (MeasureKey, &A)> {
        self.values
            .iter()
            .map(|(i, v)| (MeasureKey::from_index(i), v))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (MeasureKey, &mut A)> {
        self.values
            .iter_mut()
            .map(|(i, v)| (MeasureKey::from_index(i), v))
    }

    pub fn keys(&self) -> impl Iterator<Item = MeasureKey> + '_ {
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
}

impl<A> Index<MeasureKey> for MeasureMap<A> {
    type Output = A;

    fn index(&self, key: MeasureKey) -> &A {
        self.get(key).unwrap()
    }
}

impl<A> IndexMut<MeasureKey> for MeasureMap<A> {
    fn index_mut(&mut self, key: MeasureKey) -> &mut A {
        self.get_mut(key).unwrap()
    }
}
