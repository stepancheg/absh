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
    pub fn get(&self, key: MeasureKey) -> Option<&A> {
        self.values.get(key.index())
    }
}
