//! Fixed-size hand type with compile-time validation.

use crate::card::Card;

/// A hand of exactly `N` cards. Used for game variants with fixed hand sizes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hand<const N: usize> {
    cards: [Card; N],
}

impl<const N: usize> Hand<N> {
    pub fn new(cards: [Card; N]) -> Self {
        Self { cards }
    }

    pub fn from_slice(slice: &[Card]) -> Option<Self> {
        if slice.len() != N {
            return None;
        }
        let mut arr = [Card::from_index(0).unwrap(); N];
        arr.copy_from_slice(slice);
        Some(Self { cards: arr })
    }

    #[inline]
    pub fn cards(&self) -> &[Card; N] {
        &self.cards
    }

    #[inline]
    pub fn as_slice(&self) -> &[Card] {
        &self.cards
    }

    /// Check if this hand contains any duplicate cards.
    pub fn has_duplicates(&self) -> bool {
        for i in 0..N {
            for j in (i + 1)..N {
                if self.cards[i] == self.cards[j] {
                    return true;
                }
            }
        }
        false
    }

    /// Check if this hand contains the given card.
    pub fn contains(&self, card: Card) -> bool {
        self.cards.iter().any(|&c| c == card)
    }
}

impl<const N: usize> AsRef<[Card]> for Hand<N> {
    fn as_ref(&self) -> &[Card] {
        &self.cards
    }
}

impl<const N: usize> std::fmt::Display for Hand<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, c) in self.cards.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}
