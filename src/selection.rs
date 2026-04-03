use crate::node::Node;
use crate::population::Population;
use rand::Rng;
use std::cmp::Ordering;

/// Selects the fittest individual from a random sample of `tournament_size` candidates.
pub fn tournament<'a>(
    population: &'a Population,
    fitnesses: &[f64],
    tournament_size: usize,
    rng: &mut impl Rng,
) -> &'a Node {
    let winner: usize = (0..tournament_size)
        .map(|_| population.random_index(rng))
        .max_by(|&left_index, &right_index| {
            fitnesses[left_index]
                .partial_cmp(&fitnesses[right_index])
                .unwrap_or(Ordering::Equal)
        })
        .unwrap();
    &population[winner]
}
