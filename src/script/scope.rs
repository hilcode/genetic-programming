use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::script::value::LispVal;

pub struct Scope {
    bindings: HashMap<String, LispVal>,
    parent: Option<Rc<RefCell<Scope>>>,
}

impl Scope {
    pub fn new() -> Scope {
        Scope { bindings: HashMap::new(), parent: None }
    }

    pub fn new_child(parent: Rc<RefCell<Scope>>) -> Scope {
        Scope { bindings: HashMap::new(), parent: Some(parent) }
    }

    /// Creates a root scope from a HashMap of bindings.
    pub fn from_bindings(bindings: HashMap<String, LispVal>) -> Scope {
        Scope { bindings, parent: None }
    }

    /// Creates a child scope from a parent and a HashMap of bindings.
    pub fn child_from_bindings(
        parent: Rc<RefCell<Scope>>,
        bindings: HashMap<String, LispVal>,
    ) -> Scope {
        Scope { bindings, parent: Some(parent) }
    }

    /// Defines a new binding in this scope (shadows any outer binding with the same name).
    pub fn define(&mut self, name: String, value: LispVal) {
        self.bindings.insert(name, value);
    }

    /// Looks up a name, walking up the scope chain until a binding is found.
    pub fn lookup(&self, name: &str) -> Option<LispVal> {
        if let Some(value) = self.bindings.get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().lookup(name)
        } else {
            None
        }
    }
}
