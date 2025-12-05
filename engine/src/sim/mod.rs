pub mod events;
pub mod pathfinding;
pub mod test_fixtures;
pub mod tick;

use crate::ENGINE_VERSION;
use crate::error::SimError;
use crate::model::{DungeonState, SimulationOutcome, SimulationResult, WaveConfig};
use tick::{SimState, step_tick};

pub const MAX_UNITS: usize = 512;
pub const MAX_TICKS: u32 = 60_000;
pub const MAX_EVENTS: usize = 10_000;

/// Simulate a wave against the provided dungeon layout.
pub fn simulate_wave(
    dungeon: DungeonState,
    wave: WaveConfig,
    seed: u64,
    max_ticks: u32,
) -> Result<SimulationResult, SimError> {
    validate_dungeon(&dungeon)?;
    if max_ticks > MAX_TICKS {
        return Err(SimError::TickLimit);
    }

    let mut state = SimState::new(dungeon, &wave, seed)?;

    let mut outcome = SimulationOutcome::Timeout;
    for _ in 0..max_ticks {
        if let Some(result) = step_tick(&mut state, &wave)? {
            outcome = result;
            break;
        }
    }

    Ok(SimulationResult {
        outcome,
        final_dungeon: state.dungeon,
        final_heroes: state.heroes,
        stats: state.stats,
        events: state.events,
        engine_version: ENGINE_VERSION.to_string(),
    })
}

fn validate_dungeon(dungeon: &DungeonState) -> Result<(), SimError> {
    if dungeon.rooms.is_empty() {
        return Err(SimError::InvalidDungeon("No rooms".into()));
    }
    if !dungeon.rooms.iter().any(|r| r.id == dungeon.core_room_id) {
        return Err(SimError::InvalidDungeon("Missing core room".into()));
    }
    for (a, b) in &dungeon.edges {
        let a_exists = dungeon.rooms.iter().any(|r| r.id == *a);
        let b_exists = dungeon.rooms.iter().any(|r| r.id == *b);
        if !a_exists || !b_exists {
            return Err(SimError::InvalidDungeon(
                "Edge references unknown room".into(),
            ));
        }
    }

    let total_units: usize = dungeon.rooms.iter().map(|r| r.monsters.len()).sum();
    if total_units > MAX_UNITS {
        return Err(SimError::EntityLimit);
    }

    Ok(())
}

#[cfg(test)]
mod tests;
