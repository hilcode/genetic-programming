use std::collections::HashMap;

use crate::node::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Num,
    Bool,
}

pub enum Value {
    Num(i64),
    Bool(bool),
}

impl Value {
    pub fn as_num(&self) -> i64 {
        match self {
            Value::Num(number) => *number,
            Value::Bool(_) => panic!("expected a numeric value"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(boolean) => *boolean,
            Value::Num(_) => panic!("expected a boolean value"),
        }
    }
}

pub struct AtomDefinition<Ctx> {
    pub return_type: Type,
    pub param_types: Vec<Type>,
    eval: Box<dyn Fn(&[Value], &Ctx) -> Value>,
}

impl<Ctx> AtomDefinition<Ctx> {
    pub fn new(
        return_type: Type,
        param_types: Vec<Type>,
        eval: impl Fn(&[Value], &Ctx) -> Value + 'static,
    ) -> AtomDefinition<Ctx> {
        AtomDefinition { return_type, param_types, eval: Box::new(eval) }
    }
}

pub struct AtomRegistry<Ctx> {
    atoms: HashMap<String, AtomDefinition<Ctx>>,
    operators_by_type: HashMap<Type, Vec<String>>,
    terminals_by_type: HashMap<Type, Vec<String>>,
    pub root_type: Type,
}

impl<Ctx> AtomRegistry<Ctx> {
    pub fn new(root_type: Type) -> AtomRegistry<Ctx> {
        AtomRegistry {
            atoms: HashMap::new(),
            operators_by_type: HashMap::new(),
            terminals_by_type: HashMap::new(),
            root_type,
        }
    }

    pub fn register(&mut self, name: &str, definition: AtomDefinition<Ctx>) {
        let is_terminal: bool = definition.param_types.is_empty();
        let return_type: Type = definition.return_type;
        self.atoms.insert(name.to_string(), definition);
        if is_terminal {
            self.terminals_by_type.entry(return_type).or_default().push(name.to_string());
        } else {
            self.operators_by_type.entry(return_type).or_default().push(name.to_string());
        }
    }

    pub fn operators_of_type(&self, atom_type: Type) -> &[String] {
        self.operators_by_type.get(&atom_type).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn terminals_of_type(&self, atom_type: Type) -> &[String] {
        self.terminals_by_type.get(&atom_type).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn param_types_of(&self, name: &str) -> &[Type] {
        let definition: &AtomDefinition<Ctx> = self.atoms
            .get(name)
            .unwrap_or_else(|| panic!("unknown atom `{name}`"));
        &definition.param_types
    }

    pub fn type_of(&self, node: &Node) -> Type {
        self.atoms
            .get(&node.name)
            .unwrap_or_else(|| panic!("unknown atom `{}`", node.name))
            .return_type
    }

    pub fn eval(&self, node: &Node, context: &Ctx) -> Value {
        let atom: &AtomDefinition<Ctx> = self.atoms
            .get(&node.name)
            .unwrap_or_else(|| panic!("unknown atom `{}`", node.name));
        let child_values: Vec<Value> = node.children.iter()
            .map(|child| self.eval(child, context))
            .collect();
        (atom.eval)(&child_values, context)
    }

    pub fn indices_of_type(&self, root: &Node, target_type: Type) -> Vec<usize> {
        let mut indices: Vec<usize> = Vec::new();
        self.collect_indices(root, target_type, 0, &mut indices);
        indices
    }

    fn collect_indices(
        &self,
        node: &Node,
        target_type: Type,
        current: usize,
        indices: &mut Vec<usize>,
    ) -> usize {
        if self.type_of(node) == target_type {
            indices.push(current);
        }
        let mut counter: usize = current + 1;
        for child in &node.children {
            counter = self.collect_indices(child, target_type, counter, indices);
        }
        counter
    }
}
