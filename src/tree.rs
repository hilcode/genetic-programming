use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
}

impl fmt::Display for Op {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Add => write!(formatter, "+"),
            Op::Sub => write!(formatter, "-"),
            Op::Mul => write!(formatter, "*"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    Const(i64),
    BinOp { op: Op, left: Box<Node>, right: Box<Node> },
}

impl Node {
    pub fn eval(&self) -> i64 {
        match self {
            Node::Const(n) => *n,
            Node::BinOp { op, left, right } => {
                let left_val: i64 = left.eval();
                let right_val: i64 = right.eval();
                match op {
                    Op::Add => left_val.saturating_add(right_val),
                    Op::Sub => left_val.saturating_sub(right_val),
                    Op::Mul => left_val.saturating_mul(right_val),
                }
            }
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Node::Const(_) => 1,
            Node::BinOp { left, right, .. } => 1 + left.size() + right.size(),
        }
    }

    /// Returns the subtree at DFS index `idx` (root = 0).
    pub fn get(&self, idx: usize) -> &Node {
        let (node, _): (Option<&Node>, usize) = self.get_internal(idx, 0);
        node.expect("index out of bounds")
    }

    fn get_internal(&self, target: usize, current: usize) -> (Option<&Node>, usize) {
        if current == target {
            return (Some(self), current + self.size());
        }
        match self {
            Node::Const(_) => (None, current + 1),
            Node::BinOp { left, right, .. } => {
                let (found, after_left): (Option<&Node>, usize) = left.get_internal(target, current + 1);
                if found.is_some() {
                    return (found, after_left);
                }
                right.get_internal(target, after_left)
            }
        }
    }

    /// Returns a copy of this tree with the subtree at DFS index `idx` replaced by `replacement`.
    pub fn replace(&self, idx: usize, replacement: &Node) -> Node {
        let (node, _): (Node, usize) = self.replace_internal(idx, replacement, 0);
        node
    }

    fn replace_internal(&self, target: usize, replacement: &Node, current: usize) -> (Node, usize) {
        if current == target {
            return (replacement.clone(), current + self.size());
        }
        match self {
            Node::Const(_) => (self.clone(), current + 1),
            Node::BinOp { op, left, right } => {
                let (new_left, after_left): (Node, usize) = left.replace_internal(target, replacement, current + 1);
                let (new_right, after_right): (Node, usize) = right.replace_internal(target, replacement, after_left);
                (
                    Node::BinOp {
                        op: op.clone(),
                        left: Box::new(new_left),
                        right: Box::new(new_right),
                    },
                    after_right,
                )
            }
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Const(value) => write!(formatter, "{}", value),
            Node::BinOp { op, left, right } => write!(formatter, "({} {} {})", left, op, right),
        }
    }
}
