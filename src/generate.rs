use crate::atom::AtomRegistry;
use crate::atom::Type;
use crate::depth::Depth;
use crate::node::Node;
use crate::population::Population;
use crate::population_size::PopulationSize;
use rand::Rng;

/// Builds a tree of the given type where every path from root to leaf has
/// exactly `depth` edges (i.e. all leaves sit at the same level).
pub fn full<Ctx>(
    return_type: Type,
    depth: Depth,
    registry: &AtomRegistry<Ctx>,
    rng: &mut impl Rng,
) -> Node {
    let operators: &[String] = registry.operators_of_type(return_type);
    if depth.is_zero() || operators.is_empty() {
        return random_terminal(return_type, registry, rng);
    }
    let name: String = operators[rng.gen_range(0..operators.len())].clone();
    let param_types: Vec<Type> = registry.param_types_of(&name).to_vec();
    let children: Vec<Node> = param_types.iter()
        .map(|&child_type| full(child_type, depth.decrement(), registry, rng))
        .collect();
    Node::branch(name, children)
}

/// Builds a tree of the given type where leaves may appear at any level up
/// to `max_depth`.
pub fn grow<Ctx>(
    return_type: Type,
    max_depth: Depth,
    registry: &AtomRegistry<Ctx>,
    rng: &mut impl Rng,
) -> Node {
    let operators: &[String] = registry.operators_of_type(return_type);
    if max_depth.is_zero() || operators.is_empty() || rng.gen_bool(0.5) {
        return random_terminal(return_type, registry, rng);
    }
    let name: String = operators[rng.gen_range(0..operators.len())].clone();
    let param_types: Vec<Type> = registry.param_types_of(&name).to_vec();
    let children: Vec<Node> = param_types.iter()
        .map(|&child_type| grow(child_type, max_depth.decrement(), registry, rng))
        .collect();
    Node::branch(name, children)
}

fn random_terminal<Ctx>(
    return_type: Type,
    registry: &AtomRegistry<Ctx>,
    rng: &mut impl Rng,
) -> Node {
    let terminals: &[String] = registry.terminals_of_type(return_type);
    assert!(!terminals.is_empty(), "no terminals of type {return_type:?} registered");
    Node::leaf(&terminals[rng.gen_range(0..terminals.len())])
}

/// Standard GP population initialisation: equal mix of `full` and `grow`
/// trees across depths in `[min_depth, max_depth]`.
pub fn ramped_half_and_half<Ctx>(
    pop_size: PopulationSize,
    min_depth: Depth,
    max_depth: Depth,
    registry: &AtomRegistry<Ctx>,
    rng: &mut impl Rng,
) -> Population {
    pop_size.indices()
        .map(|index| {
            let depth: Depth = Depth::for_index(min_depth, max_depth, index);
            if index % 2 == 0 {
                full(registry.root_type, depth, registry, rng)
            } else {
                grow(registry.root_type, depth, registry, rng)
            }
        })
        .collect()
}
