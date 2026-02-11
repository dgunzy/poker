//! Texas Hold'em evaluator. 2 hole cards + 5 board, best 5 from 7.
//! Standard high hand rankings.

use poker_core::Card;

use crate::{StandardEvaluator, Evaluator};

/// Texas Hold'em evaluator. Lower rank = stronger hand.
#[derive(Debug, Default, Clone, Copy)]
pub struct HoldemEvaluator;

impl HoldemEvaluator {
    fn choose5_indices() -> Vec<[usize; 5]> {
        let mut out = Vec::new();
        for a in 0..7 {
            for b in (a + 1)..7 {
                for c in (b + 1)..7 {
                    for d in (c + 1)..7 {
                        for e in (d + 1)..7 {
                            out.push([a, b, c, d, e]);
                        }
                    }
                }
            }
        }
        out
    }

    /// Best 5-card high hand from 2 hole + 5 board (7 cards). Returns (rank, best_hand).
    pub fn evaluate_holdem_with_hand(hole: &[Card; 2], board: &[Card; 5]) -> (u32, [Card; 5]) {
        let all: [Card; 7] = [hole[0], hole[1], board[0], board[1], board[2], board[3], board[4]];
        let combos = Self::choose5_indices();

        let mut best_rank = u32::MAX;
        let mut best_hand = [all[0], all[1], all[2], all[3], all[4]];
        for &[i, j, k, l, m] in &combos {
            let hand = [all[i], all[j], all[k], all[l], all[m]];
            let rank = StandardEvaluator.evaluate_5(&hand);
            if rank < best_rank {
                best_rank = rank;
                best_hand = hand;
            }
        }
        (best_rank, best_hand)
    }

    /// Best 5-card high hand rank from 2 hole + 5 board.
    pub fn evaluate_holdem(hole: &[Card; 2], board: &[Card; 5]) -> u32 {
        Self::evaluate_holdem_with_hand(hole, board).0
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
    fn royal_flush_from_hole_and_board() {
        let hole = [card(Ace, Clubs), card(King, Clubs)];
        let board = [
            card(Queen, Clubs),
            card(Jack, Clubs),
            card(Ten, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
        ];
        let rank = HoldemEvaluator::evaluate_holdem(&hole, &board);
        let royal = StandardEvaluator.evaluate_5(&[
            card(Ace, Clubs),
            card(King, Clubs),
            card(Queen, Clubs),
            card(Jack, Clubs),
            card(Ten, Clubs),
        ]);
        assert_eq!(rank, royal);
    }

    #[test]
    fn uses_best_five_from_seven() {
        let hole = [card(Ace, Clubs), card(Ace, Diamonds)];
        let board = [
            card(Ace, Hearts),
            card(Ace, Spades),
            card(King, Clubs),
            card(Queen, Diamonds),
            card(Jack, Hearts),
        ];
        let rank = HoldemEvaluator::evaluate_holdem(&hole, &board);
        let quads = StandardEvaluator.evaluate_5(&[
            card(Ace, Clubs),
            card(Ace, Diamonds),
            card(Ace, Hearts),
            card(Ace, Spades),
            card(King, Clubs),
        ]);
        assert_eq!(rank, quads);
    }
}
