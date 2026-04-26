use gp_engine::test_support::LispError;
use gp_engine::LispVal;
use gp_engine::ScriptEngine;
use std::path::Path;

fn eval_one(input: &str) -> LispVal {
    let mut results: Vec<LispVal> = ScriptEngine::new()
        .run_str(input)
        .expect("eval failed");
    assert_eq!(results.len(), 1, "expected exactly one expression");
    results.remove(0)
}

// ── Special forms ─────────────────────────────────────────────────────────────

#[test]
fn test_if_selects_true_branch() {
    assert_eq!(eval_one("(if true 1 2)").as_num().expect("expected Num"), 1);
}

#[test]
fn test_if_selects_false_branch() {
    assert_eq!(eval_one("(if false 1 2)").as_num().expect("expected Num"), 2);
}

#[test]
fn test_if_without_else_returns_nil() {
    assert!(eval_one("(if false 1)").is_nil());
}

#[test]
fn test_and_returns_false_when_any_arg_is_false() {
    assert!(!eval_one("(and true false)").as_bool().expect("expected Bool"));
}

#[test]
fn test_and_short_circuits_on_first_false() {
    assert!(!eval_one("(and false true)").as_bool().expect("expected Bool"));
}

#[test]
fn test_or_returns_true_when_any_arg_is_true() {
    assert!(eval_one("(or false true)").as_bool().expect("expected Bool"));
}

#[test]
fn test_or_short_circuits_on_first_true() {
    assert!(eval_one("(or true false)").as_bool().expect("expected Bool"));
}

#[test]
fn test_begin_returns_last_expression() {
    assert_eq!(eval_one("(begin 1 2 3)").as_num().expect("expected Num"), 3);
}

// ── Arithmetic builtins ───────────────────────────────────────────────────────

#[test]
fn test_multiply() {
    assert_eq!(eval_one("(* 3 4)").as_num().expect("expected Num"), 12);
}

#[test]
fn test_subtract() {
    assert_eq!(eval_one("(- 10 3)").as_num().expect("expected Num"), 7);
}

#[test]
fn test_divide_returns_float() {
    assert_eq!(eval_one("(/ 10.0 4.0)").as_float().expect("expected Float"), 2.5);
}

#[test]
fn test_mod() {
    assert_eq!(eval_one("(mod 10 3)").as_num().expect("expected Num"), 1);
}

#[test]
fn test_abs_of_negative() {
    assert_eq!(eval_one("(abs -5)").as_num().expect("expected Num"), 5);
}

// ── Comparison builtins ───────────────────────────────────────────────────────

#[test]
fn test_equal() {
    assert!(eval_one("(= 1 1)").as_bool().expect("expected Bool"));
}

#[test]
fn test_not_equal() {
    assert!(eval_one("(!= 1 2)").as_bool().expect("expected Bool"));
}

#[test]
fn test_less_than() {
    assert!(eval_one("(< 1 2)").as_bool().expect("expected Bool"));
}

#[test]
fn test_greater_than() {
    assert!(eval_one("(> 2 1)").as_bool().expect("expected Bool"));
}

#[test]
fn test_less_than_or_equal() {
    assert!(eval_one("(<= 1 1)").as_bool().expect("expected Bool"));
}

#[test]
fn test_greater_than_or_equal() {
    assert!(eval_one("(>= 2 1)").as_bool().expect("expected Bool"));
}

// ── Boolean builtins ──────────────────────────────────────────────────────────

#[test]
fn test_not_negates_true() {
    assert!(!eval_one("(not true)").as_bool().expect("expected Bool"));
}

// ── List builtins ─────────────────────────────────────────────────────────────

#[test]
fn test_first_returns_head() {
    assert_eq!(eval_one("(first (list 1 2 3))").as_num().expect("expected Num"), 1);
}

#[test]
fn test_rest_returns_tail() {
    let result: LispVal = eval_one("(rest (list 1 2 3))");
    let elements: &[LispVal] = result.as_list().expect("expected List");
    assert_eq!(elements.len(), 2);
    assert_eq!(elements[0].as_num().expect("expected Num"), 2);
    assert_eq!(elements[1].as_num().expect("expected Num"), 3);
}

#[test]
fn test_prepend_adds_element_to_front() {
    let result: LispVal = eval_one("(prepend 0 (list 1 2))");
    let elements: &[LispVal] = result.as_list().expect("expected List");
    assert_eq!(elements.len(), 3);
    assert_eq!(elements[0].as_num().expect("expected Num"), 0);
    assert_eq!(elements[1].as_num().expect("expected Num"), 1);
}

#[test]
fn test_empty_returns_true_for_empty_list() {
    assert!(eval_one("(empty? (list))").as_bool().expect("expected Bool"));
}

#[test]
fn test_length_of_list() {
    assert_eq!(eval_one("(length (list 1 2 3))").as_num().expect("expected Num"), 3);
}

// ── String builtins ───────────────────────────────────────────────────────────

#[test]
fn test_concat_joins_strings() {
    assert_eq!(
        eval_one(r#"(concat "hello" " " "world")"#).as_str().expect("expected Str"),
        "hello world"
    );
}

#[test]
fn test_slice_extracts_substring() {
    assert_eq!(
        eval_one(r#"(slice "hello" 1 3)"#).as_str().expect("expected Str"),
        "el"
    );
}

// ── Type predicates ───────────────────────────────────────────────────────────

#[test]
fn test_num_predicate() {
    assert!(eval_one("(num? 42)").as_bool().expect("expected Bool"));
    assert!(!eval_one("(num? true)").as_bool().expect("expected Bool"));
}

#[test]
fn test_bool_predicate() {
    assert!(eval_one("(bool? true)").as_bool().expect("expected Bool"));
    assert!(!eval_one("(bool? 1)").as_bool().expect("expected Bool"));
}

#[test]
fn test_str_predicate() {
    assert!(eval_one(r#"(str? "hi")"#).as_bool().expect("expected Bool"));
    assert!(!eval_one("(str? 1)").as_bool().expect("expected Bool"));
}

#[test]
fn test_list_predicate() {
    assert!(eval_one("(list? (list 1))").as_bool().expect("expected Bool"));
    assert!(!eval_one("(list? 1)").as_bool().expect("expected Bool"));
}

#[test]
fn test_nil_predicate() {
    assert!(eval_one("(nil? nil)").as_bool().expect("expected Bool"));
    assert!(!eval_one("(nil? 1)").as_bool().expect("expected Bool"));
}

// ── run_file ──────────────────────────────────────────────────────────────────

#[test]
fn test_run_file_evaluates_script() {
    let engine: ScriptEngine = ScriptEngine::new();
    let results: Vec<LispVal> = engine
        .run_file(Path::new("tests/fixtures/arithmetic.lisp"))
        .expect("run_file failed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_num().expect("expected Num"), 42);
}

#[test]
fn test_run_file_returns_error_for_missing_file() {
    let engine: ScriptEngine = ScriptEngine::new();
    let result: Result<Vec<LispVal>, LispError> =
        engine.run_file(Path::new("tests/fixtures/nonexistent.lisp"));
    let error_message: String = result.err().expect("expected an error").to_string();
    assert!(error_message.contains("nonexistent.lisp"));
}
