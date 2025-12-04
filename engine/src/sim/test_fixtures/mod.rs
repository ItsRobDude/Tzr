use crate::error::SimError;
use crate::model::{
    AiBehavior, DungeonState, Faction, HeroSpawn, RoomId, RoomState, StatusInstance, StatusKind,
    TrapId, TrapInstance, TrapTriggerType, UnitId, UnitInstance, UnitStats, WaveConfig,
};
use crate::sim::simulate_wave;

#[derive(Clone)]
pub struct ScenarioFixture {
    pub name: &'static str,
    pub dungeon: DungeonState,
    pub wave: WaveConfig,
    pub seed: u64,
    pub max_ticks: u32,
    pub snapshot: &'static str,
}

impl ScenarioFixture {
    pub fn run(&self) -> Result<crate::model::SimulationResult, SimError> {
        simulate_wave(
            self.dungeon.clone(),
            self.wave.clone(),
            self.seed,
            self.max_ticks,
        )
    }

    pub fn expected_result(&self) -> crate::model::SimulationResult {
        serde_json::from_str(self.snapshot)
            .unwrap_or_else(|err| panic!("failed to parse snapshot for {}: {err}", self.name))
    }
}

pub fn movement_to_core() -> ScenarioFixture {
    let rooms = vec![room(0), room(1), room(2)];
    let edges = vec![(RoomId(0), RoomId(1)), (RoomId(1), RoomId(2))];

    let dungeon = DungeonState {
        rooms,
        edges,
        core_room_id: RoomId(2),
        core_hp: 15,
    };

    let wave = WaveConfig {
        id: "movement-wave".into(),
        entries: vec![HeroSpawn {
            hero_template_id: "scout".into(),
            count: 1,
            spawn_room_id: RoomId(0),
            delay_ticks: 0,
        }],
        modifiers: Vec::new(),
    };

    ScenarioFixture {
        name: "movement_to_core",
        dungeon,
        wave,
        seed: 1,
        max_ticks: 20,
        snapshot: include_str!("movement_to_core_snapshot.json"),
    }
}

pub fn trapped_entry_hall() -> ScenarioFixture {
    let mut entry = room(0);
    entry.traps.push(TrapInstance {
        id: TrapId(0),
        trigger_type: TrapTriggerType::OnEnter,
        cooldown_ticks: 0,
        cooldown_remaining: 0,
        max_charges: Some(1),
        charges_used: 0,
        damage: 12,
        status_on_hit: Some(StatusInstance {
            kind: StatusKind::Poison,
            remaining_ticks: 2,
            magnitude: 6.0,
        }),
        tags: Vec::new(),
    });
    let core = room(1);

    let dungeon = DungeonState {
        rooms: vec![entry, core],
        edges: vec![(RoomId(0), RoomId(1))],
        core_room_id: RoomId(1),
        core_hp: 50,
    };

    let wave = WaveConfig {
        id: "trapped-entry".into(),
        entries: vec![HeroSpawn {
            hero_template_id: "thief".into(),
            count: 1,
            spawn_room_id: RoomId(0),
            delay_ticks: 0,
        }],
        modifiers: Vec::new(),
    };

    ScenarioFixture {
        name: "trapped_entry_hall",
        dungeon,
        wave,
        seed: 2,
        max_ticks: 10,
        snapshot: include_str!("trapped_entry_hall_snapshot.json"),
    }
}

pub fn core_room_duel() -> ScenarioFixture {
    let mut core_room = room(1);
    let monster_stats = UnitStats {
        max_hp: 25,
        armor: 1,
        move_speed: 0.0,
        attack_damage: 7,
        attack_interval_ticks: 1,
        attack_range: 1,
    };

    core_room.monsters.push(UnitInstance {
        id: UnitId(0),
        faction: Faction::Monster,
        stats: monster_stats.clone(),
        hp: monster_stats.max_hp,
        room_id: core_room.id,
        status_effects: Vec::new(),
        ai_behavior: AiBehavior::Aggressive,
        attack_cooldown: 0,
    });

    let dungeon = DungeonState {
        rooms: vec![core_room.clone()],
        edges: Vec::new(),
        core_room_id: core_room.id,
        core_hp: 40,
    };

    let wave = WaveConfig {
        id: "boss-room".into(),
        entries: vec![HeroSpawn {
            hero_template_id: "champion".into(),
            count: 1,
            spawn_room_id: core_room.id,
            delay_ticks: 0,
        }],
        modifiers: Vec::new(),
    };

    ScenarioFixture {
        name: "core_room_duel",
        dungeon,
        wave,
        seed: 3,
        max_ticks: 15,
        snapshot: include_str!("core_room_duel_snapshot.json"),
    }
}

pub fn all() -> Vec<ScenarioFixture> {
    vec![movement_to_core(), trapped_entry_hall(), core_room_duel()]
}

fn room(id: u32) -> RoomState {
    RoomState {
        id: RoomId(id),
        traps: Vec::new(),
        monsters: Vec::new(),
        tags: Vec::new(),
    }
}
