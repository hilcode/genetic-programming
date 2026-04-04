mod builtins;
pub mod domain;
mod env;
mod eval;
mod reader;
mod value;

pub use domain::node_to_lisp_val;
pub use domain::LoadedDomain;
pub use value::LispError;
pub use value::LispVal;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use env::Env;

/// An interpreter instance with its own global environment.
///
/// Multiple `ScriptEngine`s are fully independent — they share no state.
pub struct ScriptEngine {
    env: Rc<RefCell<Env>>,
}

impl ScriptEngine {
    /// Creates a new engine with all standard builtins pre-registered.
    pub fn new() -> ScriptEngine {
        let env: Rc<RefCell<Env>> = Rc::new(RefCell::new(Env::new()));
        builtins::register_builtins(&env);
        ScriptEngine { env }
    }

    /// Parses and evaluates all top-level expressions in `input`.
    /// Returns the result of every expression in order.
    pub fn run_str(&self, input: &str) -> Result<Vec<LispVal>, LispError> {
        let expressions: Vec<LispVal> = reader::read_all(input)?;
        expressions
            .iter()
            .map(|expression| eval::eval(expression, &self.env))
            .collect()
    }

    /// Reads the file at `path` and evaluates all its top-level expressions.
    pub fn run_file(&self, path: &Path) -> Result<Vec<LispVal>, LispError> {
        let contents: String = std::fs::read_to_string(path).map_err(|error| {
            LispError::Eval(format!("failed to read {}: {error}", path.display()))
        })?;
        self.run_str(&contents)
    }

    /// Loads a domain from a script file.
    ///
    /// Registers `terminal`, `operator`, and `fitness` forms, evaluates the file,
    /// then builds and returns the `LoadedDomain`. Also registers `eval-tree` into
    /// this engine's environment so fitness lambdas can call it.
    pub fn load_domain_file(&self, path: &Path) -> Result<LoadedDomain, LispError> {
        domain::register_domain_forms(&self.env);
        let results: Vec<LispVal> = self.run_file(path)?;
        domain::build_domain(results, &self.env)
    }

}

/// Calls a Lisp function value from Rust code without requiring a `ScriptEngine`.
///
/// The function carries its own captured environment, so no engine instance is needed.
pub fn apply(func: &LispVal, args: Vec<LispVal>) -> Result<LispVal, LispError> {
    eval::apply(func, args)
}
