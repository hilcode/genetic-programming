mod crossover;
mod depth;
mod engine;
mod generate;
mod mutation;
mod population;
mod population_size;
mod probability;
mod selection;
mod tree;

use engine::GpConfig;
use engine::GpEngine;
use tree::Node;

fn main() {
    // Example: find an expression that evaluates as close to 42 as possible.
    let config = GpConfig {
        seed: Some(0),
        ..GpConfig::default()
    };

    let engine = GpEngine::new(config, |tree: &Node| {
        let value: i64 = tree.eval();
        let diff: i64 = (value - 42).abs();
        // Fitness is inversely proportional to distance from 42.
        // Returns 0.0 when diff is large, approaches infinity as diff -> 0.
        1.0 / (1.0 + diff as f64)
    });

    let best: Node = engine.run();
    println!("\nBest expression: {}", best);
    println!("Evaluates to:    {}", best.eval());
}
