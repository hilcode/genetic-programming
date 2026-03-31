use crate::depth::Depth;
use crate::population::Population;
use crate::population_size::PopulationSize;
use crate::tree::BoolOp;
use crate::tree::Expr;
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

fn random_bool_op(rng: &mut impl Rng) -> BoolOp {
    match rng.gen_range(0..3u8) {
        0 => BoolOp::And,
        1 => BoolOp::Or,
        _ => BoolOp::Xor,
    }
}

fn random_num_terminal(rng: &mut impl Rng) -> Expr {
    if rng.gen_bool(0.5) {
        Expr::Target
    } else {
        Expr::Const(rng.gen_range(CONST_MIN..=CONST_MAX))
    }
}

fn random_bool_terminal(rng: &mut impl Rng) -> Expr {
    match rng.gen_range(0..3u8) {
        0 => Expr::True,
        1 => Expr::False,
        _ => Expr::Flag,
    }
}

/// Builds a numeric expression tree where every leaf is at exactly `depth` levels below the root.
pub fn full_num(depth: Depth, rng: &mut impl Rng) -> Expr {
    if depth.is_zero() {
        return random_num_terminal(rng);
    }
    if rng.gen_bool(0.25) {
        Expr::If {
            condition: Box::new(grow_bool(Depth::new(2), rng)),
            true_branch: Box::new(full_num(depth.decrement(), rng)),
            false_branch: Box::new(full_num(depth.decrement(), rng)),
        }
    } else {
        Expr::BinOp {
            op: random_op(rng),
            left: Box::new(full_num(depth.decrement(), rng)),
            right: Box::new(full_num(depth.decrement(), rng)),
        }
    }
}

/// Builds a numeric expression tree where leaves may appear at any level up to `max_depth`.
pub fn grow_num(max_depth: Depth, rng: &mut impl Rng) -> Expr {
    if max_depth.is_zero() || rng.gen_bool(0.5) {
        return random_num_terminal(rng);
    }
    if rng.gen_bool(0.25) {
        Expr::If {
            condition: Box::new(grow_bool(Depth::new(2), rng)),
            true_branch: Box::new(grow_num(max_depth.decrement(), rng)),
            false_branch: Box::new(grow_num(max_depth.decrement(), rng)),
        }
    } else {
        Expr::BinOp {
            op: random_op(rng),
            left: Box::new(grow_num(max_depth.decrement(), rng)),
            right: Box::new(grow_num(max_depth.decrement(), rng)),
        }
    }
}

/// Builds a boolean expression tree where leaves may appear at any level up to `max_depth`.
pub fn grow_bool(max_depth: Depth, rng: &mut impl Rng) -> Expr {
    if max_depth.is_zero() || rng.gen_bool(0.5) {
        return random_bool_terminal(rng);
    }
    if rng.gen_bool(0.25) {
        Expr::Not {
            operand: Box::new(grow_bool(max_depth.decrement(), rng)),
        }
    } else {
        Expr::BoolBinOp {
            op: random_bool_op(rng),
            left: Box::new(grow_bool(max_depth.decrement(), rng)),
            right: Box::new(grow_bool(max_depth.decrement(), rng)),
        }
    }
}

/// Standard GP population initialisation: equal mix of `full_num` and `grow_num` across
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
                full_num(depth, rng)
            } else {
                grow_num(depth, rng)
            }
        })
        .collect()
}
