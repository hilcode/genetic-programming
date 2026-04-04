use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::script::env::Env;


/// A heap-allocated, shareable callable value.
///
/// Using `Rc` (rather than `Box`) allows native functions to be cheaply cloned
/// when a `LispVal::NativeFn` is looked up from the environment.
pub type NativeFn = Rc<dyn Fn(&[LispVal]) -> Result<LispVal, LispError>>;

/// A runtime value in the Lisp scripting layer.
#[derive(Clone)]
pub enum LispVal {
    Num(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Symbol(String),
    List(Vec<LispVal>),
    Lambda {
        params: Vec<String>,
        /// The body is a sequence of expressions; the last one's value is returned.
        body: Vec<LispVal>,
        env: Rc<RefCell<Env>>,
    },
    NativeFn {
        name: String,
        func: NativeFn,
    },
    Nil,
}

impl LispVal {
    /// Returns a short name for the variant, used in error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            LispVal::Num(_) => "Num",
            LispVal::Float(_) => "Float",
            LispVal::Bool(_) => "Bool",
            LispVal::Str(_) => "Str",
            LispVal::Symbol(_) => "Symbol",
            LispVal::List(_) => "List",
            LispVal::Lambda { .. } => "Lambda",
            LispVal::NativeFn { .. } => "NativeFn",
            LispVal::Nil => "Nil",
        }
    }

    pub fn as_num(&self) -> Result<i64, LispError> {
        match self {
            LispVal::Num(number) => Ok(*number),
            other => Err(LispError::type_mismatch("Num", other)),
        }
    }

    /// Returns the value as `f64`, coercing `Num` to `Float`.
    pub fn as_float(&self) -> Result<f64, LispError> {
        match self {
            LispVal::Float(number) => Ok(*number),
            LispVal::Num(number) => Ok(*number as f64),
            other => Err(LispError::type_mismatch("Float", other)),
        }
    }

    pub fn as_bool(&self) -> Result<bool, LispError> {
        match self {
            LispVal::Bool(boolean) => Ok(*boolean),
            other => Err(LispError::type_mismatch("Bool", other)),
        }
    }

    pub fn as_str(&self) -> Result<&str, LispError> {
        match self {
            LispVal::Str(string) => Ok(string.as_str()),
            other => Err(LispError::type_mismatch("Str", other)),
        }
    }

    pub fn as_symbol(&self) -> Result<&str, LispError> {
        match self {
            LispVal::Symbol(name) => Ok(name.as_str()),
            other => Err(LispError::type_mismatch("Symbol", other)),
        }
    }

    pub fn as_list(&self) -> Result<&[LispVal], LispError> {
        match self {
            LispVal::List(elements) => Ok(elements.as_slice()),
            other => Err(LispError::type_mismatch("List", other)),
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            LispVal::Nil => true,
            LispVal::List(elements) => elements.is_empty(),
            _ => false,
        }
    }
}

impl fmt::Display for LispVal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LispVal::Num(number) => write!(formatter, "{number}"),
            LispVal::Float(number) => write!(formatter, "{number}"),
            LispVal::Bool(true) => write!(formatter, "true"),
            LispVal::Bool(false) => write!(formatter, "false"),
            LispVal::Str(string) => write!(formatter, "\"{string}\""),
            LispVal::Symbol(name) => write!(formatter, "{name}"),
            LispVal::List(elements) => {
                write!(formatter, "(")?;
                for (index, element) in elements.iter().enumerate() {
                    if index > 0 {
                        write!(formatter, " ")?;
                    }
                    write!(formatter, "{element}")?;
                }
                write!(formatter, ")")
            }
            LispVal::Lambda { params, .. } => {
                write!(formatter, "(lambda ({}) ...)", params.join(" "))
            }
            LispVal::NativeFn { name, .. } => write!(formatter, "<builtin:{name}>"),
            LispVal::Nil => write!(formatter, "nil"),
        }
    }
}

impl fmt::Debug for LispVal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

/// Errors that can occur during parsing or evaluation.
#[derive(Debug, Clone)]
pub enum LispError {
    Parse(String),
    Eval(String),
    Arity {
        name: String,
        expected: ArityExpect,
        got: usize,
    },
    Unbound(String),
}

/// Describes the arity requirement for an arity error message.
#[derive(Debug, Clone)]
pub enum ArityExpect {
    Exactly(usize),
    AtLeast(usize),
}

impl LispError {
    pub fn type_mismatch(expected: &str, got: &LispVal) -> LispError {
        LispError::Eval(format!("type error: expected {expected}, got {}", got.type_name()))
    }

    pub fn arity(name: &str, expected: usize, got: usize) -> LispError {
        LispError::Arity {
            name: name.to_string(),
            expected: ArityExpect::Exactly(expected),
            got,
        }
    }

    pub fn arity_at_least(name: &str, min: usize, got: usize) -> LispError {
        LispError::Arity {
            name: name.to_string(),
            expected: ArityExpect::AtLeast(min),
            got,
        }
    }
}

impl fmt::Display for LispError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LispError::Parse(message) => write!(formatter, "parse error: {message}"),
            LispError::Eval(message) => write!(formatter, "eval error: {message}"),
            LispError::Arity { name, expected, got } => match expected {
                ArityExpect::Exactly(count) => write!(
                    formatter,
                    "arity error: `{name}` expected {count} argument(s), got {got}"
                ),
                ArityExpect::AtLeast(min) => write!(
                    formatter,
                    "arity error: `{name}` expected at least {min} argument(s), got {got}"
                ),
            },
            LispError::Unbound(name) => write!(formatter, "unbound symbol `{name}`"),
        }
    }
}
