use crate::math::number::Number;

pub struct NumbersSorted<'a, T: Number>(pub &'a [T]);

impl<'a, T: Number> NumbersSorted<'a, T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn min(&self) -> Option<T> {
        self.0.first().cloned()
    }

    pub fn max(&self) -> Option<T> {
        self.0.last().cloned()
    }

    pub fn med(&self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            if self.len() % 2 == 0 {
                let xy: T = self.0[self.len() / 2 - 1].clone() + self.0[self.len() / 2].clone();
                Some(xy.div_usize(2))
            } else {
                Some(self.0[self.len() / 2].clone())
            }
        }
    }

    pub fn sum(&self) -> T {
        self.0.iter().cloned().sum()
    }
}
