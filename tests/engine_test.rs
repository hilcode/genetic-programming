use gp_engine::apply;
use gp_engine::node_to_lisp_val;
use gp_engine::test_support::LispError;
use gp_engine::test_support::LoadedDomain;
use gp_engine::test_support::simplify_tree;
use gp_engine::AtomRegistry;
use gp_engine::GpConfig;
use gp_engine::GpEngine;
use gp_engine::LispVal;
use gp_engine::Node;
use gp_engine::RawConfig;
use gp_engine::ScriptEngine;
use std::path::Path;

fn load_simple_domain() -> LoadedDomain {
    ScriptEngine::new()
        .load_domain_file(Path::new("tests/fixtures/simple.lisp"))
        .expect("failed to load simple.lisp")
}

fn evaluate_fitness(domain: &LoadedDomain, tree: &Node) -> f64 {
    let tree_val: LispVal = node_to_lisp_val(tree);
    apply(&domain.fitness_fn, vec![tree_val])
        .expect("fitness eval failed")
        .as_float()
        .expect("fitness must return a number")
}

fn small_seeded_config(seed: u64) -> GpConfig {
    RawConfig::with_defaults()
        .merge(RawConfig {
            population_size: Some(30),
            generations: Some(5),
            seed: Some(seed),
            ..Default::default()
        })
        .try_into()
        .expect("valid test config")
}

#[test]
fn test_terminal_tree_has_perfect_fitness() {
    let domain: LoadedDomain = load_simple_domain();
    // TARGET returns the target value (5); fitness = -(|5 - 5|) = 0.0
    let tree: Node = Node::leaf("TARGET");
    assert_eq!(evaluate_fitness(&domain, &tree), 0.0);
}

#[test]
fn test_operator_tree_has_expected_fitness() {
    let domain: LoadedDomain = load_simple_domain();
    // (+ ONE ONE) evaluates to 2; fitness = -(|2 - 5|) = -3.0
    let tree: Node = Node::branch("+", vec![Node::leaf("ONE"), Node::leaf("ONE")]);
    assert_eq!(evaluate_fitness(&domain, &tree), -3.0);
}

#[test]
fn test_seeded_engine_run_is_deterministic() {
    let domain: LoadedDomain = load_simple_domain();
    let fitness_fn: LispVal = domain.fitness_fn;
    let engine: GpEngine<LispVal, _> = GpEngine::new(
        small_seeded_config(42),
        domain.registry,
        move |node: &Node, _registry: &AtomRegistry<LispVal>| {
            let tree_val: LispVal = node_to_lisp_val(node);
            apply(&fitness_fn, vec![tree_val])
                .expect("fitness eval failed")
                .as_float()
                .expect("fitness must return a number")
        },
        domain.simplifications,
    );

    let best_first_run: Node = engine.run();
    let best_second_run: Node = engine.run();
    assert_eq!(best_first_run.to_string(), best_second_run.to_string());
}

#[test]
fn test_domain_without_fitness_returns_error() {
    let engine: ScriptEngine = ScriptEngine::new();
    let result: Result<LoadedDomain, LispError> =
        engine.load_domain_file(Path::new("tests/fixtures/no_fitness.lisp"));
    let error_message: String = result.err().expect("expected an error").to_string();
    assert!(error_message.contains("fitness"));
}

#[test]
fn test_domain_without_terminals_returns_error() {
    let engine: ScriptEngine = ScriptEngine::new();
    let result: Result<LoadedDomain, LispError> =
        engine.load_domain_file(Path::new("tests/fixtures/no_terminals.lisp"));
    let error_message: String = result.err().expect("expected an error").to_string();
    assert!(error_message.contains("terminal"));
}

// ── Simplification ────────────────────────────────────────────────────────────

#[test]
fn test_simplification_fires_on_matching_tree() {
    let domain: LoadedDomain = load_simple_domain();
    // add-zero-l: (+ 0 ?x) → ?x
    let tree: Node = Node::branch("+", vec![Node::leaf("0"), Node::leaf("ONE")]);
    let simplified: Node = simplify_tree(&tree, &domain.simplifications);
    assert_eq!(simplified.to_string(), "ONE");
}

#[test]
fn test_simplification_applies_to_children_first() {
    let domain: LoadedDomain = load_simple_domain();
    // not-true: (NOT TRUE) → FALSE, then not-false: (NOT FALSE) → TRUE
    // simplify_tree recurses into children before trying rules at the root
    let tree: Node = Node::branch("NOT", vec![Node::branch("NOT", vec![Node::leaf("TRUE")])]);
    let simplified: Node = simplify_tree(&tree, &domain.simplifications);
    assert_eq!(simplified.to_string(), "TRUE");
}

#[test]
fn test_simplification_checks_repeated_variable_binding() {
    let domain: LoadedDomain = load_simple_domain();
    // if-same: (IF ?cond ?x ?x) → ?x — the second ?x must equal the first
    let tree: Node = Node::branch(
        "IF",
        vec![Node::leaf("COND"), Node::leaf("SAME"), Node::leaf("SAME")],
    );
    let simplified: Node = simplify_tree(&tree, &domain.simplifications);
    assert_eq!(simplified.to_string(), "SAME");
}
