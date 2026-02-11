//! Card representation with compact encoding (0-51).
//! Bits 0-3: rank (0=2, 1=3, ..., 12=A)
//! Bits 4-5: suit (0=clubs, 1=diamonds, 2=hearts, 3=spades)

use serde::{Deserialize, Serialize};

/// Suit of a playing card. Canonical ordering for evaluator optimizations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    #[inline]
    pub fn from_index(i: u8) -> Option<Suit> {
        match i {
            0 => Some(Suit::Clubs),
            1 => Some(Suit::Diamonds),
            2 => Some(Suit::Hearts),
            3 => Some(Suit::Spades),
            _ => None,
        }
    }

    #[inline]
    pub fn index(self) -> u8 {
        self as u8
    }
}

/// Rank of a playing card. 2=0 up to A=12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    Two = 0,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];

    #[inline]
    pub fn from_index(i: u8) -> Option<Rank> {
        match i {
            0 => Some(Rank::Two),
            1 => Some(Rank::Three),
            2 => Some(Rank::Four),
            3 => Some(Rank::Five),
            4 => Some(Rank::Six),
            5 => Some(Rank::Seven),
            6 => Some(Rank::Eight),
            7 => Some(Rank::Nine),
            8 => Some(Rank::Ten),
            9 => Some(Rank::Jack),
            10 => Some(Rank::Queen),
            11 => Some(Rank::King),
            12 => Some(Rank::Ace),
            _ => None,
        }
    }

    #[inline]
    pub fn index(self) -> u8 {
        self as u8
    }

    /// Numeric value for lowball: 2=0 (best), A=12 (worst)
    #[inline]
    pub fn lowball_value(self) -> u8 {
        self.index()
    }
}

/// A playing card with compact u8 encoding (0-51).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card(u8);

impl Card {
    /// Maximum valid card value (51 = Ace of Spades)
    pub const MAX: u8 = 51;

    #[inline]
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Card((rank.index() << 2) | suit.index())
    }

    #[inline]
    pub fn from_index(idx: u8) -> Option<Card> {
        if idx <= Self::MAX {
            Some(Card(idx))
        } else {
            None
        }
    }

    #[inline]
    pub fn index(self) -> u8 {
        self.0
    }

    #[inline]
    pub fn rank(self) -> Rank {
        Rank::from_index(self.0 >> 2).unwrap()
    }

    #[inline]
    pub fn suit(self) -> Suit {
        Suit::from_index(self.0 & 3).unwrap()
    }

    /// All 52 cards in canonical order (2c, 2d, 2h, 2s, 3c, ...)
    pub fn deck() -> impl Iterator<Item = Card> {
        (0..=Self::MAX).map(|i| Card(i))
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self.rank() {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "T",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        };
        let s = match self.suit() {
            Suit::Clubs => "c",
            Suit::Diamonds => "d",
            Suit::Hearts => "h",
            Suit::Spades => "s",
        };
        write!(f, "{}{}", r, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_roundtrip() {
        for r in Rank::ALL {
            for s in Suit::ALL {
                let c = Card::new(r, s);
                assert_eq!(c.rank(), r);
                assert_eq!(c.suit(), s);
            }
        }
    }

    #[test]
    fn deck_has_52_cards() {
        assert_eq!(Card::deck().count(), 52);
    }
}
