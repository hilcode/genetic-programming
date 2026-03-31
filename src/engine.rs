use std::cmp::Ordering;

use crate::crossover::subtree_crossover;
use crate::depth::Depth;
use crate::generate::ramped_half_and_half;
use crate::mutation::subtree_mutation;
use crate::population::Population;
use crate::population_size::PopulationSize;
use crate::probability::Probability;
use crate::selection::tournament;
use crate::top_fraction::TopFraction;
use crate::tree::Expr;
use rand::SeedableRng;
use rand::rngs::StdRng;

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
        Self {
            population_size: PopulationSize::new(100),
            max_depth: Depth::new(6),
            generations: 50,
            crossover_rate: Probability::new(90),
            mutation_rate: Probability::new(10),
            tournament_size: 3,
            size_reference_fraction: TopFraction::new(10),
            seed: None,
        }
    }
}

pub struct GpEngine<F> {
    config: GpConfig,
    fitness_fn: F,
}

impl<F: Fn(&Expr) -> f64> GpEngine<F> {
    pub fn new(config: GpConfig, fitness_fn: F) -> Self {
        Self { config, fitness_fn }
    }

    pub fn run(&self) -> Expr {
        let mut rng: StdRng = match self.config.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let mut population: Population =
            ramped_half_and_half(self.config.population_size, Depth::new(2), self.config.max_depth, &mut rng);
        let (_, mut fitnesses): (Vec<f64>, Vec<f64>) = self.evaluate_all(&population);
        let mut raw_fitnesses: Vec<f64>;

        let (mut best_idx, _): (usize, f64) = best_index(&fitnesses);
        let mut best: Expr = population[best_idx].clone();

        for generation in 0..self.config.generations {
            let mut next_population: Population = Population::with_capacity(self.config.population_size);

            // Elitism: carry the best individual forward unchanged.
            next_population.push(best.clone());

            while next_population.size() < self.config.population_size {
                let parent1: Expr =
                    tournament(&population, &fitnesses, self.config.tournament_size, &mut rng).clone();
                let parent2: Expr =
                    tournament(&population, &fitnesses, self.config.tournament_size, &mut rng).clone();

                let (mut child1, mut child2): (Expr, Expr) = if self.config.crossover_rate.occurs(&mut rng) {
                    subtree_crossover(&parent1, &parent2, &mut rng)
                } else {
                    (parent1, parent2)
                };

                if self.config.mutation_rate.occurs(&mut rng) {
                    child1 = subtree_mutation(&child1, self.config.max_depth, &mut rng);
                }
                if self.config.mutation_rate.occurs(&mut rng) {
                    child2 = subtree_mutation(&child2, self.config.max_depth, &mut rng);
                }

                next_population.push(child1);
                if next_population.size() < self.config.population_size {
                    next_population.push(child2);
                }
            }

            population = next_population;
            (raw_fitnesses, fitnesses) = self.evaluate_all(&population);
            best_idx = best_index(&fitnesses).0;
            best = population[best_idx].clone();

            println!(
                "Gen {:>3}: best fitness = {:.4}  expr = {}",
                generation + 1,
                raw_fitnesses[best_idx],
                best
            );
        }

        best
    }

    fn evaluate_all(&self, population: &Population) -> (Vec<f64>, Vec<f64>) {
        let raw_fitnesses: Vec<f64> = population.iter()
            .map(|expr| (self.fitness_fn)(expr))
            .collect();

        let min_fitness: f64 = raw_fitnesses.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_fitness: f64 = raw_fitnesses.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let fitness_range: f64 = max_fitness - min_fitness;
        let normalized: Vec<f64> = if fitness_range > 0.0 {
            raw_fitnesses.iter().map(|&fitness| (fitness - min_fitness) / fitness_range).collect()
        } else {
            vec![0.0; raw_fitnesses.len()]
        };

        let (reference_size, median_fitness): (usize, f64) =
            self.size_reference(population, &normalized);

        let adjusted: Vec<f64> = normalized.iter().zip(population.iter())
            .map(|(&fitness, expr)| {
                let size: usize = expr.size();
                let size_factor: f64 = (reference_size as f64 / size as f64).min(1.0);
                fitness * (1.0 - median_fitness * (1.0 - size_factor))
            })
            .collect();

        (raw_fitnesses, adjusted)
    }

    /// Computes the median size and median fitness of the top fraction of individuals
    /// by normalised fitness. The median fitness serves as the size penalty scale.
    fn size_reference(&self, population: &Population, normalized_fitnesses: &[f64]) -> (usize, f64) {
        let top_count: usize = self.config.size_reference_fraction.count(population.size());

        let mut indices: Vec<usize> = (0..normalized_fitnesses.len()).collect();
        indices.sort_by(|&left, &right| {
            normalized_fitnesses[right].partial_cmp(&normalized_fitnesses[left]).unwrap_or(Ordering::Equal)
        });

        let mut top_sizes: Vec<usize> = indices[..top_count]
            .iter()
            .map(|&index| population[index].size())
            .collect();

        top_sizes.sort();

        let reference_size: usize = if top_count % 2 == 1 {
            top_sizes[top_count / 2]
        } else {
            (top_sizes[top_count / 2 - 1] + top_sizes[top_count / 2]) / 2
        };

        let median_fitness: f64 = if top_count % 2 == 1 {
            normalized_fitnesses[indices[top_count / 2]]
        } else {
            (normalized_fitnesses[indices[top_count / 2 - 1]]
                + normalized_fitnesses[indices[top_count / 2]]) / 2.0
        };

        (reference_size, median_fitness)
    }
}

fn best_index(fitnesses: &[f64]) -> (usize, f64) {
    fitnesses
        .iter()
        .enumerate()
        .max_by(|(_, left_fitness), (_, right_fitness)| {
            left_fitness.partial_cmp(right_fitness).unwrap_or(Ordering::Equal)
        })
        .map(|(index, &fitness)| (index, fitness))
        .unwrap()
}
