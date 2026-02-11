//! Monte Carlo simulation for draw poker.

use poker_core::{Card, Deck};
use poker_eval::{AceToFiveEvaluator, DeuceToSevenEvaluator, Evaluator, GameVariant, HoldemEvaluator, OmahaEvaluator};
use rand::rngs::StdRng;
use rand::RngCore;
use rand::SeedableRng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// Input parameters for a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationInput {
    /// Held cards (e.g., 2 cards for a 2-card draw)
    pub held_cards: Vec<Card>,
    /// Dead cards (other players' cards, muck, etc.)
    pub dead_cards: Vec<Card>,
    /// Number of simulated draws
    pub draw_count: u32,
    /// Optional seed for reproducibility
    pub seed: Option<u64>,
    /// Game variant for hand evaluation
    #[serde(default)]
    pub game_variant: GameVariant,
}

/// A single result: hand rank and percentile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandResult {
    pub hand: [Card; 5],
    pub rank: u32,
    pub percentile: f64,
}

/// Full simulation output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    pub results: Vec<HandResult>,
    pub draw_count: u32,
    pub best_rank: u32,
    pub worst_rank: u32,
    pub seed_used: u64,
}

/// Simulation engine for draw poker variants.
pub struct Simulator;

impl Default for Simulator {
    fn default() -> Self {
        Self
    }
}

impl Simulator {
    /// Run simulation. Held cards + drawn cards form the 5-card hand.
    pub fn run(&self, input: &SimulationInput) -> Result<SimulationResult, SimulatorError> {
        self.validate_input(input)?;

        let held: Vec<Card> = input.held_cards.clone();
        let dead: Vec<Card> = input.dead_cards.clone();
        let draw_count = input.draw_count;
        let seed = input
            .seed
            .unwrap_or_else(|| rand::rngs::OsRng.next_u64());
        let variant = input.game_variant;

        let all_excluded: Vec<Card> = held.iter().chain(dead.iter()).copied().collect();
        if all_excluded.len() != held.len() + dead.len() {
            return Err(SimulatorError::DuplicateCards);
        }

        if variant.is_board_game() {
            return self.run_board_game(input, &held, &dead, &all_excluded, draw_count, seed, variant);
        }

        let cards_to_draw = 5 - held.len();
        if cards_to_draw > 5 {
            return Err(SimulatorError::TooManyHeldCards);
        }

        if cards_to_draw == 0 {
            return self.run_five_held(input, &held, &dead, draw_count, seed, variant);
        }

        let mut trials: Vec<(u32, [Card; 5])> = (0..draw_count)
            .into_par_iter()
            .map(|i| {
                let mut rng = StdRng::seed_from_u64(seed.wrapping_add(i as u64));
                let trial_seed = rng.next_u64();
                Self::run_one_trial(&held, cards_to_draw, &all_excluded, trial_seed, variant)
            })
            .collect();

        trials.sort_unstable_by_key(|(r, _)| *r);

        let best_rank = trials.first().map(|(r, _)| *r).unwrap_or(0);
        let worst_rank = trials.last().map(|(r, _)| *r).unwrap_or(0);
        let n = trials.len() as f64;

        let results: Vec<HandResult> = trials
            .into_iter()
            .enumerate()
            .map(|(i, (rank, hand))| {
                let percentile = if n > 0.0 {
                    ((i as f64 + 0.5) / n) * 100.0
                } else {
                    0.0
                };
                HandResult {
                    hand,
                    rank,
                    percentile,
                }
            })
            .collect();

        Ok(SimulationResult {
            results,
            draw_count,
            best_rank,
            worst_rank,
            seed_used: seed,
        })
    }

