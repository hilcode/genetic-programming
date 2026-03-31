use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Num,
    Bool,
}

pub enum Value {
    Num(i64),
    Bool(bool),
}

impl Value {
    pub fn as_num(self) -> i64 {
        match self {
            Value::Num(n) => n,
            Value::Bool(_) => panic!("expected a numeric value"),
        }
    }

    pub fn as_bool(self) -> bool {
        match self {
            Value::Bool(b) => b,
            Value::Num(_) => panic!("expected a boolean value"),
        }
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub enum BoolOp {
    And,
    Or,
    Xor,
}

impl fmt::Display for BoolOp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoolOp::And => write!(formatter, "AND"),
            BoolOp::Or => write!(formatter, "OR"),
            BoolOp::Xor => write!(formatter, "XOR"),
        }
    }
}

pub struct Context {
    pub target: i64,
    pub flag: bool,
}

#[derive(Debug, Clone)]
pub enum Expr {
    // Numeric terminals
    Const(i64),
    Target,
    // Boolean terminals
    True,
    False,
    Flag,
    // Numeric operators
    BinOp { op: Op, left: Box<Expr>, right: Box<Expr> },
    // Boolean operators
    Not { operand: Box<Expr> },
    BoolBinOp { op: BoolOp, left: Box<Expr>, right: Box<Expr> },
    // Control flow (numeric result)
    If { condition: Box<Expr>, true_branch: Box<Expr>, false_branch: Box<Expr> },
}

impl Expr {
    pub fn expr_type(&self) -> Type {
        match self {
            Expr::Const(_) | Expr::Target | Expr::BinOp { .. } | Expr::If { .. } => Type::Num,
            Expr::True | Expr::False | Expr::Flag | Expr::Not { .. } | Expr::BoolBinOp { .. } => Type::Bool,
        }
    }

    pub fn eval(&self, context: &Context) -> Value {
        match self {
            Expr::Const(n) => Value::Num(*n),
            Expr::Target => Value::Num(context.target),
            Expr::True => Value::Bool(true),
            Expr::False => Value::Bool(false),
            Expr::Flag => Value::Bool(context.flag),
            Expr::BinOp { op, left, right } => {
                let left_val: i64 = left.eval(context).as_num();
                let right_val: i64 = right.eval(context).as_num();
                Value::Num(match op {
                    Op::Add => left_val.saturating_add(right_val),
                    Op::Sub => left_val.saturating_sub(right_val),
                    Op::Mul => left_val.saturating_mul(right_val),
                })
            }
            Expr::Not { operand } => Value::Bool(!operand.eval(context).as_bool()),
            Expr::BoolBinOp { op, left, right } => {
                let left_val: bool = left.eval(context).as_bool();
                let right_val: bool = right.eval(context).as_bool();
                Value::Bool(match op {
                    BoolOp::And => left_val && right_val,
                    BoolOp::Or => left_val || right_val,
                    BoolOp::Xor => left_val ^ right_val,
                })
            }
            Expr::If { condition, true_branch, false_branch } => {
                if condition.eval(context).as_bool() {
                    true_branch.eval(context)
                } else {
                    false_branch.eval(context)
                }
            }
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Expr::Const(_) | Expr::Target | Expr::True | Expr::False | Expr::Flag => 1,
            Expr::Not { operand } => 1 + operand.size(),
            Expr::BinOp { left, right, .. } | Expr::BoolBinOp { left, right, .. } => {
                1 + left.size() + right.size()
            }
            Expr::If { condition, true_branch, false_branch } => {
                1 + condition.size() + true_branch.size() + false_branch.size()
            }
        }
    }

    /// Returns the subexpression at DFS index `index` (root = 0).
    pub fn get(&self, index: usize) -> &Expr {
        let (node, _): (Option<&Expr>, usize) = self.get_internal(index, 0);
        node.expect("index out of bounds")
    }

    fn get_internal(&self, target_index: usize, current: usize) -> (Option<&Expr>, usize) {
        if current == target_index {
            return (Some(self), current + self.size());
        }
        match self {
            Expr::Const(_) | Expr::Target | Expr::True | Expr::False | Expr::Flag => {
                (None, current + 1)
            }
            Expr::Not { operand } => operand.get_internal(target_index, current + 1),
            Expr::BinOp { left, right, .. } | Expr::BoolBinOp { left, right, .. } => {
                let (found, after_left): (Option<&Expr>, usize) = left.get_internal(target_index, current + 1);
                if found.is_some() {
                    return (found, after_left);
                }
                right.get_internal(target_index, after_left)
            }
            Expr::If { condition, true_branch, false_branch } => {
                let (found, after_condition): (Option<&Expr>, usize) = condition.get_internal(target_index, current + 1);
                if found.is_some() {
                    return (found, after_condition);
                }
                let (found, after_true): (Option<&Expr>, usize) = true_branch.get_internal(target_index, after_condition);
                if found.is_some() {
                    return (found, after_true);
                }
                false_branch.get_internal(target_index, after_true)
            }
        }
    }

