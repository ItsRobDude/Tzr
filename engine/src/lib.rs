pub mod error;
pub mod model;
pub mod rng;
pub mod sim;

pub use model::{DungeonState, SimulationResult, WaveConfig};
pub use sim::simulate_wave;
