use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod dungeon;
pub mod status;
pub mod trap;
pub mod unit;
pub mod wave;

pub use dungeon::{DungeonState, RoomState};
pub use status::{StatusInstance, StatusKind};
pub use trap::{TrapInstance, TrapTriggerType};
pub use unit::{AiBehavior, Faction, UnitInstance, UnitStats};
pub use wave::{HeroSpawn, WaveConfig};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct RoomId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct UnitId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct TrapId(pub u32);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SimulationOutcome {
    DungeonWin,
    HeroesWin,
    Timeout,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationStats {
    pub ticks_run: u32,
    pub heroes_spawned: u32,
    pub heroes_killed: u32,
    pub monsters_killed: u32,
    pub total_damage_to_core: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimulationResult {
    pub outcome: SimulationOutcome,
    pub final_dungeon: DungeonState,
    pub final_heroes: Vec<UnitInstance>,
    pub stats: SimulationStats,
    pub events: Vec<crate::sim::events::SimulationEvent>,
    pub engine_version: String,
}
