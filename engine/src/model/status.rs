use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatusKind {
    Poison,
    Burn,
    Slow,
    Stun,
    BuffDamage,
    BuffArmor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusInstance {
    pub kind: StatusKind,
    pub remaining_ticks: u32,
    pub magnitude: f32,
}
