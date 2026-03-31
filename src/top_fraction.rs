use crate::population_size::PopulationSize;

#[derive(Clone, Copy)]
pub struct TopFraction(usize);

impl TopFraction {
    /// Creates a new `TopFraction`. Panics if `value` is greater than 100.
    pub fn new(value: usize) -> TopFraction {
        assert!(value <= 100, "TopFraction must be in 0..=100, got {}", value);
        TopFraction(value)
    }

    /// Returns how many individuals correspond to this fraction of `population_size`.
    /// Always returns at least 1.
    pub fn count(self, population_size: PopulationSize) -> usize {
        let total: usize = population_size.indices().len();
        ((total * self.0) / 100).max(1)
    }
}
