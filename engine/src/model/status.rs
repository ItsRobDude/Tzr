use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Types of status effects that can be applied to a unit.
pub enum StatusKind {
    Poison,
    Burn,
    Slow,
    Stun,
    BuffDamage,
    BuffArmor,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// A concrete status effect applied to a unit.
///
/// # JSON example
/// ```json
/// {
///   "kind": "poison",
///   "remaining_ticks": 12,
///   "magnitude": 3.5
/// }
/// ```
pub struct StatusInstance {
    pub kind: StatusKind,
    pub remaining_ticks: u32,
    pub magnitude: f32,
}
