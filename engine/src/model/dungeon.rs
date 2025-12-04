use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{RoomId, TrapInstance, UnitInstance};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// State for an individual room, including its occupants and tags.
///
/// # JSON example
/// ```json
/// {
///   "id": 1,
///   "traps": [],
///   "monsters": [],
///   "tags": ["spawn", "safe"]
/// }
/// ```
pub struct RoomState {
    pub id: RoomId,
    pub traps: Vec<TrapInstance>,
    pub monsters: Vec<UnitInstance>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Top-level dungeon description used for simulations.
///
/// # JSON example
/// ```json
/// {
///   "rooms": [
///     {
///       "id": 1,
///       "traps": [],
///       "monsters": [],
///       "tags": ["core"]
///     }
///   ],
///   "edges": [[1, 2], [2, 3]],
///   "core_room_id": 1,
///   "core_hp": 250
/// }
/// ```
pub struct DungeonState {
    pub rooms: Vec<RoomState>,
    pub edges: Vec<(RoomId, RoomId)>,
    pub core_room_id: RoomId,
    pub core_hp: i32,
}
