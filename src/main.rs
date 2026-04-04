pub(crate) mod atom;
pub(crate) mod cli;
pub(crate) mod config;
pub(crate) mod script;
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

use atom::AtomRegistry;
use clap::Parser;
use cli::Cli;
use config::GpConfig;
use config::RawConfig;
use engine::GpEngine;
use node::Node;
use script::LispVal;
use script::ScriptEngine;

fn main() {
    let cli: Cli = Cli::parse();
    let script_path: PathBuf = cli.script.clone();
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

    let script_engine: ScriptEngine = ScriptEngine::new();
    let domain = script_engine.load_domain_file(&script_path).unwrap_or_else(|error| {
        eprintln!("error: {}", error);
        process::exit(1);
    });

    let fitness_fn: LispVal = domain.fitness_fn;
    let engine: GpEngine<LispVal, _> = GpEngine::new(
        gp_config,
        domain.registry,
        move |node: &Node, _registry: &AtomRegistry<LispVal>| {
            let node_val: LispVal = script::node_to_lisp_val(node);
            let result: LispVal = script::apply(&fitness_fn, vec![node_val])
                .unwrap_or_else(|error| panic!("fitness eval failed: {error}"));
            result.as_float()
                .unwrap_or_else(|error| panic!("fitness must return a number: {error}"))
        },
    );

    let best: Node = engine.run();
    println!("\nBest expression: {best}");
}
