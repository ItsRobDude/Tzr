pub mod events;
pub mod pathfinding;
pub mod tick;

use crate::error::SimError;
use crate::model::{DungeonState, SimulationOutcome, SimulationResult, SimulationStats, WaveConfig};

pub const MAX_UNITS: usize = 512;
pub const MAX_TICKS: u32 = 60_000;
pub const MAX_EVENTS: usize = 10_000;

/// Simulate a wave against the provided dungeon layout.
///
/// This placeholder returns a timeout outcome until the simulation
/// logic is implemented in later steps.
pub fn simulate_wave(
    dungeon: DungeonState,
    _wave: WaveConfig,
    _seed: u64,
    max_ticks: u32,
) -> Result<SimulationResult, SimError> {
    let stats = SimulationStats {
        ticks_run: max_ticks.min(MAX_TICKS),
        heroes_spawned: 0,
        heroes_killed: 0,
        monsters_killed: 0,
        total_damage_to_core: 0,
    };

    Ok(SimulationResult {
        outcome: SimulationOutcome::Timeout,
        final_dungeon: dungeon,
        final_heroes: Vec::new(),
        stats,
        events: Vec::new(),
    })
}
