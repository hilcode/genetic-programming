//! GP Engine - a genetic programming framework

// Re-export public API items explicitly
mod atom;
pub use atom::AtomRegistry;

mod cli;
pub use cli::Cli;

mod config;
pub use config::GpConfig;
pub use config::RawConfig;

mod engine;
pub use engine::GpEngine;

mod node;
pub use node::Node;

mod script;
pub use script::apply;
pub use script::node_to_lisp_val;
pub use script::LispVal;
pub use script::ScriptEngine;

mod simplification;
pub use simplification::SimplificationRule;

// Test support - not part of stable API
pub mod test_support {
    //! Utilities and types for integration testing.
    //! Not part of the stable public API.

    pub use crate::atom::Type;
    pub use crate::depth::Depth;
    pub use crate::population_size::PopulationSize;
    pub use crate::script::LispError;
    pub use crate::script::LoadedDomain;
    pub use crate::simplification::simplify_tree;

    /// Extract the inner value from PopulationSize for testing.
    pub fn population_size_value(pop_size: &PopulationSize) -> usize {
        pop_size.value()
    }

    /// Extract the inner value from Depth for testing.
    pub fn depth_value(depth: &Depth) -> usize {
        depth.value()
    }
}

// Internal modules - implementation details
mod crossover;
mod depth;
mod generate;
mod mutation;
mod population;
mod population_size;
mod probability;
mod selection;
mod top_fraction;
