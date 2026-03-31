mod crossover;
mod depth;
mod engine;
mod generate;
mod mutation;
mod population;
mod population_size;
mod probability;
mod selection;
mod top_fraction;
mod tree;

use engine::GpConfig;
use engine::GpEngine;
use population_size::PopulationSize;
use tree::Context;
use tree::Expr;

fn main() {
    let config = GpConfig {
        seed: Some(0),
        population_size: PopulationSize::new(500),
        generations: 200,
        ..GpConfig::default()
    };

    let engine = GpEngine::new(config, |expr: &Expr| {
        let true_value: i64 = expr.eval(&Context { target: 100, flag: true }).as_num();
        let false_value: i64 = expr.eval(&Context { target: 100, flag: false }).as_num();
        let true_diff: i64 = (true_value - 42).abs();
        let false_diff: i64 = (false_value - 123).abs();
        // Fitness is the product of per-target scores. Summing the diffs would create a
        // flat region for any value in [42, 123], giving the GP no gradient to follow.
        // Multiplying penalises being wrong on either target independently.
        (1.0 / (1.0 + true_diff as f64)) * (1.0 / (1.0 + false_diff as f64))
    });

    let best: Expr = engine.run();
    println!("\nBest expression: {}", best);
    println!("FLAG=true  evaluates to: {} (target 42)",  best.eval(&Context { target: 100, flag: true  }).as_num());
    println!("FLAG=false evaluates to: {} (target 123)", best.eval(&Context { target: 100, flag: false }).as_num());
}
