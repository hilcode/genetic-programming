use rand::Rng;

#[derive(Clone, Copy)]
pub struct Probability(usize);

impl Probability {
    /// Creates a new `Probability`. Panics if `value` is greater than 100.
    pub fn new(value: usize) -> Probability {
        assert!(value <= 100, "Probability must be in 0..=100, got {}", value);
        Probability(value)
    }

    /// Returns `true` with the likelihood represented by this probability.
    pub fn occurs(self, rng: &mut impl Rng) -> bool {
        rng.gen_range(0..100) < self.0
    }
}
