//! Standard high hand evaluator for 5-card poker.
//! Used by Omaha high. Aces high, except A-2-3-4-5 (wheel) is the lowest straight.

use poker_core::{Card, Rank};

use crate::Evaluator;

/// Standard poker hand rankings: Royal flush > Straight flush > 4K > Full house > Flush > Straight > 3K > Two pair > Pair > High.
/// Lower returned value = stronger hand.
#[derive(Debug, Default, Clone, Copy)]
pub struct StandardEvaluator;

impl StandardEvaluator {
    /// High-card value: 2=0, 3=1, ..., A=12
    fn rank_value(r: Rank) -> u8 {
        r.index()
    }

    fn rank_counts(cards: &[Card; 5]) -> [u8; 13] {
        let mut counts = [0u8; 13];
        for c in cards {
            counts[c.rank().index() as usize] += 1;
        }
        counts
    }

    fn suit_counts(cards: &[Card; 5]) -> [u8; 4] {
        let mut counts = [0u8; 4];
        for c in cards {
            counts[c.suit().index() as usize] += 1;
        }
        counts
    }

    fn is_flush(cards: &[Card; 5]) -> bool {
        Self::suit_counts(cards).iter().any(|&c| c >= 5)
    }

    fn sorted_ranks_high_to_low(cards: &[Card; 5]) -> [u8; 5] {
        let mut r: Vec<u8> = cards.iter().map(|c| Self::rank_value(c.rank())).collect();
        r.sort_by(|a, b| b.cmp(a));
        r.try_into().unwrap()
    }

    /// Is it a straight? Returns high_card. Wheel (A-2-3-4-5) returns 3 (5 is high).
    fn is_straight(cards: &[Card; 5]) -> Option<u8> {
        let mut r: Vec<u8> = cards.iter().map(|c| Self::rank_value(c.rank())).collect();
        r.sort_by(|a, b| b.cmp(a));
        let mut vals: [u8; 5] = r.try_into().unwrap();

        // Check for wheel: A-2-3-4-5 (A=12, 2=0, 3=1, 4=2, 5=3)
        let has_ace = vals.contains(&12);
        let has_two = vals.contains(&0);
        let has_three = vals.contains(&1);
        let has_four = vals.contains(&2);
        let has_five = vals.contains(&3);
        if has_ace && has_two && has_three && has_four && has_five {
            return Some(3); // 5-high straight (wheel)
        }

        vals.sort_by(|a, b| b.cmp(a));
        for i in 0..4 {
            if vals[i].saturating_sub(1) != vals[i + 1] {
                return None;
            }
        }
        Some(vals[0])
    }

    fn eval_hand(cards: &[Card; 5]) -> u32 {
        let counts = Self::rank_counts(cards);
        let flush = Self::is_flush(cards);
        let straight_high = Self::is_straight(cards);
        let sorted = Self::sorted_ranks_high_to_low(cards);

        let (groups, kickers) = Self::group_by_count(&counts);

        // Straight flush
        if flush && straight_high.is_some() {
            let sh = straight_high.unwrap();
            if sh == 12 {
                return 0; // Royal flush
            }
            return 1 + (12 - sh) as u32; // 1 = K-high SF, 2 = Q-high, ...
        }

        // Four of a kind (lower = better, so higher quad rank = lower value)
        if groups[0].0 == 4 {
            let quad = groups[0].1;
            let kicker = kickers[0];
            return 80_000 + (12 - quad) as u32 * 100 + (12 - kicker) as u32;
        }

        // Full house
        if groups[0].0 == 3 && groups.len() >= 2 && groups[1].0 >= 2 {
            let trip = groups[0].1;
            let pair = groups[1].1;
            return 90_000 + (12 - trip) as u32 * 100 + (12 - pair) as u32;
        }

        // Flush
        if flush {
            let r: u32 = sorted.iter().enumerate().map(|(i, &v)| ((12 - v) as u32) << (4 * (4 - i))).sum();
            return 100_000 + r;
        }

        // Straight
        if let Some(sh) = straight_high {
            return 110_000 + (12 - sh) as u32;
        }

        // Three of a kind
        if groups[0].0 == 3 {
            let trip = groups[0].1;
            let k1 = kickers[0];
            let k2 = kickers[1];
            return 120_000 + (12 - trip) as u32 * 1000 + (12 - k1) as u32 * 100 + (12 - k2) as u32;
        }

        // Two pair
        if groups.len() >= 2 && groups[0].0 == 2 && groups[1].0 == 2 {
            let p1 = groups[0].1.max(groups[1].1);
            let p2 = groups[0].1.min(groups[1].1);
            let kicker = kickers[0];
            return 130_000 + (12 - p1) as u32 * 1000 + (12 - p2) as u32 * 100 + (12 - kicker) as u32;
        }

        // One pair
        if groups[0].0 == 2 {
            let pair = groups[0].1;
            let k1 = kickers[0];
            let k2 = kickers[1];
            let k3 = kickers[2];
            return 140_000 + (12 - pair) as u32 * 10000 + (12 - k1) as u32 * 1000 + (12 - k2) as u32 * 100 + (12 - k3) as u32;
        }

        // High card
        let r: u32 = sorted.iter().enumerate().map(|(i, &v)| ((12 - v) as u32) << (4 * (4 - i))).sum();
        150_000 + r
    }

