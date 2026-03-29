use std::ops::Range;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PopulationSize(usize);

impl PopulationSize {
    pub fn new(value: usize) -> PopulationSize {
        PopulationSize(value)
    }

    pub fn indices(self) -> Range<usize> {
        0..self.0
    }

    pub fn new_vec<T>(self) -> Vec<T> {
        Vec::with_capacity(self.0)
    }
}
