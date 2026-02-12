//! Trait-based training scenario generation. Each game variant defines how training
//! scenarios are generated, what labels to show, and how percentiles are computed.

use poker_core::{Card, Deck};
use poker_eval::{GameVariant, HoldemEvaluator, O8Evaluator, OmahaEvaluator};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::simulation::{Simulator, SimulatorError};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A training scenario produced by a game-specific strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TrainingScenario {
    /// Draw games: 5-card hand + percentile.
    Draw {
        hand: [Card; 5],
        rank: u32,
        percentile: f64,
    },
    /// Board games: separate hole cards + board + percentile vs random opponents.
    Board {
        hole_cards: Vec<Card>,
        board: [Card; 5],
        best_hand: [Card; 5],
        rank: u32,
        percentile: f64,
    },
    /// O8 split pot: high + low.
    SplitPot {
        hole_cards: Vec<Card>,
        board: [Card; 5],
        best_high_hand: [Card; 5],
        high_rank: u32,
        high_percentile: f64,
        low_qualifies: bool,
        best_low_hand: Option<[Card; 5]>,
        low_rank: Option<u32>,
        low_percentile: Option<f64>,
    },
}

/// Metadata about how the frontend should display training for a game variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTrainingInfo {
    pub display_mode: String,
    pub held_label: String,
    pub held_hint: String,
    pub max_held: usize,
    pub has_low: bool,
    pub game_help: String,
}

/// Each game variant implements this to define its training scenario generation.
pub trait TrainingStrategy: Send + Sync {
    fn generate_scenario(
        &self,
        held_cards: &[Card],
        dead_cards: &[Card],
        sim_count: u32,
        seed: u64,
    ) -> Result<TrainingScenario, SimulatorError>;

    fn training_info(&self) -> GameTrainingInfo;
}

/// Factory: get the right training strategy for a game variant.
pub fn training_strategy(variant: GameVariant) -> Box<dyn TrainingStrategy> {
    match variant {
        GameVariant::DeuceToSeven => Box::new(DrawTrainingStrategy {
            variant: GameVariant::DeuceToSeven,
        }),
        GameVariant::AceToFive => Box::new(DrawTrainingStrategy {
            variant: GameVariant::AceToFive,
        }),
        GameVariant::Holdem => Box::new(BoardTrainingStrategy {
            variant: GameVariant::Holdem,
        }),
        GameVariant::Omaha => Box::new(BoardTrainingStrategy {
            variant: GameVariant::Omaha,
        }),
        GameVariant::Omaha8 => Box::new(SplitPotTrainingStrategy),
    }
}

// ---------------------------------------------------------------------------
// Draw games (2-7, A-5)
// ---------------------------------------------------------------------------

struct DrawTrainingStrategy {
    variant: GameVariant,
}

impl TrainingStrategy for DrawTrainingStrategy {
    fn generate_scenario(
        &self,
        held_cards: &[Card],
        dead_cards: &[Card],
        sim_count: u32,
        seed: u64,
    ) -> Result<TrainingScenario, SimulatorError> {
        validate_draw_input(held_cards, dead_cards, sim_count)?;

        let variant = self.variant;
        let all_excluded: Vec<Card> = held_cards.iter().chain(dead_cards.iter()).copied().collect();
        let cards_to_draw = 5 - held_cards.len();

        // If 5 cards held, evaluate directly against random pool.
        if cards_to_draw == 0 {
            let hand_arr: [Card; 5] = held_cards.try_into().unwrap();
            let hand_rank = Simulator::evaluate_hand(&hand_arr, variant);
            let excluded_for_pool: Vec<Card> = dead_cards.to_vec();

            let trials: Vec<u32> = (0..sim_count)
                .into_par_iter()
                .map(|i| {
                    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(i as u64));
                    let trial_seed = rng.next_u64();
                    let mut deck = Deck::new_excluding(&excluded_for_pool).with_seed(trial_seed);
                    deck.shuffle();
                    let drawn: [Card; 5] = deck.draw(5).try_into().unwrap();
                    Simulator::evaluate_hand(&drawn, variant)
                })
                .collect();

            let count_better = trials.iter().filter(|&&r| r < hand_rank).count();
            let count_same = trials.iter().filter(|&&r| r == hand_rank).count();
            let percentile = (count_better as f64 + count_same as f64 / 2.0)
                / sim_count as f64
                * 100.0;

            return Ok(TrainingScenario::Draw {
                hand: hand_arr,
                rank: hand_rank,
                percentile,
            });
        }

