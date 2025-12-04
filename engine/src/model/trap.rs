use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{StatusInstance, TrapId};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// The condition under which a trap triggers.
pub enum TrapTriggerType {
    OnEnter,
    OnExit,
    Timed,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// A placed trap instance with runtime state.
///
/// # JSON example
/// ```json
/// {
///   "id": 3,
///   "trigger_type": "on_enter",
///   "cooldown_ticks": 12,
///   "cooldown_remaining": 0,
///   "max_charges": 4,
///   "charges_used": 1,
///   "damage": 10,
///   "status_on_hit": null,
///   "tags": ["aoe"]
/// }
/// ```
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
