use std::fmt;

use serde::Deserialize;

use crate::depth::Depth;
use crate::population_size::PopulationSize;
use crate::probability::Probability;
use crate::top_fraction::TopFraction;

pub struct GpConfig {
    pub population_size: PopulationSize,
    /// Max depth for initial trees and mutation-generated subtrees.
    pub max_depth: Depth,
    pub generations: usize,
    pub crossover_rate: Probability,
    pub mutation_rate: Probability,
    pub tournament_size: usize,
    /// Fraction of the population (by raw fitness) used to compute the median size
    /// reference for the size penalty.
    pub size_reference_fraction: TopFraction,
    /// Optional RNG seed for reproducible runs.
    pub seed: Option<u64>,
}

impl Default for GpConfig {
    fn default() -> Self {
        RawConfig::with_defaults()
            .try_into()
            .expect("default configuration values are always valid")
    }
}

/// Deserializable, all-optional mirror of `GpConfig` used for config-file and CLI layering.
/// Fields absent from a source are `None`, meaning "defer to the next layer".
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RawConfig {
    pub population_size: Option<usize>,
    pub max_depth: Option<usize>,
    pub generations: Option<usize>,
    pub crossover_rate: Option<usize>,
    pub mutation_rate: Option<usize>,
    pub tournament_size: Option<usize>,
    pub size_reference_fraction: Option<usize>,
    pub seed: Option<u64>,
}

impl RawConfig {
    /// Returns a `RawConfig` with every field set to the canonical default value.
    /// This is the single source of truth for defaults.
    pub fn with_defaults() -> RawConfig {
        RawConfig {
            population_size: Some(100),
            max_depth: Some(6),
            generations: Some(50),
            crossover_rate: Some(90),
            mutation_rate: Some(10),
            tournament_size: Some(3),
            size_reference_fraction: Some(10),
            seed: None,
        }
    }

    /// Merges `other` on top of `self`: any field set in `other` overrides `self`.
    pub fn merge(self, other: RawConfig) -> RawConfig {
        RawConfig {
            population_size: other.population_size.or(self.population_size),
            max_depth: other.max_depth.or(self.max_depth),
            generations: other.generations.or(self.generations),
            crossover_rate: other.crossover_rate.or(self.crossover_rate),
            mutation_rate: other.mutation_rate.or(self.mutation_rate),
            tournament_size: other.tournament_size.or(self.tournament_size),
            size_reference_fraction: other.size_reference_fraction.or(self.size_reference_fraction),
            seed: other.seed.or(self.seed),
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    BelowMinimum {
        field: &'static str,
        value: usize,
        min: usize,
    },
    AboveMaximum {
        field: &'static str,
        value: usize,
        max: usize,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::BelowMinimum { field, value, min } => {
                let flag: String = field.replace('_', "-");
                write!(
                    formatter,
                    "`{field}` value {value} is too small (must be at least {min})\n\
                     hint: set `{field} = {min}` or higher in gp-engine.conf, \
                     or pass --{flag} <N> where N >= {min}"
                )
            }
            ConfigError::AboveMaximum { field, value, max } => {
                let flag: String = field.replace('_', "-");
                write!(
                    formatter,
                    "`{field}` value {value} is too large (must be at most {max})\n\
                     hint: set `{field} = {max}` or lower in gp-engine.conf, \
                     or pass --{flag} <N> where N <= {max}"
                )
            }
        }
    }
}

impl TryFrom<RawConfig> for GpConfig {
    type Error = ConfigError;

    fn try_from(raw: RawConfig) -> Result<GpConfig, ConfigError> {
        let population_size: usize = raw.population_size.unwrap_or(100);
        if population_size < 1 {
            return Err(ConfigError::BelowMinimum {
                field: "population_size",
                value: population_size,
                min: 1,
            });
        }

        let max_depth: usize = raw.max_depth.unwrap_or(6);
        if max_depth < 1 {
            return Err(ConfigError::BelowMinimum {
                field: "max_depth",
                value: max_depth,
                min: 1,
            });
        }

        let generations: usize = raw.generations.unwrap_or(50);
        if generations < 1 {
            return Err(ConfigError::BelowMinimum {
                field: "generations",
                value: generations,
                min: 1,
            });
        }

        let crossover_rate: usize = raw.crossover_rate.unwrap_or(90);
        if crossover_rate > 100 {
            return Err(ConfigError::AboveMaximum {
                field: "crossover_rate",
                value: crossover_rate,
                max: 100,
            });
        }

        let mutation_rate: usize = raw.mutation_rate.unwrap_or(10);
        if mutation_rate > 100 {
            return Err(ConfigError::AboveMaximum {
                field: "mutation_rate",
                value: mutation_rate,
                max: 100,
            });
        }

        let tournament_size: usize = raw.tournament_size.unwrap_or(3);
        if tournament_size < 1 {
            return Err(ConfigError::BelowMinimum {
                field: "tournament_size",
                value: tournament_size,
                min: 1,
            });
        }

        let size_reference_fraction: usize = raw.size_reference_fraction.unwrap_or(10);
        if size_reference_fraction < 1 {
            return Err(ConfigError::BelowMinimum {
                field: "size_reference_fraction",
                value: size_reference_fraction,
                min: 1,
            });
        }
        if size_reference_fraction > 100 {
            return Err(ConfigError::AboveMaximum {
                field: "size_reference_fraction",
                value: size_reference_fraction,
                max: 100,
            });
        }

        Ok(GpConfig {
            population_size: PopulationSize::new(population_size),
            max_depth: Depth::new(max_depth),
            generations,
            crossover_rate: Probability::new(crossover_rate),
            mutation_rate: Probability::new(mutation_rate),
            tournament_size,
            size_reference_fraction: TopFraction::new(size_reference_fraction),
            seed: raw.seed,
        })
    }
}
