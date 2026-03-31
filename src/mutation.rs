use crate::depth::Depth;
use crate::generate::grow_bool;
use crate::generate::grow_num;
use crate::tree::Expr;
use crate::tree::Type;
use rand::Rng;

/// Subtree mutation: replaces a random subexpression with a newly generated one of the same type.
pub fn subtree_mutation(tree: &Expr, max_depth: Depth, rng: &mut impl Rng) -> Expr {
    let index: usize = rng.gen_range(0..tree.size());
    let new_subtree: Expr = match tree.get(index).expr_type() {
        Type::Num => grow_num(max_depth, rng),
        Type::Bool => grow_bool(max_depth, rng),
    };
    tree.replace(index, &new_subtree)
}
