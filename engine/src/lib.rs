pub mod error;
pub mod model;
pub mod rng;
pub mod sim;

pub use model::{DungeonState, SimulationResult, WaveConfig};
pub use sim::simulate_wave;

/// Semantic version of the engine, taken from Cargo.toml
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
