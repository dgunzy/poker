//! Deck with support for dead cards and Fisher-Yates shuffle.

use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::card::Card;

/// A deck that tracks removed (dead) cards and supports drawing from the remainder.
#[derive(Debug, Clone)]
pub struct Deck {
    /// Cards available to draw (excluding held + dead)
    cards: Vec<Card>,
    /// RNG for shuffling
    rng: rand::rngs::StdRng,
}

impl Deck {
    /// Create a full deck, then remove the given cards.
    pub fn new_excluding(excluded: &[Card]) -> Self {
        let excluded_set: std::collections::HashSet<Card> = excluded.iter().copied().collect();
        let cards: Vec<Card> = Card::deck().filter(|c| !excluded_set.contains(c)).collect();
        Self {
            cards,
            rng: rand::SeedableRng::from_entropy(),
        }
    }

    /// Create a full 52-card deck.
    pub fn full() -> Self {
        Self::new_excluding(&[])
    }

    /// Set seed for reproducible shuffles.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng = rand::rngs::StdRng::seed_from_u64(seed);
        self
    }

    /// Shuffle the remaining cards (Fisher-Yates).
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut self.rng);
    }

    /// Draw up to `n` cards. Returns fewer if not enough remain.
    pub fn draw(&mut self, n: usize) -> Vec<Card> {
        let k = n.min(self.cards.len());
        self.cards.drain(..k).collect()
    }

    /// Number of cards remaining.
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rank::*;
    use crate::Suit::*;

    #[test]
    fn deck_excludes_cards() {
        let c1 = Card::new(Two, Clubs);
        let c2 = Card::new(Three, Diamonds);
        let mut deck = Deck::new_excluding(&[c1, c2]);
        deck.shuffle();
        assert_eq!(deck.len(), 50);
        let drawn: Vec<Card> = deck.draw(52);
        assert!(!drawn.contains(&c1));
        assert!(!drawn.contains(&c2));
    }

    #[test]
    fn deck_seeded_reproducible() {
        let mut d1 = Deck::full().with_seed(42);
        d1.shuffle();
        let a: Vec<Card> = d1.draw(5);

        let mut d2 = Deck::full().with_seed(42);
        d2.shuffle();
        let b: Vec<Card> = d2.draw(5);

        assert_eq!(a, b);
    }
}
