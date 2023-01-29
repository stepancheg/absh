use crate::math::number::Number;

#[derive(Eq, PartialEq, Debug)]
pub struct NumbersSorted<'a, T: Number>(pub &'a [T]);

impl<T: Number> Clone for NumbersSorted<'_, T> {
    fn clone(&self) -> Self {
        NumbersSorted(self.0)
    }
}

impl<T: Number> Copy for NumbersSorted<'_, T> {}

pub enum FilterCond {
    Lt,
    Le,
    Ge,
    Gt,
}

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

    pub fn mean(&self) -> Option<T> {
        if self.len() == 0 {
            None
        } else {
            Some(self.sum().div_usize(self.len()))
        }
    }

    pub fn std(&self) -> Option<T> {
        if self.len() < 2 {
            return None;
        }
        let mean = self.mean()?;
        let s_2 = self
            .0
            .iter()
            .map(|d| (d.as_f64() - mean.as_f64()) * (d.as_f64() - mean.as_f64()))
            .sum::<f64>()
            / ((self.len() - 1) as f64);
        let std_seconds = f64::sqrt(s_2);

        Some(T::from_f64(std_seconds))
    }

    pub fn filter(&self, cond: FilterCond, val: T) -> NumbersSorted<'a, T> {
        match cond {
            FilterCond::Lt => {
                let i = self.0.partition_point(|x| x < &val);
                NumbersSorted(&self.0[..i])
            }
            FilterCond::Le => {
                let i = self.0.partition_point(|x| x <= &val);
                NumbersSorted(&self.0[..i])
            }
            FilterCond::Ge => {
                let i = self.0.partition_point(|x| x < &val);
                NumbersSorted(&self.0[i..])
            }
            FilterCond::Gt => {
                let i = self.0.partition_point(|x| x <= &val);
                NumbersSorted(&self.0[i..])
            }
        }
    }

    fn filter_3_sigma_inner(&self) -> Option<NumbersSorted<'a, T>> {
        let std = self.std()?;
        let mean = self.mean()?;
        let min = mean - std.mul_usize(3);
        let max = mean + std.mul_usize(3);
        let nums = self.filter(FilterCond::Ge, min);
        let nums = nums.filter(FilterCond::Le, max);
        Some(nums)
    }

    pub fn filter_3_sigma(&self) -> NumbersSorted<'a, T> {
        self.filter_3_sigma_inner().unwrap_or(*self)
    }
}

#[cfg(test)]
mod tests {
    use crate::math::sorted::FilterCond;
    use crate::math::sorted::NumbersSorted;

    #[test]
    fn test_filter() {
        let nums = vec![1, 1, 2, 2, 3, 3, 4, 4];
        let nums = NumbersSorted(&nums);
        assert_eq!(NumbersSorted(&[1, 1]), nums.filter(FilterCond::Lt, 2));
        assert_eq!(NumbersSorted(&[1, 1, 2, 2]), nums.filter(FilterCond::Le, 2));
        assert_eq!(
            NumbersSorted(&[2, 2, 3, 3, 4, 4]),
            nums.filter(FilterCond::Ge, 2)
        );
        assert_eq!(NumbersSorted(&[3, 3, 4, 4]), nums.filter(FilterCond::Gt, 2));
    }
}
