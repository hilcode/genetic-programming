use std::collections::HashMap;
use std::fmt;

use crate::node::Node;

pub enum PatternNode {
    Var(String),
    Literal(String, Vec<PatternNode>),
}

pub struct SimplificationRule {
    pub name: String,
    pattern: PatternNode,
    template: PatternNode,
}

impl SimplificationRule {
    pub fn new(name: String, pattern: PatternNode, template: PatternNode) -> SimplificationRule {
        SimplificationRule { name, pattern, template }
    }
}

impl fmt::Display for PatternNode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternNode::Var(var_name) => write!(formatter, "{var_name}"),
            PatternNode::Literal(name, children) if children.is_empty() => {
                write!(formatter, "{name}")
            }
            PatternNode::Literal(name, children) => {
                write!(formatter, "({name}")?;
                for child in children {
                    write!(formatter, " {child}")?;
                }
                write!(formatter, ")")
            }
        }
    }
}

impl SimplificationRule {
    pub fn display_aligned(&self, name_width: usize) -> String {
        let label: String = format!("{}:", self.name);
        format!("{label:<name_width$} {} → {}", self.pattern, self.template)
    }
}

impl fmt::Display for SimplificationRule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {} → {}", self.name, self.pattern, self.template)
    }
}

/// Applies one bottom-up simplification pass to `node`.
///
/// Children are simplified first, then each rule is tried in order at the resulting node.
/// The first matching rule fires; unmatched nodes are returned unchanged.
pub fn simplify_tree(node: &Node, rules: &[SimplificationRule]) -> Node {
    let simplified_children: Vec<Node> = node.children.iter()
        .map(|child| simplify_tree(child, rules))
        .collect();
    let candidate: Node = if simplified_children.is_empty() {
        node.clone()
    } else {
        Node::branch(node.name.clone(), simplified_children)
    };
    for rule in rules {
        let mut bindings: HashMap<String, Node> = HashMap::new();
        if match_pattern(&rule.pattern, &candidate, &mut bindings) {
            return instantiate(&rule.template, &bindings);
        }
    }
    candidate
}

fn match_pattern(
    pattern: &PatternNode,
    node: &Node,
    bindings: &mut HashMap<String, Node>,
) -> bool {
    match pattern {
        PatternNode::Var(var_name) => {
            if let Some(existing) = bindings.get(var_name) {
                nodes_equal(existing, node)
            } else {
                bindings.insert(var_name.clone(), node.clone());
                true
            }
        }
        PatternNode::Literal(name, children) => {
            if name != &node.name || children.len() != node.children.len() {
                return false;
            }
            for (child_pattern, child_node) in children.iter().zip(node.children.iter()) {
                if !match_pattern(child_pattern, child_node, bindings) {
                    return false;
                }
            }
            true
        }
    }
}

fn instantiate(template: &PatternNode, bindings: &HashMap<String, Node>) -> Node {
    match template {
        PatternNode::Var(var_name) => bindings[var_name].clone(),
        PatternNode::Literal(name, children) => Node::branch(
            name.clone(),
            children.iter().map(|child| instantiate(child, bindings)).collect(),
        ),
    }
}

fn nodes_equal(left: &Node, right: &Node) -> bool {
    left.name == right.name
        && left.children.len() == right.children.len()
        && left.children
            .iter()
            .zip(right.children.iter())
            .all(|(left_child, right_child)| nodes_equal(left_child, right_child))
}