    /// Returns (count, rank) sorted by count desc, then rank desc. Kickers = ranks with count 1, sorted high to low.
    fn group_by_count(counts: &[u8; 13]) -> (Vec<(u8, u8)>, Vec<u8>) {
        let mut groups: Vec<(u8, u8)> = counts
            .iter()
            .enumerate()
            .filter(|(_, &c)| c > 0)
            .map(|(r, &c)| (c, r as u8))
            .collect();
        groups.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));

        let mut kickers: Vec<u8> = groups.iter().filter(|(c, _)| *c == 1).map(|(_, r)| *r).collect();
        kickers.sort_by(|a, b| b.cmp(a));

        (groups, kickers)
    }
}

impl Evaluator for StandardEvaluator {
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
    fn royal_flush_is_nuts() {
        let royal = [
            card(Ace, Clubs),
            card(King, Clubs),
            card(Queen, Clubs),
            card(Jack, Clubs),
            card(Ten, Clubs),
        ];
        let sf = [
            card(King, Clubs),
            card(Queen, Clubs),
            card(Jack, Clubs),
            card(Ten, Clubs),
            card(Nine, Clubs),
        ];
        assert!(StandardEvaluator.evaluate_5(&royal) < StandardEvaluator.evaluate_5(&sf));
    }

    #[test]
    fn wheel_is_lowest_straight() {
        let wheel = [
            card(Ace, Clubs),
            card(Two, Diamonds),
            card(Three, Hearts),
            card(Four, Spades),
            card(Five, Clubs),
        ];
        let six_high = [
            card(Six, Clubs),
            card(Five, Diamonds),
            card(Four, Hearts),
            card(Three, Spades),
            card(Two, Clubs),
        ];
        let rank_wheel = StandardEvaluator.evaluate_5(&wheel);
        let rank_six = StandardEvaluator.evaluate_5(&six_high);
        assert!(rank_wheel > rank_six, "6-high straight beats wheel (wheel is weaker)");
    }

    #[test]
    fn aces_high_in_pairs() {
        let aces = [
            card(Ace, Clubs),
            card(Ace, Diamonds),
            card(King, Hearts),
            card(Queen, Spades),
            card(Jack, Clubs),
        ];
        let kings = [
            card(King, Clubs),
            card(King, Diamonds),
            card(Ace, Hearts),
            card(Queen, Spades),
            card(Jack, Clubs),
        ];
        assert!(StandardEvaluator.evaluate_5(&aces) < StandardEvaluator.evaluate_5(&kings));
    }

    #[test]
    fn full_house_beats_flush() {
        let fh = [
            card(Ace, Clubs),
            card(Ace, Diamonds),
            card(Ace, Hearts),
            card(Two, Spades),
            card(Two, Clubs),
        ];
        let flush = [
            card(Ace, Clubs),
            card(King, Clubs),
            card(Queen, Clubs),
            card(Jack, Clubs),
            card(Nine, Clubs),
        ];
        assert!(StandardEvaluator.evaluate_5(&fh) < StandardEvaluator.evaluate_5(&flush));
    }

    #[test]
    fn four_of_a_kind_beats_full_house() {
        let quads = [
            card(Ace, Clubs),
            card(Ace, Diamonds),
            card(Ace, Hearts),
            card(Ace, Spades),
            card(Two, Clubs),
        ];
        let fh = [
            card(King, Clubs),
            card(King, Diamonds),
            card(King, Hearts),
            card(Two, Spades),
            card(Two, Clubs),
        ];
        assert!(StandardEvaluator.evaluate_5(&quads) < StandardEvaluator.evaluate_5(&fh));
    }

    #[test]
    fn straight_beats_three_of_a_kind() {
        let straight = [
            card(Nine, Clubs),
            card(Eight, Diamonds),
            card(Seven, Hearts),
            card(Six, Spades),
            card(Five, Clubs),
        ];
        let trips = [
            card(Ace, Clubs),
            card(Ace, Diamonds),
            card(Ace, Hearts),
            card(King, Spades),
            card(Queen, Clubs),
        ];
        assert!(StandardEvaluator.evaluate_5(&straight) < StandardEvaluator.evaluate_5(&trips));
    }
}
