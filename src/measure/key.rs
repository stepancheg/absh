pub enum MeasureKey {
    WallTime,
    MaxRss,
}

impl MeasureKey {
    pub fn index(&self) -> usize {
        match self {
            MeasureKey::WallTime => 0,
            MeasureKey::MaxRss => 1,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => MeasureKey::WallTime,
            1 => MeasureKey::MaxRss,
            _ => panic!("invalid index"),
        }
    }
}
