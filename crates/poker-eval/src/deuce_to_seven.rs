//! NL 2-7 Lowball (Deuce-to-Seven) evaluator.
//! Aces high, straights and flushes count against you. Best hand: 7-5-4-3-2 with 2+ suits.

use poker_core::{Card, Hand, Rank};

use crate::Evaluator;

/// Reference evaluator for 2-7 lowball. Lower rank = better hand.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeuceToSevenEvaluator;

impl DeuceToSevenEvaluator {
    /// Check if hand is a flush (all same suit).
    fn is_flush(cards: &[Card; 5]) -> bool {
        let s = cards[0].suit();
        cards[1..].iter().all(|c| c.suit() == s)
    }

    /// Check if hand is a straight. In 2-7 lowball, A is always high, so A-2-3-4-5 is NOT a straight.
    fn is_straight(cards: &[Card; 5]) -> bool {
        let mut ranks: Vec<u8> = cards.iter().map(|c| c.rank().index()).collect();
        ranks.sort_unstable();
        ranks.windows(2).all(|w| w[1] == w[0] + 1)
    }

    /// Count ranks for pairs/trips/etc.
    fn rank_counts(cards: &[Card; 5]) -> [u8; 13] {
        let mut counts = [0u8; 13];
        for c in cards {
            counts[c.rank().index() as usize] += 1;
        }
        counts
    }

    /// Check for pairs or better.
    fn has_pair_or_worse(cards: &[Card; 5]) -> bool {
        let counts = Self::rank_counts(cards);
        counts.iter().any(|&c| c >= 2)
    }

    /// Primary rank: lower is better. We encode hand strength so that:
    /// - Pairs/straights/flushes get high values (bad)
    /// - Unpaired, non-straight, non-flush get low values (good)
    /// - Tiebreaker: high cards first (so 7-5-4-3-2 beats 8-5-4-3-2)
    fn eval_hand(cards: &[Card; 5]) -> u32 {
        let is_flush = Self::is_flush(cards);
        let is_straight = Self::is_straight(cards);
        let has_pair = Self::has_pair_or_worse(cards);

        // In 2-7 lowball, these are bad
        if has_pair || is_straight || is_flush {
            // Assign high ranks to bad hands. We need a consistent ordering.
            // Pair < two pair < trips < straight < flush < full house < quads < straight flush
            let mut rank = 0u32;
            let counts = Self::rank_counts(cards);
            let max_count = *counts.iter().max().unwrap();
            let pair_count = counts.iter().filter(|&&c| c >= 2).count();

            if max_count == 4 {
                rank = 1_000_000;
            } else if is_straight && is_flush {
                rank = 900_000;
            } else if max_count == 3 && pair_count >= 2 {
                rank = 800_000; // full house
            } else if is_flush {
                rank = 700_000;
            } else if is_straight {
                rank = 600_000;
            } else if max_count == 3 {
                rank = 500_000;
            } else if pair_count == 2 {
                rank = 400_000; // two pair
            } else {
                rank = 300_000; // one pair
            }
            // Add tiebreaker: high cards contribute
            let mut sorted: Vec<u8> = cards.iter().map(|c| c.rank().index()).collect();
            sorted.sort_by(|a, b| b.cmp(a)); // high to low
            let tb: u32 = sorted.iter().enumerate().map(|(i, &r)| (r as u32) << (4 * (4 - i))).sum();
            return rank + tb;
        }

        // Good hand: unpaired, no straight, no flush. Rank by high cards (high to low matters).
        let mut sorted: Vec<u8> = cards.iter().map(|c| c.rank().index()).collect();
        sorted.sort_by(|a, b| b.cmp(a));
        // Encode as base-13 number: highest card most significant
        sorted
            .iter()
            .enumerate()
            .map(|(i, &r)| (r as u32) * 13u32.pow(4 - i as u32))
            .sum()
    }
}

impl Evaluator for DeuceToSevenEvaluator {
    fn evaluate_5(&self, cards: &[Card; 5]) -> u32 {
        Self::eval_hand(cards)
    }
}