    /// Board games: held cards (0 to max) + random board. Pad hole with random cards when needed.
    /// Dead cards are excluded from all draws. See GAME-RULES.md for game-specific rules.
    fn run_board_game(
        &self,
        _input: &SimulationInput,
        held: &[Card],
        _dead: &[Card],
        all_excluded: &[Card],
        draw_count: u32,
        seed: u64,
        variant: GameVariant,
    ) -> Result<SimulationResult, SimulatorError> {
        if variant.is_holdem_style() {
            // Hold'em: 2 hole + 5 board. Pad held (0–2) with random cards.
            let hole_count = 2;
            let mut trials: Vec<(u32, [Card; 5])> = (0..draw_count)
                .into_par_iter()
                .map(|i| {
                    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(i as u64));
                    let trial_seed = rng.next_u64();
                    let mut deck = Deck::new_excluding(all_excluded).with_seed(trial_seed);
                    deck.shuffle();
                    let mut full_hole: Vec<Card> = held.to_vec();
                    while full_hole.len() < hole_count {
                        full_hole.push(deck.draw(1).pop().unwrap());
                    }
                    let hole: [Card; 2] = full_hole.as_slice().try_into().unwrap();
                    let board: [Card; 5] = deck.draw(5).try_into().unwrap();
                    HoldemEvaluator::evaluate_holdem_with_hand(&hole, &board)
                })
                .collect();
            trials.sort_unstable_by_key(|(r, _)| *r);
            let best_rank = trials.first().map(|(r, _)| *r).unwrap_or(0);
            let worst_rank = trials.last().map(|(r, _)| *r).unwrap_or(0);
            let n = trials.len() as f64;
            let results: Vec<HandResult> = trials
                .into_iter()
                .enumerate()
                .map(|(i, (rank, hand))| {
                    let percentile = if n > 0.0 { ((i as f64 + 0.5) / n) * 100.0 } else { 0.0 };
                    HandResult { hand, rank, percentile }
                })
                .collect();
            return Ok(SimulationResult {
                results,
                draw_count,
                best_rank,
                worst_rank,
                seed_used: seed,
            });
        } else {
            // Omaha: 4 hole + 5 board. Pad held (0–4) with random cards.
            let hole_count = 4;
            let mut trials: Vec<(u32, [Card; 5])> = (0..draw_count)
                .into_par_iter()
                .map(|i| {
                    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(i as u64));
                    let trial_seed = rng.next_u64();
                    let mut deck = Deck::new_excluding(all_excluded).with_seed(trial_seed);
                    deck.shuffle();
                    let mut full_hole: Vec<Card> = held.to_vec();
                    while full_hole.len() < hole_count {
                        full_hole.push(deck.draw(1).pop().unwrap());
                    }
                    let hole: [Card; 4] = full_hole.as_slice().try_into().unwrap();
                    let board: [Card; 5] = deck.draw(5).try_into().unwrap();
                    OmahaEvaluator::evaluate_omaha_with_hand(&hole, &board)
                })
                .collect();
            trials.sort_unstable_by_key(|(r, _)| *r);
            let best_rank = trials.first().map(|(r, _)| *r).unwrap_or(0);
            let worst_rank = trials.last().map(|(r, _)| *r).unwrap_or(0);
            let n = trials.len() as f64;
            let results: Vec<HandResult> = trials
                .into_iter()
                .enumerate()
                .map(|(i, (rank, hand))| {
                    let percentile = if n > 0.0 { ((i as f64 + 0.5) / n) * 100.0 } else { 0.0 };
                    HandResult { hand, rank, percentile }
                })
                .collect();
            return Ok(SimulationResult {
                results,
                draw_count,
                best_rank,
                worst_rank,
                seed_used: seed,
            });
        }
    }

    fn run_five_held(
        &self,
        _input: &SimulationInput,
        held: &[Card],
        dead: &[Card],
        draw_count: u32,
        seed: u64,
        variant: GameVariant,
    ) -> Result<SimulationResult, SimulatorError> {
        let all_excluded: Vec<Card> = dead.iter().copied().collect();
        let hand_arr: [Card; 5] = held.try_into().unwrap();
        let hand_rank = Self::evaluate_hand(&hand_arr, variant);

        let mut trials: Vec<(u32, [Card; 5])> = (0..draw_count)
            .into_par_iter()
            .map(|i| {
                let mut rng = StdRng::seed_from_u64(seed.wrapping_add(i as u64));
                let trial_seed = rng.next_u64();
                Self::run_one_trial(&[], 5, &all_excluded, trial_seed, variant)
            })
            .collect();

        trials.sort_unstable_by_key(|(r, _)| *r);
        let count_better = trials.iter().filter(|(r, _)| *r < hand_rank).count();
        let count_same = trials.iter().filter(|(r, _)| *r == hand_rank).count();
        let percentile = if draw_count > 0 {
            ((count_better as f64 + count_same as f64 / 2.0) / draw_count as f64) * 100.0
        } else {
            0.0
        };

        let best_rank = trials.first().map(|(r, _)| *r).unwrap_or(0);
        let worst_rank = trials.last().map(|(r, _)| *r).unwrap_or(0);

        Ok(SimulationResult {
            results: vec![HandResult {
                hand: hand_arr,
                rank: hand_rank,
                percentile,
            }],
            draw_count,
            best_rank,
            worst_rank,
            seed_used: seed,
        })
    }

    fn evaluate_hand(cards: &[Card; 5], variant: GameVariant) -> u32 {
        match variant {
            GameVariant::DeuceToSeven => DeuceToSevenEvaluator.evaluate_5(cards),
            GameVariant::AceToFive => AceToFiveEvaluator.evaluate_5(cards),
            GameVariant::Holdem | GameVariant::Omaha | GameVariant::Omaha8 => unreachable!("Board games use run_board_game"),
        }
    }

    fn run_one_trial(
        held: &[Card],
        to_draw: usize,
        excluded: &[Card],
        seed: u64,
        variant: GameVariant,
    ) -> (u32, [Card; 5]) {
        let mut hand: Vec<Card> = held.to_vec();
        if to_draw > 0 {
            let mut deck = Deck::new_excluding(excluded).with_seed(seed);
            deck.shuffle();
            let drawn = deck.draw(to_draw);
            hand.extend(drawn);
        }
        let arr: [Card; 5] = hand.try_into().unwrap();
        let rank = Self::evaluate_hand(&arr, variant);
        (rank, arr)
    }

    fn validate_input(&self, input: &SimulationInput) -> Result<(), SimulatorError> {
        if input.game_variant.is_holdem_style() {
            if input.held_cards.len() > 2 {
                return Err(SimulatorError::TooManyHeldCards);
            }
        } else if input.game_variant.is_omaha_style() {
            if input.held_cards.len() > 4 {
                return Err(SimulatorError::TooManyHeldCards);
            }
        } else if input.held_cards.len() > 5 {
            return Err(SimulatorError::TooManyHeldCards);
        }
        if input.draw_count < 1 || input.draw_count > 1_000_000 {
            return Err(SimulatorError::InvalidDrawCount);
        }
        let all: Vec<Card> = input
            .held_cards
            .iter()
            .chain(input.dead_cards.iter())
            .copied()
            .collect();
        let unique: std::collections::HashSet<Card> = all.iter().copied().collect();
        if unique.len() != all.len() {
            return Err(SimulatorError::DuplicateCards);
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum SimulatorError {
    TooManyHeldCards,
    InvalidDrawCount,
    DuplicateCards,
}

impl std::fmt::Display for SimulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulatorError::TooManyHeldCards => write!(f, "Too many held cards"),
            SimulatorError::InvalidDrawCount => write!(f, "Invalid draw count"),
            SimulatorError::DuplicateCards => write!(f, "Duplicate cards in input"),
        }
    }
}

