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