        // Run N draws, pick one at random.
        let held = held_cards.to_vec();
        let mut trials: Vec<(u32, [Card; 5])> = (0..sim_count)
            .into_par_iter()
            .map(|i| {
                let mut rng = StdRng::seed_from_u64(seed.wrapping_add(i as u64));
                let trial_seed = rng.next_u64();
                let mut deck = Deck::new_excluding(&all_excluded).with_seed(trial_seed);
                deck.shuffle();
                let drawn = deck.draw(cards_to_draw);
                let mut hand: Vec<Card> = held.clone();
                hand.extend(drawn);
                let arr: [Card; 5] = hand.try_into().unwrap();
                let rank = Simulator::evaluate_hand(&arr, variant);
                (rank, arr)
            })
            .collect();

        trials.sort_unstable_by_key(|(r, _)| *r);

        let mut pick_rng = StdRng::seed_from_u64(seed.wrapping_add(sim_count as u64));
        let idx = (pick_rng.next_u64() % trials.len() as u64) as usize;
        let (rank, hand) = trials[idx];
        let n = trials.len() as f64;
        // Find actual sorted position for percentile calculation.
        let pos = trials
            .iter()
            .position(|(r, h)| *r == rank && *h == hand)
            .unwrap_or(idx);
        let percentile = ((pos as f64 + 0.5) / n) * 100.0;

        Ok(TrainingScenario::Draw {
            hand,
            rank,
            percentile,
        })
    }

    fn training_info(&self) -> GameTrainingInfo {
        match self.variant {
            GameVariant::DeuceToSeven => GameTrainingInfo {
                display_mode: "draw".to_string(),
                held_label: "Your draw (cards you're keeping)".to_string(),
                held_hint: "0\u{2013}5 cards. We simulate random draws to complete 5-card hands."
                    .to_string(),
                max_held: 5,
                has_low: false,
                game_help: "Guess where a random draw ranks. 0% = nuts (7-5-4-3-2), 100% = worst. Straights and flushes count against you.".to_string(),
            },
            GameVariant::AceToFive => GameTrainingInfo {
                display_mode: "draw".to_string(),
                held_label: "Your draw (cards you're keeping)".to_string(),
                held_hint: "0\u{2013}5 cards. We simulate random draws to complete 5-card hands."
                    .to_string(),
                max_held: 5,
                has_low: false,
                game_help: "Guess where a random draw ranks. Aces low, straights/flushes ignored. 0% = wheel (A-2-3-4-5).".to_string(),
            },
            _ => unreachable!(),
        }
    }
}

// ---------------------------------------------------------------------------
// Board games (Hold'em, PLO)
// ---------------------------------------------------------------------------

struct BoardTrainingStrategy {
    variant: GameVariant,
}

