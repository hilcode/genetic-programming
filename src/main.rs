pub(crate) mod atom;
pub(crate) mod cli;
pub(crate) mod config;
mod crossover;
mod depth;
mod engine;
mod generate;
mod mutation;
pub(crate) mod node;
mod population;
mod population_size;
mod probability;
mod selection;
mod top_fraction;

use std::path::PathBuf;
use std::process;

use atom::AtomDefinition;
use atom::AtomRegistry;
use atom::Type;
use atom::Value;
use clap::Parser;
use cli::Cli;
use config::GpConfig;
use config::RawConfig;
use engine::GpEngine;
use node::Node;

pub struct Context {
    pub target: i64,
    pub flag: bool,
}

fn build_registry() -> AtomRegistry<Context> {
    let mut registry: AtomRegistry<Context> = AtomRegistry::new(Type::Num);

    // Numeric operators
    registry.register("+", AtomDefinition::new(
        Type::Num, vec![Type::Num, Type::Num],
        |args: &[Value], _ctx: &Context| {
            Value::Num(args[0].as_num().saturating_add(args[1].as_num()))
        },
    ));
    registry.register("-", AtomDefinition::new(
        Type::Num, vec![Type::Num, Type::Num],
        |args: &[Value], _ctx: &Context| {
            Value::Num(args[0].as_num().saturating_sub(args[1].as_num()))
        },
    ));
    registry.register("*", AtomDefinition::new(
        Type::Num, vec![Type::Num, Type::Num],
        |args: &[Value], _ctx: &Context| {
            Value::Num(args[0].as_num().saturating_mul(args[1].as_num()))
        },
    ));

    // Control flow
    registry.register("IF", AtomDefinition::new(
        Type::Num, vec![Type::Bool, Type::Num, Type::Num],
        |args: &[Value], _ctx: &Context| {
            if args[0].as_bool() { Value::Num(args[1].as_num()) } else { Value::Num(args[2].as_num()) }
        },
    ));

    // Boolean operators
    registry.register("NOT", AtomDefinition::new(
        Type::Bool, vec![Type::Bool],
        |args: &[Value], _ctx: &Context| Value::Bool(!args[0].as_bool()),
    ));
    registry.register("AND", AtomDefinition::new(
        Type::Bool, vec![Type::Bool, Type::Bool],
        |args: &[Value], _ctx: &Context| Value::Bool(args[0].as_bool() && args[1].as_bool()),
    ));
    registry.register("OR", AtomDefinition::new(
        Type::Bool, vec![Type::Bool, Type::Bool],
        |args: &[Value], _ctx: &Context| Value::Bool(args[0].as_bool() || args[1].as_bool()),
    ));
    registry.register("XOR", AtomDefinition::new(
        Type::Bool, vec![Type::Bool, Type::Bool],
        |args: &[Value], _ctx: &Context| Value::Bool(args[0].as_bool() ^ args[1].as_bool()),
    ));

    // Numeric terminals
    registry.register("TARGET", AtomDefinition::new(
        Type::Num, vec![],
        |_args: &[Value], ctx: &Context| Value::Num(ctx.target),
    ));
    for constant in -10i64..=10 {
        registry.register(&constant.to_string(), AtomDefinition::new(
            Type::Num, vec![],
            move |_args: &[Value], _ctx: &Context| Value::Num(constant),
        ));
    }

    // Boolean terminals
    registry.register("TRUE", AtomDefinition::new(
        Type::Bool, vec![],
        |_args: &[Value], _ctx: &Context| Value::Bool(true),
    ));
    registry.register("FALSE", AtomDefinition::new(
        Type::Bool, vec![],
        |_args: &[Value], _ctx: &Context| Value::Bool(false),
    ));
    registry.register("FLAG", AtomDefinition::new(
        Type::Bool, vec![],
        |_args: &[Value], ctx: &Context| Value::Bool(ctx.flag),
    ));

    registry
}

fn main() {
    let cli: Cli = Cli::parse();
    let config_path: PathBuf =
        cli.config.clone().unwrap_or_else(|| PathBuf::from("gp-engine.conf"));
    let has_explicit_config: bool = cli.config.is_some();
    let cli_raw: RawConfig = cli.into_raw_config();

    let file_raw: RawConfig = if config_path.exists() {
        let contents: String = std::fs::read_to_string(&config_path).unwrap_or_else(|error| {
            eprintln!("error: failed to read {}: {}", config_path.display(), error);
            process::exit(1);
        });
        toml::from_str(&contents).unwrap_or_else(|error| {
            eprintln!("error: failed to parse {}: {}", config_path.display(), error);
            process::exit(1);
        })
    } else if has_explicit_config {
        eprintln!("error: config file not found: {}", config_path.display());
        eprintln!("hint: create the file or omit --config to run without a config file");
        process::exit(1);
    } else {
        RawConfig::default()
    };

    let merged: RawConfig = RawConfig::with_defaults().merge(file_raw).merge(cli_raw);

    let gp_config: GpConfig = merged.try_into().unwrap_or_else(|error| {
        eprintln!("error: {}", error);
        process::exit(1);
    });

    let engine = GpEngine::new(
        gp_config,
        build_registry(),
        |node: &Node, registry: &AtomRegistry<Context>| {
            let true_value: i64 =
                registry.eval(node, &Context { target: 100, flag: true }).as_num();
            let false_value: i64 =
                registry.eval(node, &Context { target: 100, flag: false }).as_num();
            let true_diff: i64 = (true_value - 42).abs();
            let false_diff: i64 = (false_value - 123).abs();
            // Fitness is the product of per-target scores. Summing the diffs would create a
            // flat region for any value in [42, 123], giving the GP no gradient to follow.
            // Multiplying penalises being wrong on either target independently.
            (1.0 / (1.0 + true_diff as f64)) * (1.0 / (1.0 + false_diff as f64))
        },
    );

    let best: Node = engine.run();
    let true_result: i64 = engine.eval(&best, &Context { target: 100, flag: true }).as_num();
    let false_result: i64 = engine.eval(&best, &Context { target: 100, flag: false }).as_num();

    println!("\nBest expression: {}", best);
    println!("FLAG=true  evaluates to: {} (target 42)",  true_result);
    println!("FLAG=false evaluates to: {} (target 123)", false_result);
}
