use serde::{Deserialize, Serialize};

use crate::model::{RoomId, StatusKind, TrapId, UnitId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimulationEvent {
    UnitSpawned {
        tick: u32,
        unit_id: UnitId,
        room_id: RoomId,
    },
    UnitMoved {
        tick: u32,
        unit_id: UnitId,
        from: RoomId,
        to: RoomId,
    },
    TrapTriggered {
        tick: u32,
        trap_id: TrapId,
        room_id: RoomId,
    },
    DamageApplied {
        tick: u32,
        source: Option<UnitId>,
        target: UnitId,
        amount: i32,
    },
    StatusApplied {
        tick: u32,
        target: UnitId,
        kind: StatusKind,
    },
    UnitDied {
        tick: u32,
        unit_id: UnitId,
    },
    CoreDamaged {
        tick: u32,
        amount: i32,
        core_hp_after: i32,
    },
}