impl TrainingStrategy for BoardTrainingStrategy {
    fn generate_scenario(
        &self,
        held_cards: &[Card],
        dead_cards: &[Card],
        sim_count: u32,
        seed: u64,
    ) -> Result<TrainingScenario, SimulatorError> {
        let hole_count = if self.variant.is_holdem_style() {
            2usize
        } else {
            4
        };
        if held_cards.len() > hole_count {
            return Err(SimulatorError::TooManyHeldCards);
        }
        validate_common(held_cards, dead_cards, sim_count)?;

        let all_excluded: Vec<Card> = held_cards.iter().chain(dead_cards.iter()).copied().collect();

        // Deal hero's hole cards + board once.
        let mut deal_rng = StdRng::seed_from_u64(seed);
        let deal_seed = deal_rng.next_u64();
        let mut deck = Deck::new_excluding(&all_excluded).with_seed(deal_seed);
        deck.shuffle();

        let mut full_hole: Vec<Card> = held_cards.to_vec();
        while full_hole.len() < hole_count {
            full_hole.push(deck.draw(1)[0]);
        }
        let board: [Card; 5] = deck.draw(5).try_into().unwrap();

        // Evaluate hero.
        let (hero_rank, best_hand) = if self.variant.is_holdem_style() {
            let hole: [Card; 2] = full_hole.as_slice().try_into().unwrap();
            HoldemEvaluator::evaluate_holdem_with_hand(&hole, &board)
        } else {
            let hole: [Card; 4] = full_hole.as_slice().try_into().unwrap();
            OmahaEvaluator::evaluate_omaha_with_hand(&hole, &board)
        };

        // Simulate random opponents on the SAME board.
        let board_copy = board;
        let excluded_from_opponents: Vec<Card> = all_excluded
            .iter()
            .chain(full_hole.iter())
            .chain(board.iter())
            .copied()
            .collect();
        let variant = self.variant;

        let opp_ranks: Vec<u32> = (0..sim_count)
            .into_par_iter()
            .map(|i| {
                let mut rng = StdRng::seed_from_u64(seed.wrapping_add(i as u64 + 1));
                let trial_seed = rng.next_u64();
                let mut opp_deck =
                    Deck::new_excluding(&excluded_from_opponents).with_seed(trial_seed);
                opp_deck.shuffle();
                if variant.is_holdem_style() {
                    let opp_hole: [Card; 2] = opp_deck.draw(2).try_into().unwrap();
                    HoldemEvaluator::evaluate_holdem(&opp_hole, &board_copy)
                } else {
                    let opp_hole: [Card; 4] = opp_deck.draw(4).try_into().unwrap();
                    OmahaEvaluator::evaluate_omaha(&opp_hole, &board_copy)
                }
            })
            .collect();

        let count_better = opp_ranks.iter().filter(|&&r| r < hero_rank).count();
        let count_same = opp_ranks.iter().filter(|&&r| r == hero_rank).count();
        let percentile =
            (count_better as f64 + count_same as f64 / 2.0) / sim_count as f64 * 100.0;

        Ok(TrainingScenario::Board {
            hole_cards: full_hole,
            board,
            best_hand,
            rank: hero_rank,
            percentile,
        })
    }

    fn training_info(&self) -> GameTrainingInfo {
        match self.variant {
            GameVariant::Holdem => GameTrainingInfo {
                display_mode: "board".to_string(),
                held_label: "Hole cards".to_string(),
                held_hint: "0\u{2013}2 cards. Empty = random hole cards dealt.".to_string(),
                max_held: 2,
                has_low: false,
                game_help: "See your hole cards and a board. Guess where your hand ranks vs random opponents on this board. 0% = nuts, 100% = worst.".to_string(),
            },
            GameVariant::Omaha => GameTrainingInfo {
                display_mode: "board".to_string(),
                held_label: "Hole cards".to_string(),
                held_hint: "0\u{2013}4 cards. Empty = random hole cards dealt. Must use exactly 2 from hand.".to_string(),
                max_held: 4,
                has_low: false,
                game_help: "See your hole cards and a board. Guess where your hand ranks vs random opponents. Must use exactly 2 from hand + 3 from board. 0% = nuts, 100% = worst.".to_string(),
            },
            _ => unreachable!(),
        }
    }
}

// ---------------------------------------------------------------------------
// Split pot (O8)
// ---------------------------------------------------------------------------

struct SplitPotTrainingStrategy;

