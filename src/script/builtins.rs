use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

use crate::script::value::LispError;
use crate::script::value::LispVal;

/// Returns a HashMap of all standard builtin functions.
pub fn register_builtins() -> HashMap<String, LispVal> {
    let mut bindings: HashMap<String, LispVal> = HashMap::new();

    // Helper: wraps a function in a NativeFn and adds it to bindings.
    fn reg(
        bindings: &mut HashMap<String, LispVal>,
        name: &str,
        func: impl Fn(&[LispVal]) -> Result<LispVal, LispError> + 'static,
    ) {
        let owned_name: String = name.to_string();
        bindings.insert(
            owned_name.clone(),
            LispVal::NativeFn { name: owned_name, func: Rc::new(func) },
        );
    }

    // Arithmetic
    reg(&mut bindings,"+",   builtin_add);
    reg(&mut bindings,"-",   builtin_sub);
    reg(&mut bindings,"*",   builtin_mul);
    reg(&mut bindings,"/",   builtin_div);
    reg(&mut bindings,"abs", builtin_abs);
    reg(&mut bindings,"mod", builtin_mod);

    // Comparison
    reg(&mut bindings,"=",  |args| compare_eq(args,  "=",  false));
    reg(&mut bindings,"!=", |args| compare_eq(args,  "!=", true));
    reg(&mut bindings,"<",  |args| compare_ord(args, "<",  |ord| ord == Ordering::Less));
    reg(&mut bindings,">",  |args| compare_ord(args, ">",  |ord| ord == Ordering::Greater));
    reg(&mut bindings,"<=", |args| compare_ord(args, "<=", |ord| ord != Ordering::Greater));
    reg(&mut bindings,">=", |args| compare_ord(args, ">=", |ord| ord != Ordering::Less));

    // Boolean (`and`/`or` are special forms in the evaluator; only `not` is a function)
    reg(&mut bindings,"not", builtin_not);

    // List operations
    reg(&mut bindings,"list",    |args| Ok(LispVal::List(args.to_vec())));
    reg(&mut bindings,"first",   builtin_first);
    reg(&mut bindings,"rest",    builtin_rest);
    reg(&mut bindings,"prepend", builtin_prepend);
    reg(&mut bindings,"empty?",  builtin_empty);
    reg(&mut bindings,"length",  builtin_length);
    reg(&mut bindings,"nth",     builtin_nth);

    // String operations
    reg(&mut bindings,"concat", builtin_concat);
    reg(&mut bindings,"slice",  builtin_slice);

    // Type predicates
    reg(&mut bindings,"num?",    |args| type_predicate(args, "num?",    |value| matches!(value, LispVal::Num(_))));
    reg(&mut bindings,"float?",  |args| type_predicate(args, "float?",  |value| matches!(value, LispVal::Float(_))));
    reg(&mut bindings,"bool?",   |args| type_predicate(args, "bool?",   |value| matches!(value, LispVal::Bool(_))));
    reg(&mut bindings,"str?",    |args| type_predicate(args, "str?",    |value| matches!(value, LispVal::Str(_))));
    reg(&mut bindings,"symbol?", |args| type_predicate(args, "symbol?", |value| matches!(value, LispVal::Symbol(_))));
    reg(&mut bindings,"list?",   |args| type_predicate(args, "list?",   |value| matches!(value, LispVal::List(_))));
    reg(&mut bindings,"nil?",    |args| type_predicate(args, "nil?",    LispVal::is_nil));

    // Association list
    reg(&mut bindings,"get", builtin_get);

    // I/O
    reg(&mut bindings,"print", builtin_print);

    bindings
}

// ── Arithmetic ────────────────────────────────────────────────────────────────

fn builtin_add(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.iter().any(|value| matches!(value, LispVal::Float(_))) {
        let mut result: f64 = 0.0;
        for value in args { result += value.as_float()?; }
        Ok(LispVal::Float(result))
    } else {
        let mut result: i64 = 0;
        for value in args { result = result.saturating_add(value.as_num()?); }
        Ok(LispVal::Num(result))
    }
}

fn builtin_sub(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.is_empty() {
        return Err(LispError::arity_at_least("-", 1, 0));
    }
    if args.iter().any(|value| matches!(value, LispVal::Float(_))) {
        if args.len() == 1 {
            return Ok(LispVal::Float(-args[0].as_float()?));
        }
        let mut result: f64 = args[0].as_float()?;
        for value in &args[1..] { result -= value.as_float()?; }
        Ok(LispVal::Float(result))
    } else {
        if args.len() == 1 {
            return Ok(LispVal::Num(-args[0].as_num()?));
        }
        let mut result: i64 = args[0].as_num()?;
        for value in &args[1..] { result = result.saturating_sub(value.as_num()?); }
        Ok(LispVal::Num(result))
    }
}

