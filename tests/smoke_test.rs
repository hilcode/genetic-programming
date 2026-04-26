use gp_engine::test_support::LoadedDomain;
use gp_engine::test_support::Type;
use gp_engine::Node;
use gp_engine::ScriptEngine;
use std::path::Path;

#[test]
fn test_load_simple_script() {
    let script_engine: ScriptEngine = ScriptEngine::new();
    let domain: LoadedDomain = script_engine
        .load_domain_file(Path::new("tests/fixtures/simple.lisp"))
        .expect("Failed to load script");

    // Verify we got a registry with the expected operators and terminals
    let num_operators: &[String] = domain.registry.operators_of_type(Type::Num);
    assert!(num_operators.contains(&"+".to_string()));

    let num_terminals: &[String] = domain.registry.terminals_of_type(Type::Num);
    assert!(num_terminals.contains(&"TARGET".to_string()));
    assert!(num_terminals.contains(&"ONE".to_string()));

    // Verify we can create a simple tree: (+ ONE ONE)
    let tree: Node = Node::branch("+", vec![Node::leaf("ONE"), Node::leaf("ONE")]);
    assert_eq!(tree.size(), 3); // 1 root + 2 leaves

    // Verify the tree has the correct type
    let tree_type: Type = domain.registry.type_of(&tree);
    assert_eq!(tree_type, Type::Num);
}
