use crate::tree::Expr;
use crate::tree::Type;
use rand::Rng;

/// Subtree crossover: picks a random subexpression in parent1, finds a type-compatible
/// subexpression in parent2, and swaps them to produce two offspring.
/// If parent2 has no subexpression of the matching type, the parents are returned unchanged.
pub fn subtree_crossover(parent1: &Expr, parent2: &Expr, rng: &mut impl Rng) -> (Expr, Expr) {
    let index1: usize = rng.gen_range(0..parent1.size());
    let subtree_type: Type = parent1.get(index1).expr_type();

    let compatible: Vec<usize> = parent2.indices_of_type(subtree_type);
    if compatible.is_empty() {
        return (parent1.clone(), parent2.clone());
    }
    let index2: usize = compatible[rng.gen_range(0..compatible.len())];

    let subtree1: &Expr = parent1.get(index1);
    let subtree2: &Expr = parent2.get(index2);

    let child1: Expr = parent1.replace(index1, subtree2);
    let child2: Expr = parent2.replace(index2, subtree1);

    (child1, child2)
}