impl std::error::Error for SimulatorError {}

#[cfg(test)]
mod tests {
    use super::*;
    use poker_core::{Rank::*, Suit::*};
    use poker_eval::GameVariant;

    fn card(r: poker_core::Rank, s: poker_core::Suit) -> Card {
        Card::new(r, s)
    }

    #[test]
    fn draw_simulation_variety() {
        let input = SimulationInput {
            held_cards: vec![card(Two, Clubs), card(Three, Diamonds)],
            dead_cards: vec![],
            draw_count: 100,
            seed: None,
            game_variant: GameVariant::DeuceToSeven,
        };
        let sim = Simulator::default();
        let result = sim.run(&input).unwrap();
        assert_eq!(result.results.len(), 100);
        let unique_hands: std::collections::HashSet<_> = result.results.iter().map(|r| r.hand).collect();
        assert!(unique_hands.len() > 1, "Different draws should produce different hands");
    }

    #[test]
    fn five_held_returns_one_result_with_percentile() {
        let held = vec![
            card(Seven, Clubs),
            card(Five, Diamonds),
            card(Four, Hearts),
            card(Three, Spades),
            card(Two, Clubs),
        ];
        let input = SimulationInput {
            held_cards: held.clone(),
            dead_cards: vec![],
            draw_count: 1000,
            seed: Some(42),
            game_variant: GameVariant::DeuceToSeven,
        };
        let sim = Simulator::default();
        let result = sim.run(&input).unwrap();
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].hand.len(), 5);
        assert!(result.results[0].percentile >= 0.0 && result.results[0].percentile <= 100.0);
        let expected: [Card; 5] = held.as_slice().try_into().unwrap();
        assert_eq!(result.results[0].hand, expected);
    }

    #[test]
    fn holdem_simulation() {
        let hole = [card(Two, Clubs), card(Three, Diamonds)];
        let input = SimulationInput {
            held_cards: hole.to_vec(),
            dead_cards: vec![],
            draw_count: 20,
            seed: Some(777),
            game_variant: GameVariant::Holdem,
        };
        let sim = Simulator::default();
        let result = sim.run(&input).unwrap();
        assert_eq!(result.results.len(), 20);
    }

    #[test]
    fn holdem_zero_held_works() {
        let input = SimulationInput {
            held_cards: vec![],
            dead_cards: vec![],
            draw_count: 30,
            seed: Some(111),
            game_variant: GameVariant::Holdem,
        };
        let sim = Simulator::default();
        let result = sim.run(&input).unwrap();
        assert_eq!(result.results.len(), 30);
    }

    #[test]
    fn omaha_simulation() {
        let hole = [
            card(Two, Clubs),
            card(Three, Diamonds),
            card(Four, Hearts),
            card(Five, Spades),
        ];
        let input = SimulationInput {
            held_cards: hole.to_vec(),
            dead_cards: vec![],
            draw_count: 50,
            seed: Some(999),
            game_variant: GameVariant::Omaha,
        };
        let sim = Simulator::default();
        let result = sim.run(&input).unwrap();
        assert_eq!(result.results.len(), 50);
        assert!(result.results.iter().all(|r| r.hand.len() == 5));
    }

    #[test]
    fn omaha_two_held_padded() {
        let held = vec![card(Two, Clubs), card(Three, Diamonds)];
        let input = SimulationInput {
            held_cards: held,
            dead_cards: vec![],
            draw_count: 20,
            seed: Some(888),
            game_variant: GameVariant::Omaha,
        };
        let sim = Simulator::default();
        let result = sim.run(&input).unwrap();
        assert_eq!(result.results.len(), 20);
    }

    #[test]
    fn ace_to_five_variant_used() {
        let held = vec![
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
            card(Four, Spades),
            card(Five, Clubs),
        ];
        let input = SimulationInput {
            held_cards: held.clone(),
            dead_cards: vec![],
            draw_count: 500,
            seed: Some(123),
            game_variant: GameVariant::AceToFive,
        };
        let sim = Simulator::default();
        let result = sim.run(&input).unwrap();
        assert_eq!(result.results.len(), 1);
        assert!(result.results[0].percentile < 5.0,
            "Wheel A-2-3-4-5 should be top percentile in A-5");
    }
}
