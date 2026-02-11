//! Pluggable hand evaluation engines. Each poker variant implements the Evaluator trait.

mod ace_to_five;
mod deuce_to_seven;
mod evaluator;
mod holdem;
mod omaha;
mod omaha8;
mod standard_high;

pub use ace_to_five::AceToFiveEvaluator;
pub use deuce_to_seven::DeuceToSevenEvaluator;
pub use evaluator::Evaluator;
pub use holdem::HoldemEvaluator;
pub use omaha::OmahaEvaluator;
pub use omaha8::O8Evaluator;
pub use standard_high::StandardEvaluator;

/// Game variant identifier for selecting evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameVariant {
    #[default]
    DeuceToSeven, // Aces high, straights/flushes bad
    AceToFive,    // Aces low, straights/flushes ignored (Razz, A-5 lowball)
    Holdem,       // 2 hole + 5 board, standard high
    Omaha,        // 4 hole + 5 board, standard high (PLO)
    Omaha8,       // 4 hole + 5 board, high + low (8-or-better)
}

impl GameVariant {
    pub fn from_game_id(id: &str) -> Self {
        match id {
            "ace_to_five" => GameVariant::AceToFive,
            "holdem" => GameVariant::Holdem,
            "omaha" => GameVariant::Omaha,
            "omaha8" => GameVariant::Omaha8,
            _ => GameVariant::DeuceToSeven,
        }
    }

    pub fn is_holdem_style(&self) -> bool {
        matches!(self, GameVariant::Holdem)
    }

    pub fn is_omaha_style(&self) -> bool {
        matches!(self, GameVariant::Omaha | GameVariant::Omaha8)
    }

    pub fn is_board_game(&self) -> bool {
        self.is_holdem_style() || self.is_omaha_style()
    }
}