    /// Returns a copy of this tree with the subexpression at DFS index `index` replaced by `replacement`.
    pub fn replace(&self, index: usize, replacement: &Expr) -> Expr {
        let (node, _): (Expr, usize) = self.replace_internal(index, replacement, 0);
        node
    }

    fn replace_internal(&self, target_index: usize, replacement: &Expr, current: usize) -> (Expr, usize) {
        if current == target_index {
            return (replacement.clone(), current + self.size());
        }
        match self {
            Expr::Const(_) | Expr::Target | Expr::True | Expr::False | Expr::Flag => {
                (self.clone(), current + 1)
            }
            Expr::Not { operand } => {
                let (new_operand, after_operand): (Expr, usize) = operand.replace_internal(target_index, replacement, current + 1);
                (Expr::Not { operand: Box::new(new_operand) }, after_operand)
            }
            Expr::BinOp { op, left, right } => {
                let (new_left, after_left): (Expr, usize) = left.replace_internal(target_index, replacement, current + 1);
                let (new_right, after_right): (Expr, usize) = right.replace_internal(target_index, replacement, after_left);
                (Expr::BinOp { op: op.clone(), left: Box::new(new_left), right: Box::new(new_right) }, after_right)
            }
            Expr::BoolBinOp { op, left, right } => {
                let (new_left, after_left): (Expr, usize) = left.replace_internal(target_index, replacement, current + 1);
                let (new_right, after_right): (Expr, usize) = right.replace_internal(target_index, replacement, after_left);
                (Expr::BoolBinOp { op: op.clone(), left: Box::new(new_left), right: Box::new(new_right) }, after_right)
            }
            Expr::If { condition, true_branch, false_branch } => {
                let (new_condition, after_condition): (Expr, usize) = condition.replace_internal(target_index, replacement, current + 1);
                let (new_true, after_true): (Expr, usize) = true_branch.replace_internal(target_index, replacement, after_condition);
                let (new_false, after_false): (Expr, usize) = false_branch.replace_internal(target_index, replacement, after_true);
                (Expr::If {
                    condition: Box::new(new_condition),
                    true_branch: Box::new(new_true),
                    false_branch: Box::new(new_false),
                }, after_false)
            }
        }
    }

    /// Returns the DFS indices of all subexpressions with the given type.
    pub fn indices_of_type(&self, target_type: Type) -> Vec<usize> {
        let mut indices: Vec<usize> = Vec::new();
        self.collect_indices(target_type, 0, &mut indices);
        indices
    }

    fn collect_indices(&self, target_type: Type, current: usize, indices: &mut Vec<usize>) -> usize {
        if self.expr_type() == target_type {
            indices.push(current);
        }
        match self {
            Expr::Const(_) | Expr::Target | Expr::True | Expr::False | Expr::Flag => current + 1,
            Expr::Not { operand } => operand.collect_indices(target_type, current + 1, indices),
            Expr::BinOp { left, right, .. } | Expr::BoolBinOp { left, right, .. } => {
                let after_left = left.collect_indices(target_type, current + 1, indices);
                right.collect_indices(target_type, after_left, indices)
            }
            Expr::If { condition, true_branch, false_branch } => {
                let after_condition = condition.collect_indices(target_type, current + 1, indices);
                let after_true = true_branch.collect_indices(target_type, after_condition, indices);
                false_branch.collect_indices(target_type, after_true, indices)
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Const(value) => write!(formatter, "{}", value),
            Expr::Target => write!(formatter, "TARGET"),
            Expr::True => write!(formatter, "TRUE"),
            Expr::False => write!(formatter, "FALSE"),
            Expr::Flag => write!(formatter, "FLAG"),
            Expr::BinOp { op, left, right } => write!(formatter, "({} {} {})", left, op, right),
            Expr::Not { operand } => write!(formatter, "(NOT {})", operand),
            Expr::BoolBinOp { op, left, right } => write!(formatter, "({} {} {})", op, left, right),
            Expr::If { condition, true_branch, false_branch } => {
                write!(formatter, "(IF {} {} {})", condition, true_branch, false_branch)
            }
        }
    }
}
