use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::RoomId;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Defines a hero spawn entry in a wave.
pub struct HeroSpawn {
    pub hero_template_id: String,
    pub count: u32,
    pub spawn_room_id: RoomId,
    pub delay_ticks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Configuration for a wave of invading heroes.
///
/// # JSON example
/// ```json
/// {
///   "id": "wave-1",
///   "entries": [
///     {
///       "hero_template_id": "ember_guard",
///       "count": 3,
///       "spawn_room_id": 2,
///       "delay_ticks": 0
///     }
///   ],
///   "modifiers": ["enraged"]
/// }
/// ```
pub struct WaveConfig {
    pub id: String,
    pub entries: Vec<HeroSpawn>,
    pub modifiers: Vec<String>,
}
