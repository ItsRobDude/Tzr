use serde::{Deserialize, Serialize};

use super::{RoomId, TrapInstance, UnitInstance};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoomState {
    pub id: RoomId,
    pub traps: Vec<TrapInstance>,
    pub monsters: Vec<UnitInstance>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DungeonState {
    pub rooms: Vec<RoomState>,
    pub edges: Vec<(RoomId, RoomId)>,
    pub core_room_id: RoomId,
    pub core_hp: i32,
}
