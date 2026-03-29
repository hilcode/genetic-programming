use std::cmp::Ordering;

use crate::crossover::subtree_crossover;
use crate::depth::Depth;
use crate::generate::ramped_half_and_half;
use crate::mutation::subtree_mutation;
use crate::population::Population;
use crate::population_size::PopulationSize;
use crate::probability::Probability;
use crate::selection::tournament;
use crate::tree::Node;
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
            seed: None,
        }
    }
}

pub struct GpEngine<F> {
    config: GpConfig,
    fitness_fn: F,
}

impl<F: Fn(&Node) -> f64> GpEngine<F> {
    pub fn new(config: GpConfig, fitness_fn: F) -> Self {
        Self { config, fitness_fn }
    }

    pub fn run(&self) -> Node {
        let mut rng: StdRng = match self.config.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let mut population: Population =
            ramped_half_and_half(self.config.population_size, Depth::new(2), self.config.max_depth, &mut rng);
        let mut fitnesses: Vec<f64> = self.evaluate_all(&population);

        let (mut best_idx, _): (usize, f64) = best_index(&fitnesses);
        let mut best: Node = population[best_idx].clone();

        for generation in 0..self.config.generations {
            let mut next_population: Population = Population::with_capacity(self.config.population_size);

            // Elitism: carry the best individual forward unchanged.
            next_population.push(best.clone());

            while next_population.size() < self.config.population_size {
                let parent1: Node =
                    tournament(&population, &fitnesses, self.config.tournament_size, &mut rng).clone();
                let parent2: Node =
                    tournament(&population, &fitnesses, self.config.tournament_size, &mut rng).clone();

                let (mut child1, mut child2): (Node, Node) = if self.config.crossover_rate.occurs(&mut rng) {
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
            fitnesses = self.evaluate_all(&population);
            best_idx = best_index(&fitnesses).0;
            best = population[best_idx].clone();

            println!(
                "Gen {:>3}: best fitness = {:.4}  expr = {}",
                generation + 1,
                fitnesses[best_idx],
                best
            );
        }

        best
    }

    fn evaluate_all(&self, population: &Population) -> Vec<f64> {
        population.iter().map(|tree| (self.fitness_fn)(tree)).collect()
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