impl TrainingStrategy for SplitPotTrainingStrategy {
    fn generate_scenario(
        &self,
        held_cards: &[Card],
        dead_cards: &[Card],
        sim_count: u32,
        seed: u64,
    ) -> Result<TrainingScenario, SimulatorError> {
        let hole_count = 4usize;
        if held_cards.len() > hole_count {
            return Err(SimulatorError::TooManyHeldCards);
        }
        validate_common(held_cards, dead_cards, sim_count)?;

        let all_excluded: Vec<Card> = held_cards.iter().chain(dead_cards.iter()).copied().collect();

        // Deal hero's hole cards + board.
        let mut deal_rng = StdRng::seed_from_u64(seed);
        let deal_seed = deal_rng.next_u64();
        let mut deck = Deck::new_excluding(&all_excluded).with_seed(deal_seed);
        deck.shuffle();

        let mut full_hole: Vec<Card> = held_cards.to_vec();
        while full_hole.len() < hole_count {
            full_hole.push(deck.draw(1)[0]);
        }
        let board: [Card; 5] = deck.draw(5).try_into().unwrap();
        let hole_arr: [Card; 4] = full_hole.as_slice().try_into().unwrap();

        // Evaluate hero high + low.
        let (hero_high_rank, best_high_hand) =
            O8Evaluator::evaluate_high_with_hand(&hole_arr, &board);
        let hero_low = O8Evaluator::evaluate_low_with_hand(&hole_arr, &board);
        let low_qualifies = hero_low.is_some();
        let hero_low_rank = hero_low.map(|(r, _)| r);
        let best_low_hand = hero_low.map(|(_, h)| h);

        // Simulate opponents on the same board.
        let board_copy = board;
        let excluded_from_opponents: Vec<Card> = all_excluded
            .iter()
            .chain(full_hole.iter())
            .chain(board.iter())
            .copied()
            .collect();

        let opp_results: Vec<(u32, Option<u32>)> = (0..sim_count)
            .into_par_iter()
            .map(|i| {
                let mut rng = StdRng::seed_from_u64(seed.wrapping_add(i as u64 + 1));
                let trial_seed = rng.next_u64();
                let mut opp_deck =
                    Deck::new_excluding(&excluded_from_opponents).with_seed(trial_seed);
                opp_deck.shuffle();
                let opp_hole: [Card; 4] = opp_deck.draw(4).try_into().unwrap();
                let high = O8Evaluator::evaluate_high(&opp_hole, &board_copy);
                let low = O8Evaluator::evaluate_low(&opp_hole, &board_copy);
                (high, low)
            })
            .collect();

        // High percentile.
        let count_better_high = opp_results.iter().filter(|(r, _)| *r < hero_high_rank).count();
        let count_same_high = opp_results.iter().filter(|(r, _)| *r == hero_high_rank).count();
        let high_percentile = (count_better_high as f64 + count_same_high as f64 / 2.0)
            / sim_count as f64
            * 100.0;

        // Low percentile (only among opponents who also qualify for low).
        let low_percentile = if let Some(hero_lr) = hero_low_rank {
            let opp_lows: Vec<u32> = opp_results.iter().filter_map(|(_, lo)| *lo).collect();
            if opp_lows.is_empty() {
                // Hero qualifies but no opponent does — hero has the only low.
                Some(0.0)
            } else {
                let count_better_low = opp_lows.iter().filter(|&&r| r < hero_lr).count();
                let count_same_low = opp_lows.iter().filter(|&&r| r == hero_lr).count();
                let pct = (count_better_low as f64 + count_same_low as f64 / 2.0)
                    / opp_lows.len() as f64
                    * 100.0;
                Some(pct)
            }
        } else {
            None
        };

        Ok(TrainingScenario::SplitPot {
            hole_cards: full_hole,
            board,
            best_high_hand,
            high_rank: hero_high_rank,
            high_percentile,
            low_qualifies,
            best_low_hand,
            low_rank: hero_low_rank,
            low_percentile,
        })
    }

