use std::ops::Index;

use crate::population_size::PopulationSize;
use crate::tree::Expr;
use rand::Rng;

pub struct Population(Vec<Expr>);

impl Population {
    pub fn with_capacity(size: PopulationSize) -> Self {
        Population(size.new_vec())
    }

    pub fn push(&mut self, expr: Expr) {
        self.0.push(expr);
    }

    pub fn size(&self) -> PopulationSize {
        PopulationSize::new(self.0.len())
    }

    pub fn random_index(&self, rng: &mut impl Rng) -> usize {
        rng.gen_range(self.size().indices())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Expr> {
        self.0.iter()
    }
}

impl Index<usize> for Population {
    type Output = Expr;

    fn index(&self, index: usize) -> &Expr {
        &self.0[index]
    }
}

impl FromIterator<Expr> for Population {
    fn from_iter<Iter: IntoIterator<Item = Expr>>(iter: Iter) -> Self {
        Population(iter.into_iter().collect())
    }
}