/// Evaluate a Hand<5> for 2-7 lowball.
pub fn evaluate_hand_5(hand: &Hand<5>) -> u32 {
    DeuceToSevenEvaluator.evaluate_5(hand.cards())
}

#[cfg(test)]
mod tests {
    use super::*;
    use poker_core::{Rank::*, Suit, Suit::*};

    fn card(r: Rank, s: Suit) -> Card {
        Card::new(r, s)
    }

    #[test]
    fn best_hand_ranks_lowest() {
        // 7-5-4-3-2 with 2+ suits is the nuts
        let nut_low = [card(Seven, Clubs), card(Five, Diamonds), card(Four, Hearts), card(Three, Spades), card(Two, Clubs)];
        let rank_nut = DeuceToSevenEvaluator.evaluate_5(&nut_low);
        // 8-5-4-3-2 is second best
        let second = [card(Eight, Clubs), card(Five, Diamonds), card(Four, Hearts), card(Three, Spades), card(Two, Clubs)];
        let rank_second = DeuceToSevenEvaluator.evaluate_5(&second);
        assert!(rank_nut < rank_second);
    }

    #[test]
    fn pair_loses_to_wheel() {
        let pair = [card(Two, Clubs), card(Two, Diamonds), card(Three, Hearts), card(Four, Spades), card(Five, Clubs)];
        let wheel = [card(Seven, Clubs), card(Five, Diamonds), card(Four, Hearts), card(Three, Spades), card(Two, Clubs)];
        assert!(DeuceToSevenEvaluator.evaluate_5(&wheel) < DeuceToSevenEvaluator.evaluate_5(&pair));
    }

    #[test]
    fn straight_is_bad() {
        let straight = [card(Six, Clubs), card(Five, Diamonds), card(Four, Hearts), card(Three, Spades), card(Two, Clubs)];
        let bad_low = [card(King, Clubs), card(Five, Diamonds), card(Four, Hearts), card(Three, Spades), card(Two, Clubs)];
        assert!(DeuceToSevenEvaluator.evaluate_5(&straight) > DeuceToSevenEvaluator.evaluate_5(&bad_low));
    }

    #[test]
    fn aces_are_high() {
        // A-2-3-4-5 is NOT a straight in 2-7 (A is high). It's a 5-high hand.
        let ace_high = [card(Ace, Clubs), card(Two, Diamonds), card(Three, Hearts), card(Four, Spades), card(Five, Clubs)];
        let seven_low = [card(Seven, Clubs), card(Five, Diamonds), card(Four, Hearts), card(Three, Spades), card(Two, Clubs)];
        assert!(DeuceToSevenEvaluator.evaluate_5(&seven_low) < DeuceToSevenEvaluator.evaluate_5(&ace_high),
            "7-5-4-3-2 beats A-2-3-4-5 (aces high in 2-7)");
    }

    #[test]
    fn flushes_hurt() {
        let flush = [card(Seven, Clubs), card(Five, Clubs), card(Four, Clubs), card(Three, Clubs), card(Two, Clubs)];
        let no_flush = [card(Seven, Clubs), card(Five, Diamonds), card(Four, Hearts), card(Three, Spades), card(Two, Clubs)];
        assert!(DeuceToSevenEvaluator.evaluate_5(&no_flush) < DeuceToSevenEvaluator.evaluate_5(&flush),
            "Same ranks but flush is bad - 7-5-4-3-2 with 2+ suits beats 7-5-4-3-2 flush");
    }

    #[test]
    fn wheel_not_straight_in_2_7() {
        let wheel = [card(Ace, Clubs), card(Two, Diamonds), card(Three, Hearts), card(Four, Spades), card(Five, Clubs)];
        let pair = [card(Two, Clubs), card(Two, Diamonds), card(Three, Hearts), card(Four, Spades), card(Five, Clubs)];
        assert!(DeuceToSevenEvaluator.evaluate_5(&wheel) < DeuceToSevenEvaluator.evaluate_5(&pair),
            "A-2-3-4-5 is not a straight in 2-7, so it beats a pair");
    }
}
