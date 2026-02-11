//! Core evaluator trait: given cards, produce a numeric rank (lower = stronger).

use poker_core::Card;

/// Evaluator for a poker variant. Produces a numeric rank where lower = stronger hand.
pub trait Evaluator {
    /// Evaluate a 5-card hand. Lower rank = better hand.
    fn evaluate_5(&self, cards: &[Card; 5]) -> u32;
}
