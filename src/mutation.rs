use crate::depth::Depth;
use crate::generate::grow;
use crate::tree::Node;
use rand::Rng;

/// Subtree mutation: replaces a random subtree with a newly generated one.
pub fn subtree_mutation(tree: &Node, max_depth: Depth, rng: &mut impl Rng) -> Node {
    let index: usize = rng.gen_range(0..tree.size());
    let new_subtree: Node = grow(max_depth, rng);
    tree.replace(index, &new_subtree)
}
