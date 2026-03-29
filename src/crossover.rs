use crate::tree::Node;
use rand::Rng;

/// Subtree crossover: picks a random node in each parent and swaps the subtrees.
/// Returns two offspring.
pub fn subtree_crossover(parent1: &Node, parent2: &Node, rng: &mut impl Rng) -> (Node, Node) {
    let index1: usize = rng.gen_range(0..parent1.size());
    let index2: usize = rng.gen_range(0..parent2.size());

    let subtree1: &Node = parent1.get(index1);
    let subtree2: &Node = parent2.get(index2);

    let child1: Node = parent1.replace(index1, subtree2);
    let child2: Node = parent2.replace(index2, subtree1);

    (child1, child2)
}
