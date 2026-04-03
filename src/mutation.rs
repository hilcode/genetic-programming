use crate::atom::AtomRegistry;
use crate::depth::Depth;
use crate::generate::grow;
use crate::node::Node;
use rand::Rng;

/// Subtree mutation: replaces a random node with a freshly generated subtree
/// of the same type.
pub fn subtree_mutation<Ctx>(
    tree: &Node,
    registry: &AtomRegistry<Ctx>,
    max_depth: Depth,
    rng: &mut impl Rng,
) -> Node {
    let index: usize = rng.gen_range(0..tree.size());
    let subtree_type = registry.type_of(tree.get(index));
    let new_subtree: Node = grow(subtree_type, max_depth, registry, rng);
    tree.replace(index, &new_subtree)
}
