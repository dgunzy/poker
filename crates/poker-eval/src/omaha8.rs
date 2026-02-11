//! Omaha 8-or-better (O8) evaluator. 4 hole + 5 board.
//! High: standard rankings. Low: A-5, qualifier 8-or-better (no card higher than 8).

use poker_core::Card;

use crate::{AceToFiveEvaluator, Evaluator};

/// O8 evaluator. Returns (high_rank, low_rank) where low_rank is None if no qualifier.
#[derive(Debug, Default, Clone, Copy)]
pub struct O8Evaluator;

impl O8Evaluator {
    fn choose2(indices: &[usize]) -> Vec<[usize; 2]> {
        let mut out = Vec::new();
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                out.push([indices[i], indices[j]]);
            }
        }
        out
    }

    fn choose3(indices: &[usize]) -> Vec<[usize; 3]> {
        let mut out = Vec::new();
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                for k in (j + 1)..indices.len() {
                    out.push([indices[i], indices[j], indices[k]]);
                }
            }
        }
        out
    }

    /// 8-or-better: no card higher than 8. Ace=0, 2=1, ..., 8=7.
    fn qualifies_low_fixed(cards: &[Card; 5]) -> bool {
        for c in cards {
            let v = AceToFiveEvaluator::ace_low_value(c.rank());
            if v > 7 {
                return false;
            }
        }
        let counts = AceToFiveEvaluator::rank_counts(cards);
        counts.iter().all(|&c| c <= 1)
    }

    fn best_low_rank(hole: &[Card; 4], board: &[Card; 5]) -> Option<u32> {
        let hole_indices = [0, 1, 2, 3];
        let board_indices = [0, 1, 2, 3, 4];
        let hole_combos = Self::choose2(&hole_indices);
        let board_combos = Self::choose3(&board_indices);

        let mut best = None;
        for &[h1, h2] in &hole_combos {
            for &[b1, b2, b3] in &board_combos {
                let hand = [hole[h1], hole[h2], board[b1], board[b2], board[b3]];
                if Self::qualifies_low_fixed(&hand) {
                    let rank = AceToFiveEvaluator.evaluate_5(&hand);
                    best = Some(best.map_or(rank, |b: u32| b.min(rank)));
                }
            }
        }
        best
    }

    /// Best high hand rank.
    pub fn evaluate_high(hole: &[Card; 4], board: &[Card; 5]) -> u32 {
        crate::omaha::OmahaEvaluator::evaluate_omaha(hole, board)
    }

    /// Best low hand rank if qualifier exists. None = no qualifying low.
    pub fn evaluate_low(hole: &[Card; 4], board: &[Card; 5]) -> Option<u32> {
        Self::best_low_rank(hole, board)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poker_core::{Rank::*, Suit, Suit::*};

    fn card(r: poker_core::Rank, s: Suit) -> Card {
        Card::new(r, s)
    }

    #[test]
    fn wheel_low_qualifies_and_is_nuts() {
        let hole = [
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(King, Hearts),
            card(Queen, Spades),
        ];
        let board = [
            card(Three, Clubs),
            card(Four, Diamonds),
            card(Five, Hearts),
            card(Nine, Spades),
            card(Ten, Clubs),
        ];
        let low = O8Evaluator::evaluate_low(&hole, &board);
        assert!(low.is_some(), "Wheel A-2-3-4-5 qualifies");
        let rank = low.unwrap();
        let wheel_rank = AceToFiveEvaluator.evaluate_5(&[
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Clubs),
            card(Four, Diamonds),
            card(Five, Hearts),
        ]);
        assert_eq!(rank, wheel_rank, "Best low is wheel");
    }

    #[test]
    fn no_low_when_board_has_no_three_low_cards() {
        let hole = [
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
            card(Four, Spades),
        ];
        let board = [
            card(King, Clubs),
            card(Queen, Diamonds),
            card(Jack, Hearts),
            card(Ten, Spades),
            card(Nine, Clubs),
        ];
        let low = O8Evaluator::evaluate_low(&hole, &board);
        assert!(low.is_none(), "Board has no 3 cards 8-or-better - no qualifying low");
    }

    #[test]
    fn eight_or_better_qualifier() {
        let hole = [
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
            card(Eight, Spades),
        ];
        let board = [
            card(Three, Clubs),
            card(Four, Diamonds),
            card(Five, Hearts),
            card(Six, Spades),
            card(King, Clubs),
        ];
        let low = O8Evaluator::evaluate_low(&hole, &board);
        assert!(low.is_some(), "A-2-3-4-5-6-7-8 all qualify");
        let wheel = AceToFiveEvaluator.evaluate_5(&[
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
            card(Four, Diamonds),
            card(Five, Hearts),
        ]);
        assert_eq!(low.unwrap(), wheel, "Use A 2 from hole + 3 4 5 from board = wheel");
    }

    #[test]
    fn low_with_three_board_cards_eight_or_better() {
        let hole = [
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
            card(Four, Spades),
        ];
        let board = [
            card(Five, Clubs),
            card(Six, Diamonds),
            card(Seven, Hearts),
            card(King, Spades),
            card(Queen, Clubs),
        ];
        let low = O8Evaluator::evaluate_low(&hole, &board);
        assert!(low.is_some(), "Board has 5,6,7 - use A,2 from hole + 5,6,7 from board = A-2-5-6-7");
    }

    #[test]
    fn truly_no_qualifying_low() {
        let hole = [
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
            card(Four, Spades),
        ];
        let board = [
            card(King, Clubs),
            card(Queen, Diamonds),
            card(Jack, Hearts),
            card(Ten, Spades),
            card(Nine, Clubs),
        ];
        let low = O8Evaluator::evaluate_low(&hole, &board);
        assert!(low.is_none(), "Board has no cards 8-or-better - cannot make low");
    }

    #[test]
    fn high_hand_evaluated_correctly() {
        let hole = [
            card(Ace, Clubs),
            card(Ace, Diamonds),
            card(Two, Hearts),
            card(Three, Spades),
        ];
        let board = [
            card(Ace, Hearts),
            card(Ace, Spades),
            card(King, Clubs),
            card(Queen, Diamonds),
            card(Jack, Hearts),
        ];
        let high = O8Evaluator::evaluate_high(&hole, &board);
        let quad_aces = crate::StandardEvaluator.evaluate_5(&[
            card(Ace, Clubs),
            card(Ace, Diamonds),
            card(Ace, Hearts),
            card(Ace, Spades),
            card(King, Clubs),
        ]);
        assert_eq!(high, quad_aces);
    }
}
