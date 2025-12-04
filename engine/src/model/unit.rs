use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{RoomId, StatusInstance, UnitId};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Team allegiance for a unit.
///
/// Serialized values are lower snake case to keep JSON stable (`"hero"`, `"monster"`).
pub enum Faction {
    Hero,
    Monster,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Basic AI behavior for a unit.
pub enum AiBehavior {
    Passive,
    Aggressive,
    Defensive,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Immutable stat block for a unit instance.
pub struct UnitStats {
    pub max_hp: i32,
    pub armor: i32,
    pub move_speed: f32,
    pub attack_damage: i32,
    pub attack_interval_ticks: u32,
    pub attack_range: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Fully instantiated unit with runtime state.
///
/// # JSON example
/// ```json
/// {
///   "id": 10,
///   "faction": "monster",
///   "stats": {
///     "max_hp": 120,
///     "armor": 5,
///     "move_speed": 1.1,
///     "attack_damage": 12,
///     "attack_interval_ticks": 30,
///     "attack_range": 1
///   },
///   "hp": 120,
///   "room_id": 2,
///   "status_effects": [],
///   "ai_behavior": "aggressive",
///   "attack_cooldown": 0
/// }
/// ```
pub struct UnitInstance {
    pub id: UnitId,
    pub faction: Faction,
    pub stats: UnitStats,
    pub hp: i32,
    pub room_id: RoomId,
    pub status_effects: Vec<StatusInstance>,
    pub ai_behavior: AiBehavior,
    pub attack_cooldown: u32,
}
