mod builtins;
pub mod domain;
mod eval;
mod reader;
mod scope;
mod value;

pub use domain::node_to_lisp_val;
pub use domain::LoadedDomain;
pub use value::LispError;
pub use value::LispVal;

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use scope::Scope;

/// An interpreter instance with its own global environment.
///
/// Multiple `ScriptEngine`s are fully independent — they share no state.
pub struct ScriptEngine {
    env: Rc<RefCell<Scope>>,
}

impl ScriptEngine {
    /// Creates a new engine with all standard builtins pre-registered.
    pub fn new() -> ScriptEngine {
        let bindings = builtins::register_builtins();
        let scope: Scope = Scope::from_bindings(bindings);
        let env: Rc<RefCell<Scope>> = Rc::new(RefCell::new(scope));
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
        let domain_bindings = domain::register_domain_forms();
        let domain_scope: Scope = Scope::child_from_bindings(Rc::clone(&self.env), domain_bindings);
        let domain_env: Rc<RefCell<Scope>> = Rc::new(RefCell::new(domain_scope));

        // Evaluate default simplifications
        let default_exprs: Vec<LispVal> = reader::read_all(include_str!("default_simplifications.lisp"))?;
        let default_results: Vec<LispVal> = default_exprs
            .iter()
            .map(|expr| eval::eval(expr, &domain_env))
            .collect::<Result<Vec<_>, _>>()?;

        // Evaluate user domain script
        let user_contents: String = std::fs::read_to_string(path).map_err(|error| {
            LispError::Eval(format!("failed to read {}: {error}", path.display()))
        })?;
        let user_exprs: Vec<LispVal> = reader::read_all(&user_contents)?;
        let user_results: Vec<LispVal> = user_exprs
            .iter()
            .map(|expr| eval::eval(expr, &domain_env))
            .collect::<Result<Vec<_>, _>>()?;

        domain::build_domain([default_results, user_results].concat(), &domain_env)
    }

}

/// Calls a Lisp function value from Rust code without requiring a `ScriptEngine`.
///
/// The function carries its own captured environment, so no engine instance is needed.
pub fn apply(func: &LispVal, args: Vec<LispVal>) -> Result<LispVal, LispError> {
    eval::apply(func, args)
}
