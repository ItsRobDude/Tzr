use serde::{Deserialize, Serialize};

use super::{StatusInstance, TrapId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TrapTriggerType {
    OnEnter,
    OnExit,
    Timed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrapInstance {
    pub id: TrapId,
    pub trigger_type: TrapTriggerType,
    pub cooldown_ticks: u32,
    pub cooldown_remaining: u32,
    pub max_charges: Option<u32>,
    pub charges_used: u32,
    pub damage: i32,
    pub status_on_hit: Option<StatusInstance>,
    pub tags: Vec<String>,
}
