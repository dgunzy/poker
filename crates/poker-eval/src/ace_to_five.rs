//! A-5 Lowball (Ace-to-Five) evaluator.
//! Used by Razz and A-5 draw. Aces are LOW. Straights and flushes are IGNORED (don't hurt).
//! Best hand: A-2-3-4-5 (the wheel).

use poker_core::{Card, Rank};

use crate::Evaluator;

/// Reference evaluator for A-5 lowball. Lower rank = better hand.
#[derive(Debug, Default, Clone, Copy)]
pub struct AceToFiveEvaluator;

impl AceToFiveEvaluator {
    /// Ace-low value: A=0, 2=1, 3=2, ..., K=12
    pub(crate) fn ace_low_value(rank: Rank) -> u8 {
        match rank {
            Rank::Ace => 0,
            Rank::Two => 1,
            Rank::Three => 2,
            Rank::Four => 3,
            Rank::Five => 4,
            Rank::Six => 5,
            Rank::Seven => 6,
            Rank::Eight => 7,
            Rank::Nine => 8,
            Rank::Ten => 9,
            Rank::Jack => 10,
            Rank::Queen => 11,
            Rank::King => 12,
        }
    }

    pub(crate) fn rank_counts(cards: &[Card; 5]) -> [u8; 13] {
        let mut counts = [0u8; 13];
        for c in cards {
            counts[c.rank().index() as usize] += 1;
        }
        counts
    }

    fn has_pair_or_worse(cards: &[Card; 5]) -> bool {
        let counts = Self::rank_counts(cards);
        counts.iter().any(|&c| c >= 2)
    }

    /// Evaluate: pairs/better are bad (high rank). Otherwise rank by low cards (ace low).
    fn eval_hand(cards: &[Card; 5]) -> u32 {
        if Self::has_pair_or_worse(cards) {
            let counts = Self::rank_counts(cards);
            let max_count = *counts.iter().max().unwrap();
            let pair_count = counts.iter().filter(|&&c| c >= 2).count();
            let mut rank = 0u32;
            if max_count == 4 {
                rank = 1_000_000;
            } else if max_count == 3 && pair_count >= 2 {
                rank = 800_000;
            } else if max_count == 3 {
                rank = 500_000;
            } else if pair_count == 2 {
                rank = 400_000;
            } else {
                rank = 300_000;
            }
            let mut sorted: Vec<u8> = cards.iter().map(|c| Self::ace_low_value(c.rank())).collect();
            sorted.sort_by(|a, b| b.cmp(a));
            let tb: u32 = sorted.iter().enumerate().map(|(i, &r)| (r as u32) << (4 * (4 - i))).sum();
            return rank + tb;
        }

        let mut values: Vec<u8> = cards.iter().map(|c| Self::ace_low_value(c.rank())).collect();
        values.sort_by(|a, b| b.cmp(a));
        values
            .iter()
            .enumerate()
            .map(|(i, &r)| (r as u32) * 13u32.pow(4 - i as u32))
            .sum()
    }
}

impl Evaluator for AceToFiveEvaluator {
    fn evaluate_5(&self, cards: &[Card; 5]) -> u32 {
        Self::eval_hand(cards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use poker_core::{Rank::*, Suit, Suit::*};

    fn card(r: Rank, s: Suit) -> Card {
        Card::new(r, s)
    }

    #[test]
    fn wheel_is_nuts() {
        let wheel = [
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
            card(Four, Spades),
            card(Five, Clubs),
        ];
        let rank_wheel = AceToFiveEvaluator.evaluate_5(&wheel);
        let six_four = [
            card(Six, Clubs),
            card(Four, Diamonds),
            card(Three, Hearts),
            card(Two, Spades),
            card(Ace, Clubs),
        ];
        let rank_six = AceToFiveEvaluator.evaluate_5(&six_four);
        assert!(rank_wheel < rank_six, "wheel A-2-3-4-5 should beat 6-4-3-2-A");
    }

    #[test]
    fn straights_dont_hurt() {
        let wheel = [
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
            card(Four, Spades),
            card(Five, Clubs),
        ];
        let rank_wheel = AceToFiveEvaluator.evaluate_5(&wheel);
        let six_high = [
            card(Six, Clubs),
            card(Five, Diamonds),
            card(Four, Hearts),
            card(Three, Spades),
            card(Two, Clubs),
        ];
        let rank_six = AceToFiveEvaluator.evaluate_5(&six_high);
        assert!(rank_wheel < rank_six, "wheel should beat 6-5-4-3-2 (straight doesn't hurt)");
    }

    #[test]
    fn flushes_dont_hurt() {
        let wheel = [
            card(Ace, Clubs),
            card(Two, Clubs),
            card(Three, Clubs),
            card(Four, Clubs),
            card(Five, Clubs),
        ];
        let rank_flush_wheel = AceToFiveEvaluator.evaluate_5(&wheel);
        let six_four = [
            card(Six, Clubs),
            card(Four, Diamonds),
            card(Three, Hearts),
            card(Two, Spades),
            card(Ace, Clubs),
        ];
        let rank_six = AceToFiveEvaluator.evaluate_5(&six_four);
        assert!(
            rank_flush_wheel < rank_six,
            "flush wheel A-2-3-4-5 same suit should beat 6-4-3-2-A (flush doesn't hurt)"
        );
    }

    #[test]
    fn pair_loses_to_any_no_pair() {
        let pair = [
            card(Ace, Clubs),
            card(Ace, Diamonds),
            card(Two, Hearts),
            card(Three, Spades),
            card(Four, Clubs),
        ];
        let rank_pair = AceToFiveEvaluator.evaluate_5(&pair);
        let bad_low = [
            card(King, Clubs),
            card(Queen, Diamonds),
            card(Jack, Hearts),
            card(Ten, Spades),
            card(Nine, Clubs),
        ];
        let rank_bad = AceToFiveEvaluator.evaluate_5(&bad_low);
        assert!(rank_bad < rank_pair, "King-high no-pair beats any pair");
    }

    #[test]
    fn aces_are_low() {
        let wheel = [
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
            card(Four, Spades),
            card(Five, Clubs),
        ];
        let rank_wheel = AceToFiveEvaluator.evaluate_5(&wheel);
        let two_six = [
            card(Two, Clubs),
            card(Three, Diamonds),
            card(Four, Hearts),
            card(Five, Spades),
            card(Six, Clubs),
        ];
        let rank_two_six = AceToFiveEvaluator.evaluate_5(&two_six);
        assert!(rank_wheel < rank_two_six, "A-2-3-4-5 beats 2-3-4-5-6 (ace is low)");
    }
}
