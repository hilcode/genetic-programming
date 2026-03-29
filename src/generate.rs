use crate::depth::Depth;
use crate::population::Population;
use crate::population_size::PopulationSize;
use crate::tree::Node;
use crate::tree::Op;
use rand::Rng;

const CONST_MIN: i64 = -10;
const CONST_MAX: i64 = 10;

fn random_op(rng: &mut impl Rng) -> Op {
    match rng.gen_range(0..3u8) {
        0 => Op::Add,
        1 => Op::Sub,
        _ => Op::Mul,
    }
}

fn random_const(rng: &mut impl Rng) -> Node {
    Node::Const(rng.gen_range(CONST_MIN..=CONST_MAX))
}

/// Builds a tree where every leaf is at exactly `depth` levels below the root.
pub fn full(depth: Depth, rng: &mut impl Rng) -> Node {
    if depth.is_zero() {
        random_const(rng)
    } else {
        Node::BinOp {
            op: random_op(rng),
            left: Box::new(full(depth.decrement(), rng)),
            right: Box::new(full(depth.decrement(), rng)),
        }
    }
}

/// Builds a tree where leaves may appear at any level up to `max_depth`.
pub fn grow(max_depth: Depth, rng: &mut impl Rng) -> Node {
    if max_depth.is_zero() || rng.gen_bool(0.5) {
        random_const(rng)
    } else {
        Node::BinOp {
            op: random_op(rng),
            left: Box::new(grow(max_depth.decrement(), rng)),
            right: Box::new(grow(max_depth.decrement(), rng)),
        }
    }
}

/// Standard GP population initialisation: equal mix of `full` and `grow` across
/// depths in `[min_depth, max_depth]`.
pub fn ramped_half_and_half(
    pop_size: PopulationSize,
    min_depth: Depth,
    max_depth: Depth,
    rng: &mut impl Rng,
) -> Population {
    pop_size.indices()
        .map(|index| {
            let depth = Depth::for_index(min_depth, max_depth, index);
            if index % 2 == 0 {
                full(depth, rng)
            } else {
                grow(depth, rng)
            }
        })
        .collect()
}
