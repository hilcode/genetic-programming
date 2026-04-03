use crate::atom::AtomRegistry;
use crate::node::Node;
use rand::Rng;

/// Subtree crossover: picks a random node in `parent1`, finds a type-compatible
/// node in `parent2`, and swaps their subtrees to produce two offspring.
/// If `parent2` has no node of the matching type, both parents are returned unchanged.
pub fn subtree_crossover<Ctx>(
    parent1: &Node,
    parent2: &Node,
    registry: &AtomRegistry<Ctx>,
    rng: &mut impl Rng,
) -> (Node, Node) {
    let index1: usize = rng.gen_range(0..parent1.size());
    let subtree_type = registry.type_of(parent1.get(index1));

    let compatible: Vec<usize> = registry.indices_of_type(parent2, subtree_type);
    if compatible.is_empty() {
        return (parent1.clone(), parent2.clone());
    }
    let index2: usize = compatible[rng.gen_range(0..compatible.len())];

    let subtree1: &Node = parent1.get(index1);
    let subtree2: &Node = parent2.get(index2);

    let child1: Node = parent1.replace(index1, subtree2);
    let child2: Node = parent2.replace(index2, subtree1);

    (child1, child2)
}
