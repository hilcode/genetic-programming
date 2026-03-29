use std::ops::Index;

use crate::population_size::PopulationSize;
use crate::tree::Node;
use rand::Rng;

pub struct Population(Vec<Node>);

impl Population {
    pub fn with_capacity(size: PopulationSize) -> Self {
        Population(size.new_vec())
    }

    pub fn push(&mut self, node: Node) {
        self.0.push(node);
    }

    pub fn size(&self) -> PopulationSize {
        PopulationSize::new(self.0.len())
    }

    pub fn random_index(&self, rng: &mut impl Rng) -> usize {
        rng.gen_range(self.size().indices())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Node> {
        self.0.iter()
    }
}

impl Index<usize> for Population {
    type Output = Node;

    fn index(&self, index: usize) -> &Node {
        &self.0[index]
    }
}

impl FromIterator<Node> for Population {
    fn from_iter<Iter: IntoIterator<Item = Node>>(iter: Iter) -> Self {
        Population(iter.into_iter().collect())
    }
}
