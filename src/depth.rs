#[derive(Clone, Copy)]
pub struct Depth(usize);

impl Depth {
    pub fn new(value: usize) -> Depth {
        Depth(value)
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn decrement(self) -> Depth {
        Depth(self.0 - 1)
    }

    /// Given a `min`/`max` range and a population `index`, returns the depth
    /// that index maps to in a ramped initialisation scheme.
    pub fn for_index(min: Depth, max: Depth, index: usize) -> Depth {
        let range: usize = (max.0 - min.0) + 1;
        Depth(min.0 + (index % range))
    }
}
