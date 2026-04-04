use std::path::PathBuf;

use clap::Parser;

use crate::config::RawConfig;

/// GP Engine — a genetic programming engine
///
/// Evolves a population of symbolic expression trees over a series of generations
/// using selection, crossover, and mutation.
///
/// Configuration is layered: built-in defaults are overridden by values in the
/// configuration file, which are in turn overridden by command-line flags.
#[derive(Parser)]
#[command(
    name = "gp-engine",
    after_help = "\
CONFIGURATION FILE (gp-engine.conf):
    Any flag below can be set persistently in a TOML file:

        population_size         = 100
        max_depth               = 6
        generations             = 50
        crossover_rate          = 90
        mutation_rate           = 10
        tournament_size         = 3
        size_reference_fraction = 10
        # seed = 42

    Pass --config <FILE> to use a file at a non-default path."
)]
pub struct Cli {
    /// Path to the domain script (.scm file)
    #[arg(value_name = "SCRIPT")]
    pub script: PathBuf,

    /// Path to a TOML configuration file [default: ./gp-engine.conf, if it exists]
    ///
    /// Settings in the file are overridden by any flags passed on the command line.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Number of individuals in the population [default: 100]
    ///
    /// Larger populations explore more of the search space at the cost of more
    /// fitness evaluations per generation.
    #[arg(long, value_name = "N")]
    pub population_size: Option<usize>,

    /// Maximum tree depth for initial trees and mutation subtrees [default: 6]
    ///
    /// Deeper trees can represent more complex programs but increase evaluation
    /// cost and bloat pressure.
    #[arg(long, value_name = "N")]
    pub max_depth: Option<usize>,

    /// Number of generations to evolve [default: 50]
    #[arg(long, value_name = "N")]
    pub generations: Option<usize>,

    /// Crossover probability as a percentage (0–100) [default: 90]
    ///
    /// When two parents are selected, this is the chance their subtrees are
    /// swapped to produce children. If crossover does not occur, the parents
    /// are passed through unchanged.
    #[arg(long, value_name = "N")]
    pub crossover_rate: Option<usize>,

    /// Mutation probability as a percentage (0–100) [default: 10]
    ///
    /// Each child is independently mutated at this rate by replacing a randomly
    /// chosen subtree with a freshly generated one.
    #[arg(long, value_name = "N")]
    pub mutation_rate: Option<usize>,

    /// Number of individuals competing in each tournament selection [default: 3]
    ///
    /// Higher values increase selection pressure, favouring fitter individuals
    /// more strongly.
    #[arg(long, value_name = "N")]
    pub tournament_size: Option<usize>,

    /// Elite fraction used as the size-penalty reference, as a percentage (1–100) [default: 10]
    ///
    /// The median tree size and fitness of this top group set the scale of the
    /// size penalty applied to bloated individuals.
    #[arg(long, value_name = "N")]
    pub size_reference_fraction: Option<usize>,

    /// RNG seed for reproducible runs [default: random]
    ///
    /// When omitted, a random seed is chosen at startup.
    #[arg(long, value_name = "N")]
    pub seed: Option<u64>,
}

impl Cli {
    pub fn into_raw_config(self) -> RawConfig {
        RawConfig {
            population_size: self.population_size,
            max_depth: self.max_depth,
            generations: self.generations,
            crossover_rate: self.crossover_rate,
            mutation_rate: self.mutation_rate,
            tournament_size: self.tournament_size,
            size_reference_fraction: self.size_reference_fraction,
            seed: self.seed,
        }
    }
}
