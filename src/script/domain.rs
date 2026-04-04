use std::cell::RefCell;
use std::rc::Rc;

use crate::atom::AtomDefinition;
use crate::atom::AtomRegistry;
use crate::atom::Type;
use crate::atom::Value;
use crate::node::Node;
use crate::script::env::Env;
use crate::script::value::LispError;
use crate::script::value::LispVal;

use super::eval;

/// Registers `terminal`, `operator`, and `fitness` as pure data-returning forms in `env`.
///
/// Each form returns a tagged `LispVal::List` rather than accumulating state.
/// Collect the top-level results of a script run and pass them to `build_domain`.
pub fn register_domain_forms(env: &Rc<RefCell<Env>>) {
    reg(env, "terminal", |args| {
        if args.len() != 3 {
            return Err(LispError::arity("terminal", 3, args.len()));
        }
        args[0].as_str()?;
        args[1].as_str()?;
        Ok(LispVal::List(vec![
            LispVal::Symbol("__terminal__".to_string()),
            args[0].clone(),
            args[1].clone(),
            args[2].clone(),
        ]))
    });

    reg(env, "operator", |args| {
        if args.len() != 4 {
            return Err(LispError::arity("operator", 4, args.len()));
        }
        args[0].as_str()?;
        args[1].as_str()?;
        args[2].as_list()?;
        Ok(LispVal::List(vec![
            LispVal::Symbol("__operator__".to_string()),
            args[0].clone(),
            args[1].clone(),
            args[2].clone(),
            args[3].clone(),
        ]))
    });

    reg(env, "fitness", |args| {
        if args.len() != 1 {
            return Err(LispError::arity("fitness", 1, args.len()));
        }
        Ok(LispVal::List(vec![
            LispVal::Symbol("__fitness__".to_string()),
            args[0].clone(),
        ]))
    });
}

/// Processes top-level results from a domain script into a `LoadedDomain`.
///
/// Also registers `eval-tree` into `env` so that fitness lambdas can call it.
/// `eval-tree` must be looked up at call time, not at lambda-definition time,
/// so registering it here (after building the registry) is safe.
pub fn build_domain(
    results: Vec<LispVal>,
    env: &Rc<RefCell<Env>>,
) -> Result<LoadedDomain, LispError> {
    let mut root_type: Option<Type> = None;
    let mut terminal_defs: Vec<(String, Type, LispVal)> = Vec::new();
    let mut operator_defs: Vec<(String, Type, Vec<Type>, LispVal)> = Vec::new();
    let mut fitness_fn: Option<LispVal> = None;

    for result in results {
        let LispVal::List(ref elements) = result else { continue };
        let Some(tag) = elements.first().and_then(|value| value.as_symbol().ok()) else {
            continue;
        };
        match tag {
            "__terminal__" => {
                if elements.len() != 4 {
                    return Err(LispError::Eval(
                        "malformed __terminal__ tag (expected 4 elements)".to_string(),
                    ));
                }
                let name: String = elements[1].as_str()?.to_string();
                let return_type: Type = parse_gp_type(elements[2].as_str()?)?;
                let eval_fn: LispVal = elements[3].clone();
                if root_type.is_none() {
                    root_type = Some(return_type);
                }
                terminal_defs.push((name, return_type, eval_fn));
            }
            "__operator__" => {
                if elements.len() != 5 {
                    return Err(LispError::Eval(
                        "malformed __operator__ tag (expected 5 elements)".to_string(),
                    ));
                }
                let name: String = elements[1].as_str()?.to_string();
                let return_type: Type = parse_gp_type(elements[2].as_str()?)?;
                let param_types: Vec<Type> = elements[3]
                    .as_list()?
                    .iter()
                    .map(|type_val| parse_gp_type(type_val.as_str()?))
                    .collect::<Result<Vec<_>, _>>()?;
                let eval_fn: LispVal = elements[4].clone();
                operator_defs.push((name, return_type, param_types, eval_fn));
            }
            "__fitness__" => {
                if elements.len() != 2 {
                    return Err(LispError::Eval(
                        "malformed __fitness__ tag (expected 2 elements)".to_string(),
                    ));
                }
                fitness_fn = Some(elements[1].clone());
            }
            _ => {}
        }
    }

    let root_type: Type = root_type.ok_or_else(|| {
        LispError::Eval("no `terminal` declarations found in domain script".to_string())
    })?;
    let fitness_fn: LispVal = fitness_fn.ok_or_else(|| {
        LispError::Eval("no `fitness` form found in domain script".to_string())
    })?;

    let mut registry: AtomRegistry<LispVal> = AtomRegistry::new(root_type);

    for (name, return_type, eval_fn) in terminal_defs {
        let closure_name: String = name.clone();
        registry.register(
            &name,
            AtomDefinition::new(
                return_type,
                vec![],
                move |_args: &[Value], context: &LispVal| {
                    let result: LispVal =
                        eval::apply(&eval_fn, vec![context.clone()]).unwrap_or_else(|error| {
                            panic!("terminal `{closure_name}` eval failed: {error}")
                        });
                    lisp_to_gp_value(result, return_type)
                },
            ),
        );
    }

    for (name, return_type, param_types, eval_fn) in operator_defs {
        let closure_name: String = name.clone();
        registry.register(
            &name,
            AtomDefinition::new(
                return_type,
                param_types,
                move |args: &[Value], context: &LispVal| {
                    let lisp_args: LispVal =
                        LispVal::List(args.iter().map(gp_to_lisp_value).collect());
                    let result: LispVal =
                        eval::apply(&eval_fn, vec![lisp_args, context.clone()])
                            .unwrap_or_else(|error| {
                                panic!("operator `{closure_name}` eval failed: {error}")
                            });
                    lisp_to_gp_value(result, return_type)
                },
            ),
        );
    }

    let registry: Rc<AtomRegistry<LispVal>> = Rc::new(registry);

    let eval_tree_registry: Rc<AtomRegistry<LispVal>> = Rc::clone(&registry);
    reg(env, "eval-tree", move |args: &[LispVal]| {
        if args.len() != 2 {
            return Err(LispError::arity("eval-tree", 2, args.len()));
        }
        let node: Node = lisp_val_to_node(&args[0])?;
        let result: Value = eval_tree_registry.eval(&node, &args[1]);
        Ok(gp_to_lisp_value(&result))
    });

    Ok(LoadedDomain { registry, fitness_fn })
}