    fn training_info(&self) -> GameTrainingInfo {
        GameTrainingInfo {
            display_mode: "split".to_string(),
            held_label: "Hole cards".to_string(),
            held_hint: "0\u{2013}4 cards. Empty = random hole cards dealt. Must use exactly 2 from hand.".to_string(),
            max_held: 4,
            has_low: true,
            game_help: "Split pot \u{2014} guess BOTH your high and low percentiles. Low requires 5 unpaired cards 8-or-below. 'No low' when the board can't make one.".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

fn validate_common(
    held_cards: &[Card],
    dead_cards: &[Card],
    sim_count: u32,
) -> Result<(), SimulatorError> {
    if sim_count < 1 || sim_count > 1_000_000 {
        return Err(SimulatorError::InvalidDrawCount);
    }
    let all: Vec<Card> = held_cards.iter().chain(dead_cards.iter()).copied().collect();
    let unique: std::collections::HashSet<Card> = all.iter().copied().collect();
    if unique.len() != all.len() {
        return Err(SimulatorError::DuplicateCards);
    }
    Ok(())
}

fn validate_draw_input(
    held_cards: &[Card],
    dead_cards: &[Card],
    sim_count: u32,
) -> Result<(), SimulatorError> {
    if held_cards.len() > 5 {
        return Err(SimulatorError::TooManyHeldCards);
    }
    validate_common(held_cards, dead_cards, sim_count)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use poker_core::{Rank::*, Suit::*};

    fn card(r: poker_core::Rank, s: poker_core::Suit) -> Card {
        Card::new(r, s)
    }

    #[test]
    fn draw_strategy_returns_draw_scenario() {
        let strategy = training_strategy(GameVariant::DeuceToSeven);
        let result = strategy
            .generate_scenario(&[], &[], 500, 42)
            .unwrap();
        match result {
            TrainingScenario::Draw {
                hand,
                rank: _,
                percentile,
            } => {
                assert_eq!(hand.len(), 5);
                assert!(percentile >= 0.0 && percentile <= 100.0);
            }
            _ => panic!("Expected Draw scenario"),
        }
    }

    #[test]
    fn draw_strategy_with_held_cards() {
        let held = vec![card(Seven, Clubs), card(Five, Diamonds)];
        let strategy = training_strategy(GameVariant::DeuceToSeven);
        let result = strategy
            .generate_scenario(&held, &[], 200, 99)
            .unwrap();
        match result {
            TrainingScenario::Draw { hand, .. } => {
                assert_eq!(hand.len(), 5);
            }
            _ => panic!("Expected Draw scenario"),
        }
    }

    #[test]
    fn draw_strategy_five_held() {
        let held = vec![
            card(Seven, Clubs),
            card(Five, Diamonds),
            card(Four, Hearts),
            card(Three, Spades),
            card(Two, Clubs),
        ];
        let strategy = training_strategy(GameVariant::DeuceToSeven);
        let result = strategy
            .generate_scenario(&held, &[], 500, 42)
            .unwrap();
        match result {
            TrainingScenario::Draw {
                hand, percentile, ..
            } => {
                assert_eq!(hand, <[Card; 5]>::try_from(held.as_slice()).unwrap());
                assert!(percentile < 5.0, "7-5-4-3-2 should be near-nuts");
            }
            _ => panic!("Expected Draw scenario"),
        }
    }

    #[test]
    fn ace_to_five_draw_strategy() {
        let strategy = training_strategy(GameVariant::AceToFive);
        let info = strategy.training_info();
        assert_eq!(info.display_mode, "draw");
        assert_eq!(info.max_held, 5);
        let result = strategy
            .generate_scenario(&[], &[], 200, 55)
            .unwrap();
        assert!(matches!(result, TrainingScenario::Draw { .. }));
    }

    #[test]
    fn holdem_board_strategy() {
        let strategy = training_strategy(GameVariant::Holdem);
        let info = strategy.training_info();
        assert_eq!(info.display_mode, "board");
        assert_eq!(info.max_held, 2);

        let result = strategy
            .generate_scenario(&[], &[], 200, 77)
            .unwrap();
        match result {
            TrainingScenario::Board {
                hole_cards,
                board,
                best_hand,
                percentile,
                ..
            } => {
                assert_eq!(hole_cards.len(), 2);
                assert_eq!(board.len(), 5);
                assert_eq!(best_hand.len(), 5);
                assert!(percentile >= 0.0 && percentile <= 100.0);
            }
            _ => panic!("Expected Board scenario"),
        }
    }

    #[test]
    fn holdem_with_held_hole_cards() {
        let held = vec![card(Ace, Spades), card(Ace, Hearts)];
        let strategy = training_strategy(GameVariant::Holdem);
        let result = strategy
            .generate_scenario(&held, &[], 500, 42)
            .unwrap();
        match result {
            TrainingScenario::Board {
                hole_cards,
                percentile,
                ..
            } => {
                assert_eq!(hole_cards[0], card(Ace, Spades));
                assert_eq!(hole_cards[1], card(Ace, Hearts));
                assert!(
                    percentile < 50.0,
                    "Aces should usually be top half: got {}",
                    percentile
                );
            }
            _ => panic!("Expected Board scenario"),
        }
    }

    #[test]
    fn omaha_board_strategy() {
        let strategy = training_strategy(GameVariant::Omaha);
        let info = strategy.training_info();
        assert_eq!(info.display_mode, "board");
        assert_eq!(info.max_held, 4);

        let result = strategy
            .generate_scenario(&[], &[], 200, 88)
            .unwrap();
        match result {
            TrainingScenario::Board {
                hole_cards, board, ..
            } => {
                assert_eq!(hole_cards.len(), 4);
                assert_eq!(board.len(), 5);
            }
            _ => panic!("Expected Board scenario"),
        }
    }

    #[test]
    fn o8_split_pot_strategy() {
        let strategy = training_strategy(GameVariant::Omaha8);
        let info = strategy.training_info();
        assert_eq!(info.display_mode, "split");
        assert!(info.has_low);
        assert_eq!(info.max_held, 4);

        let result = strategy
            .generate_scenario(&[], &[], 200, 99)
            .unwrap();
        match result {
            TrainingScenario::SplitPot {
                hole_cards,
                board,
                high_percentile,
                low_qualifies,
                low_percentile,
                ..
            } => {
                assert_eq!(hole_cards.len(), 4);
                assert_eq!(board.len(), 5);
                assert!(high_percentile >= 0.0 && high_percentile <= 100.0);
                if low_qualifies {
                    assert!(low_percentile.is_some());
                    let lp = low_percentile.unwrap();
                    assert!(lp >= 0.0 && lp <= 100.0);
                } else {
                    assert!(low_percentile.is_none());
                }
            }
            _ => panic!("Expected SplitPot scenario"),
        }
    }

    #[test]
    fn o8_with_low_qualifying_hole_cards() {
        // Force cards likely to make a low.
        let held = vec![
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
            card(Four, Spades),
        ];
        let strategy = training_strategy(GameVariant::Omaha8);
        let result = strategy
            .generate_scenario(&held, &[], 300, 42)
            .unwrap();
        match result {
            TrainingScenario::SplitPot {
                hole_cards,
                high_percentile,
                ..
            } => {
                assert_eq!(hole_cards[0], card(Ace, Clubs));
                assert!(high_percentile >= 0.0 && high_percentile <= 100.0);
            }
            _ => panic!("Expected SplitPot scenario"),
        }
    }

    #[test]
    fn too_many_held_cards_errors() {
        let held = vec![
            card(Two, Clubs),
            card(Three, Diamonds),
            card(Four, Hearts),
        ];
        let strategy = training_strategy(GameVariant::Holdem);
        let result = strategy.generate_scenario(&held, &[], 100, 42);
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_cards_errors() {
        let held = vec![card(Ace, Clubs)];
        let dead = vec![card(Ace, Clubs)];
        let strategy = training_strategy(GameVariant::DeuceToSeven);
        let result = strategy.generate_scenario(&held, &dead, 100, 42);
        assert!(result.is_err());
    }
}
