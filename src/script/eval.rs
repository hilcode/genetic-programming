use std::cell::RefCell;
use std::rc::Rc;

use crate::script::scope::Scope;
use crate::script::value::LispError;
use crate::script::value::LispVal;

/// Evaluates `expr` in the given environment and returns the result.
pub fn eval(expr: &LispVal, env: &Rc<RefCell<Scope>>) -> Result<LispVal, LispError> {
    match expr {
        // Self-evaluating atoms
        LispVal::Num(_)
        | LispVal::Float(_)
        | LispVal::Bool(_)
        | LispVal::Str(_)
        | LispVal::Nil
        | LispVal::NativeFn { .. }
        | LispVal::Lambda { .. } => Ok(expr.clone()),

        // Symbol: look up in the environment
        LispVal::Symbol(name) => env
            .borrow()
            .lookup(name)
            .ok_or_else(|| LispError::Unbound(name.clone())),

        // Empty list evaluates to nil
        LispVal::List(elements) if elements.is_empty() => Ok(LispVal::Nil),

        // Non-empty list: special form or function call
        LispVal::List(elements) => eval_list(elements, env),
    }
}

/// Applies an already-evaluated `func` to the already-evaluated `args`.
pub fn apply(func: &LispVal, args: Vec<LispVal>) -> Result<LispVal, LispError> {
    match func {
        LispVal::NativeFn { func: native_func, .. } => native_func(&args),
        LispVal::Lambda { params, body, env: closure_env } => {
            if args.len() != params.len() {
                return Err(LispError::Eval(format!(
                    "lambda expected {} argument(s), got {}",
                    params.len(),
                    args.len()
                )));
            }
            let call_env: Rc<RefCell<Scope>> =
                Rc::new(RefCell::new(Scope::new_child(Rc::clone(closure_env))));
            for (param, value) in params.iter().zip(args) {
                call_env.borrow_mut().define(param.clone(), value);
            }
            eval_sequence(body, &call_env)
        }
        other => Err(LispError::Eval(format!("not callable: {other}"))),
    }
}

fn eval_list(elements: &[LispVal], env: &Rc<RefCell<Scope>>) -> Result<LispVal, LispError> {
    let head: &LispVal = &elements[0];
    let tail: &[LispVal] = &elements[1..];

    // Dispatch special forms before evaluating arguments
    if let LispVal::Symbol(name) = head {
        match name.as_str() {
            "lambda" => return eval_lambda(tail, env),
            "let"    => return eval_let(tail, env),
            "if"     => return eval_if(tail, env),
            "and"    => return eval_and(tail, env),
            "or"     => return eval_or(tail, env),
            "quote"  => return eval_quote(tail),
            "begin"  => return eval_begin(tail, env),
            _ => {}
        }
    }

    // Regular function call: evaluate head and all arguments, then apply
    let func: LispVal = eval(head, env)?;
    let args: Vec<LispVal> = tail
        .iter()
        .map(|arg| eval(arg, env))
        .collect::<Result<Vec<_>, _>>()?;
    apply(&func, args)
}

fn eval_sequence(exprs: &[LispVal], env: &Rc<RefCell<Scope>>) -> Result<LispVal, LispError> {
    let mut result: LispVal = LispVal::Nil;
    for expr in exprs {
        result = eval(expr, env)?;
    }
    Ok(result)
}

fn eval_lambda(args: &[LispVal], env: &Rc<RefCell<Scope>>) -> Result<LispVal, LispError> {
    if args.len() < 2 {
        return Err(LispError::Eval(
            "`lambda` requires a parameter list and at least one body expression".to_string(),
        ));
    }
    let params: Vec<String> = args[0]
        .as_list()?
        .iter()
        .map(|param| param.as_symbol().map(str::to_string))
        .collect::<Result<Vec<_>, _>>()?;
    let body: Vec<LispVal> = args[1..].to_vec();
    Ok(LispVal::Lambda { params, body, env: Rc::clone(env) })
}

fn eval_let(args: &[LispVal], env: &Rc<RefCell<Scope>>) -> Result<LispVal, LispError> {
    if args.len() < 2 {
        return Err(LispError::Eval(
            "`let` requires a binding list and at least one body expression".to_string(),
        ));
    }
    let bindings: &[LispVal] = args[0].as_list()?;
    let let_env: Rc<RefCell<Scope>> = Rc::new(RefCell::new(Scope::new_child(Rc::clone(env))));
    for binding in bindings {
        let pair: &[LispVal] = binding.as_list()?;
        if pair.len() != 2 {
            return Err(LispError::Eval(
                "each `let` binding must be a 2-element list `(name value)`".to_string(),
            ));
        }
        let name: String = pair[0].as_symbol()?.to_string();
        let value: LispVal = eval(&pair[1], env)?;
        let_env.borrow_mut().define(name, value);
    }
    eval_sequence(&args[1..], &let_env)
}

fn eval_if(args: &[LispVal], env: &Rc<RefCell<Scope>>) -> Result<LispVal, LispError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(LispError::Eval(
            "`if` requires 2 or 3 arguments: (if condition then [else])".to_string(),
        ));
    }
    let condition: bool = eval(&args[0], env)?.as_bool()?;
    if condition {
        eval(&args[1], env)
    } else if args.len() == 3 {
        eval(&args[2], env)
    } else {
        Ok(LispVal::Nil)
    }
}

fn eval_and(args: &[LispVal], env: &Rc<RefCell<Scope>>) -> Result<LispVal, LispError> {
    for arg in args {
        let value: LispVal = eval(arg, env)?;
        if !value.as_bool()? {
            return Ok(LispVal::Bool(false));
        }
    }
    Ok(LispVal::Bool(true))
}

fn eval_or(args: &[LispVal], env: &Rc<RefCell<Scope>>) -> Result<LispVal, LispError> {
    for arg in args {
        let value: LispVal = eval(arg, env)?;
        if value.as_bool()? {
            return Ok(LispVal::Bool(true));
        }
    }
    Ok(LispVal::Bool(false))
}

fn eval_quote(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 1 {
        return Err(LispError::arity("quote", 1, args.len()));
    }
    Ok(args[0].clone())
}

fn eval_begin(args: &[LispVal], env: &Rc<RefCell<Scope>>) -> Result<LispVal, LispError> {
    eval_sequence(args, env)
}
