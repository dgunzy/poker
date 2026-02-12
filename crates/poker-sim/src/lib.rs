//! Simulation engine: draw cards, evaluate hands, produce ranked results with percentile data.

mod simulation;
pub mod training;

pub use simulation::{SimulationInput, SimulationResult, Simulator, SimulatorError};
