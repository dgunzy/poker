//! Foundational types for poker simulation: cards, deck, and hand.
//!
//! Cards use compact u8 encoding (0-51). Rank and suit extractable via bit operations.

mod card;
mod deck;
mod hand;

pub use card::{Card, Rank, Suit};
pub use deck::Deck;
pub use hand::Hand;