fn builtin_mul(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.iter().any(|value| matches!(value, LispVal::Float(_))) {
        let mut result: f64 = 1.0;
        for value in args { result *= value.as_float()?; }
        Ok(LispVal::Float(result))
    } else {
        let mut result: i64 = 1;
        for value in args { result = result.saturating_mul(value.as_num()?); }
        Ok(LispVal::Num(result))
    }
}

fn builtin_div(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.is_empty() {
        return Err(LispError::arity_at_least("/", 1, 0));
    }
    // Division always returns Float to avoid surprising integer truncation.
    if args.len() == 1 {
        let denominator: f64 = args[0].as_float()?;
        if denominator == 0.0 {
            return Err(LispError::Eval("division by zero".to_string()));
        }
        return Ok(LispVal::Float(1.0 / denominator));
    }
    let mut result: f64 = args[0].as_float()?;
    for value in &args[1..] {
        let divisor: f64 = value.as_float()?;
        if divisor == 0.0 {
            return Err(LispError::Eval("division by zero".to_string()));
        }
        result /= divisor;
    }
    Ok(LispVal::Float(result))
}

fn builtin_abs(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 1 {
        return Err(LispError::arity("abs", 1, args.len()));
    }
    match &args[0] {
        LispVal::Num(number) => Ok(LispVal::Num(number.abs())),
        LispVal::Float(number) => Ok(LispVal::Float(number.abs())),
        other => Err(LispError::type_mismatch("Num or Float", other)),
    }
}

fn builtin_mod(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 2 {
        return Err(LispError::arity("mod", 2, args.len()));
    }
    let dividend: i64 = args[0].as_num()?;
    let divisor: i64 = args[1].as_num()?;
    if divisor == 0 {
        return Err(LispError::Eval("mod: division by zero".to_string()));
    }
    Ok(LispVal::Num(dividend % divisor))
}

// ── Comparison ────────────────────────────────────────────────────────────────

fn lisp_ord(left: &LispVal, right: &LispVal) -> Result<Ordering, LispError> {
    match (left, right) {
        (LispVal::Num(lhs), LispVal::Num(rhs)) => Ok(lhs.cmp(rhs)),
        (LispVal::Float(lhs), LispVal::Float(rhs)) => lhs
            .partial_cmp(rhs)
            .ok_or_else(|| LispError::Eval("cannot compare NaN".to_string())),
        (LispVal::Num(lhs), LispVal::Float(rhs)) => (*lhs as f64)
            .partial_cmp(rhs)
            .ok_or_else(|| LispError::Eval("cannot compare NaN".to_string())),
        (LispVal::Float(lhs), LispVal::Num(rhs)) => lhs
            .partial_cmp(&(*rhs as f64))
            .ok_or_else(|| LispError::Eval("cannot compare NaN".to_string())),
        (LispVal::Str(lhs), LispVal::Str(rhs)) => Ok(lhs.cmp(rhs)),
        (LispVal::Bool(lhs), LispVal::Bool(rhs)) => Ok(lhs.cmp(rhs)),
        _ => Err(LispError::Eval(format!(
            "cannot compare {} and {}",
            left.type_name(),
            right.type_name()
        ))),
    }
}

fn lisp_equal(left: &LispVal, right: &LispVal) -> bool {
    match (left, right) {
        (LispVal::Num(lhs), LispVal::Num(rhs)) => lhs == rhs,
        (LispVal::Float(lhs), LispVal::Float(rhs)) => lhs == rhs,
        (LispVal::Num(lhs), LispVal::Float(rhs)) => (*lhs as f64) == *rhs,
        (LispVal::Float(lhs), LispVal::Num(rhs)) => *lhs == (*rhs as f64),
        (LispVal::Bool(lhs), LispVal::Bool(rhs)) => lhs == rhs,
        (LispVal::Str(lhs), LispVal::Str(rhs)) => lhs == rhs,
        (LispVal::Symbol(lhs), LispVal::Symbol(rhs)) => lhs == rhs,
        (LispVal::Nil, LispVal::Nil) => true,
        (LispVal::List(lhs), LispVal::List(rhs)) => {
            lhs.len() == rhs.len()
                && lhs
                    .iter()
                    .zip(rhs.iter())
                    .all(|(left_elem, right_elem)| lisp_equal(left_elem, right_elem))
        }
        _ => false,
    }
}

fn compare_eq(args: &[LispVal], name: &str, negate: bool) -> Result<LispVal, LispError> {
    if args.len() != 2 {
        return Err(LispError::arity(name, 2, args.len()));
    }
    let equal: bool = lisp_equal(&args[0], &args[1]);
    Ok(LispVal::Bool(if negate { !equal } else { equal }))
}

fn compare_ord(
    args: &[LispVal],
    name: &str,
    pred: impl Fn(Ordering) -> bool,
) -> Result<LispVal, LispError> {
    if args.len() != 2 {
        return Err(LispError::arity(name, 2, args.len()));
    }
    Ok(LispVal::Bool(pred(lisp_ord(&args[0], &args[1])?)))
}

