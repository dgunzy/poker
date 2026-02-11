//! Omaha (high) evaluator. 4 hole cards + 5 board, must use exactly 2 from hand + 3 from board.
//! Standard high hand rankings.

use poker_core::Card;

use crate::{StandardEvaluator, Evaluator};

/// Omaha high evaluator. Lower rank = stronger hand.
#[derive(Debug, Default, Clone, Copy)]
pub struct OmahaEvaluator;

impl OmahaEvaluator {
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

    /// Best 5-card high hand from 4 hole + 5 board. Returns (rank, best_hand).
    pub fn evaluate_omaha_with_hand(hole: &[Card; 4], board: &[Card; 5]) -> (u32, [Card; 5]) {
        let hole_indices = [0, 1, 2, 3];
        let board_indices = [0, 1, 2, 3, 4];

        let hole_combos = Self::choose2(&hole_indices);
        let board_combos = Self::choose3(&board_indices);

        let mut best_rank = u32::MAX;
        let mut best_hand = [hole[0], hole[1], board[0], board[1], board[2]];
        for &[h1, h2] in &hole_combos {
            for &[b1, b2, b3] in &board_combos {
                let hand = [hole[h1], hole[h2], board[b1], board[b2], board[b3]];
                let rank = StandardEvaluator.evaluate_5(&hand);
                if rank < best_rank {
                    best_rank = rank;
                    best_hand = hand;
                }
            }
        }
        (best_rank, best_hand)
    }

    /// Best 5-card high hand rank from 4 hole + 5 board.
    pub fn evaluate_omaha(hole: &[Card; 4], board: &[Card; 5]) -> u32 {
        Self::evaluate_omaha_with_hand(hole, board).0
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
    fn uses_exactly_two_from_hand() {
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
        let rank = OmahaEvaluator::evaluate_omaha(&hole, &board);
        let quad_aces = StandardEvaluator.evaluate_5(&[
            card(Ace, Clubs),
            card(Ace, Diamonds),
            card(Ace, Hearts),
            card(Ace, Spades),
            card(King, Clubs),
        ]);
        assert_eq!(rank, quad_aces, "Must use 2 from hand - can make quad aces with Ah As on board");
    }

    #[test]
    fn cant_use_three_aces_from_hand() {
        let hole = [
            card(Ace, Clubs),
            card(Ace, Diamonds),
            card(Ace, Hearts),
            card(Two, Spades),
        ];
        let board = [
            card(Ace, Spades),
            card(King, Clubs),
            card(Queen, Diamonds),
            card(Jack, Hearts),
            card(Ten, Spades),
        ];
        let rank = OmahaEvaluator::evaluate_omaha(&hole, &board);
        let trips_aces = StandardEvaluator.evaluate_5(&[
            card(Ace, Clubs),
            card(Ace, Diamonds),
            card(Ace, Spades),
            card(King, Clubs),
            card(Queen, Diamonds),
        ]);
        assert_eq!(rank, trips_aces, "Use 2 aces from hole + A from board = trips aces + KQ (max 3 aces)");
    }

    #[test]
    fn best_hand_from_60_combinations() {
        let hole = [
            card(Ace, Clubs),
            card(King, Clubs),
            card(Queen, Diamonds),
            card(Jack, Hearts),
        ];
        let board = [
            card(Ten, Clubs),
            card(Jack, Clubs),
            card(Queen, Clubs),
            card(Three, Hearts),
            card(Two, Spades),
        ];
        let rank = OmahaEvaluator::evaluate_omaha(&hole, &board);
        let royal = StandardEvaluator.evaluate_5(&[
            card(Ace, Clubs),
            card(King, Clubs),
            card(Queen, Clubs),
            card(Jack, Clubs),
            card(Ten, Clubs),
        ]);
        assert_eq!(rank, royal, "Use Ac Kc from hole + Tc Jc Qc from board = royal flush");
    }
}
