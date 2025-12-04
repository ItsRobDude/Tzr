use serde::{Deserialize, Serialize};

use super::{RoomId, StatusInstance, UnitId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Faction {
    Hero,
    Monster,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AiBehavior {
    Passive,
    Aggressive,
    Defensive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitStats {
    pub max_hp: i32,
    pub armor: i32,
    pub move_speed: f32,
    pub attack_damage: i32,
    pub attack_interval_ticks: u32,
    pub attack_range: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