pub struct LoadedDomain {
    pub registry: Rc<AtomRegistry<LispVal>>,
    pub fitness_fn: LispVal,
}

/// Converts a GP tree node to its S-expression representation as a `LispVal`.
pub fn node_to_lisp_val(node: &Node) -> LispVal {
    if node.children.is_empty() {
        LispVal::Symbol(node.name.clone())
    } else {
        let mut elements: Vec<LispVal> = vec![LispVal::Symbol(node.name.clone())];
        elements.extend(node.children.iter().map(node_to_lisp_val));
        LispVal::List(elements)
    }
}

fn lisp_val_to_node(val: &LispVal) -> Result<Node, LispError> {
    match val {
        LispVal::Symbol(name) => Ok(Node::leaf(name)),
        LispVal::List(elements) if !elements.is_empty() => {
            let name: &str = elements[0].as_symbol()?;
            let children: Vec<Node> = elements[1..]
                .iter()
                .map(lisp_val_to_node)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Node::branch(name, children))
        }
        other => Err(LispError::Eval(format!(
            "expected a tree node (symbol or list), got {}",
            other.type_name()
        ))),
    }
}

fn gp_to_lisp_value(value: &Value) -> LispVal {
    match value {
        Value::Num(number) => LispVal::Num(*number),
        Value::Bool(boolean) => LispVal::Bool(*boolean),
    }
}

fn lisp_to_gp_value(val: LispVal, expected_type: Type) -> Value {
    match expected_type {
        Type::Num => Value::Num(
            val.as_num()
                .unwrap_or_else(|error| panic!("expected Num from script: {error}")),
        ),
        Type::Bool => Value::Bool(
            val.as_bool()
                .unwrap_or_else(|error| panic!("expected Bool from script: {error}")),
        ),
    }
}

pub fn parse_gp_type(name: &str) -> Result<Type, LispError> {
    match name {
        "Num" => Ok(Type::Num),
        "Bool" => Ok(Type::Bool),
        other => Err(LispError::Eval(format!(
            "unknown GP type `{other}` (expected `\"Num\"` or `\"Bool\"`)"
        ))),
    }
}

fn reg(
    env: &Rc<RefCell<Env>>,
    name: &str,
    func: impl Fn(&[LispVal]) -> Result<LispVal, LispError> + 'static,
) {
    let owned_name: String = name.to_string();
    env.borrow_mut().define(
        owned_name.clone(),
        LispVal::NativeFn { name: owned_name, func: Rc::new(func) },
    );
}
