pub mod model;
pub mod sim;
pub mod rng;
pub mod error;

pub use sim::simulate_wave;
pub use model::{DungeonState, WaveConfig, SimulationResult};
