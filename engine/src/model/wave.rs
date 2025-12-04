use serde::{Deserialize, Serialize};

use super::RoomId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeroSpawn {
    pub hero_template_id: String,
    pub count: u32,
    pub spawn_room_id: RoomId,
    pub delay_ticks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WaveConfig {
    pub id: String,
    pub entries: Vec<HeroSpawn>,
    pub modifiers: Vec<String>,
}