// ── Boolean ───────────────────────────────────────────────────────────────────

fn builtin_not(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 1 {
        return Err(LispError::arity("not", 1, args.len()));
    }
    Ok(LispVal::Bool(!args[0].as_bool()?))
}

// ── List operations ───────────────────────────────────────────────────────────

fn builtin_first(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 1 {
        return Err(LispError::arity("first", 1, args.len()));
    }
    let elements: &[LispVal] = args[0].as_list()?;
    elements
        .first()
        .cloned()
        .ok_or_else(|| LispError::Eval("first: empty list".to_string()))
}

fn builtin_rest(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 1 {
        return Err(LispError::arity("rest", 1, args.len()));
    }
    let elements: &[LispVal] = args[0].as_list()?;
    if elements.is_empty() {
        return Err(LispError::Eval("rest: empty list".to_string()));
    }
    Ok(LispVal::List(elements[1..].to_vec()))
}

fn builtin_prepend(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 2 {
        return Err(LispError::arity("prepend", 2, args.len()));
    }
    let element: LispVal = args[0].clone();
    let elements: &[LispVal] = args[1].as_list()?;
    let mut new_elements: Vec<LispVal> = Vec::with_capacity(elements.len() + 1);
    new_elements.push(element);
    new_elements.extend_from_slice(elements);
    Ok(LispVal::List(new_elements))
}

fn builtin_empty(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 1 {
        return Err(LispError::arity("empty?", 1, args.len()));
    }
    Ok(LispVal::Bool(args[0].is_nil()))
}

fn builtin_length(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 1 {
        return Err(LispError::arity("length", 1, args.len()));
    }
    match &args[0] {
        LispVal::List(elements) => Ok(LispVal::Num(elements.len() as i64)),
        LispVal::Str(string) => Ok(LispVal::Num(string.chars().count() as i64)),
        LispVal::Nil => Ok(LispVal::Num(0)),
        other => Err(LispError::type_mismatch("List or Str", other)),
    }
}

fn builtin_nth(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 2 {
        return Err(LispError::arity("nth", 2, args.len()));
    }
    let elements: &[LispVal] = args[0].as_list()?;
    let index: i64 = args[1].as_num()?;
    if index < 0 || index as usize >= elements.len() {
        return Err(LispError::Eval(format!(
            "nth: index {index} out of bounds (list length {})",
            elements.len()
        )));
    }
    Ok(elements[index as usize].clone())
}

// ── String operations ─────────────────────────────────────────────────────────

fn builtin_concat(args: &[LispVal]) -> Result<LispVal, LispError> {
    let mut result: String = String::new();
    for value in args {
        result.push_str(value.as_str()?);
    }
    Ok(LispVal::Str(result))
}

fn builtin_slice(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 3 {
        return Err(LispError::arity("slice", 3, args.len()));
    }
    let string: &str = args[0].as_str()?;
    let start: i64 = args[1].as_num()?;
    let end: i64 = args[2].as_num()?;
    let chars: Vec<char> = string.chars().collect();
    let char_count: usize = chars.len();
    if start < 0 || end < 0 || start as usize > char_count || end as usize > char_count || start > end {
        return Err(LispError::Eval(format!(
            "slice: invalid range {start}..{end} for string of length {char_count}"
        )));
    }
    let sliced: String = chars[start as usize..end as usize].iter().collect();
    Ok(LispVal::Str(sliced))
}

// ── Type predicates ───────────────────────────────────────────────────────────

fn type_predicate(
    args: &[LispVal],
    name: &str,
    pred: impl Fn(&LispVal) -> bool,
) -> Result<LispVal, LispError> {
    if args.len() != 1 {
        return Err(LispError::arity(name, 1, args.len()));
    }
    Ok(LispVal::Bool(pred(&args[0])))
}

// ── Association list ──────────────────────────────────────────────────────────

fn builtin_get(args: &[LispVal]) -> Result<LispVal, LispError> {
    if args.len() != 2 {
        return Err(LispError::arity("get", 2, args.len()));
    }
    let pairs: &[LispVal] = args[0].as_list()?;
    let key: &LispVal = &args[1];
    for pair in pairs {
        let elements: &[LispVal] = pair.as_list()?;
        if elements.len() != 2 {
            return Err(LispError::Eval(format!(
                "get: expected pairs of length 2, got length {}",
                elements.len()
            )));
        }
        if lisp_equal(&elements[0], key) {
            return Ok(elements[1].clone());
        }
    }
    Err(LispError::Eval(format!("get: key {key} not found")))
}

// ── I/O ───────────────────────────────────────────────────────────────────────

fn builtin_print(args: &[LispVal]) -> Result<LispVal, LispError> {
    let parts: Vec<String> = args.iter().map(LispVal::to_string).collect();
    println!("{}", parts.join(" "));
    Ok(LispVal::Nil)
}
