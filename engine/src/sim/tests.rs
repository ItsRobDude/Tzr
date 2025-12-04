use crate::model::SimulationOutcome;
use crate::model::{
    AiBehavior, DungeonState, Faction, RoomId, StatusKind, TrapId, UnitId, UnitInstance,
    WaveConfig, dungeon::RoomState, status::StatusInstance, trap::TrapInstance,
    trap::TrapTriggerType, unit::UnitStats, wave::HeroSpawn,
};
use crate::sim::simulate_wave;

fn basic_room(id: u32) -> RoomState {
    RoomState {
        id: RoomId(id),
        traps: Vec::new(),
        monsters: Vec::new(),
        tags: Vec::new(),
    }
}

fn monster(id: u32, room_id: RoomId, stats: UnitStats, hp: i32) -> UnitInstance {
    UnitInstance {
        id: UnitId(id),
        faction: Faction::Monster,
        stats: stats.clone(),
        hp,
        room_id,
        status_effects: Vec::new(),
        ai_behavior: AiBehavior::Aggressive,
        attack_cooldown: 0,
    }
}

#[test]
fn heroes_can_destroy_core() {
    let room0 = basic_room(0);
    let room1 = basic_room(1);

    let dungeon = DungeonState {
        rooms: vec![room0.clone(), room1.clone()],
        edges: vec![(room0.id, room1.id)],
        core_room_id: room1.id,
        core_hp: 5,
    };

    let wave = WaveConfig {
        id: "wave1".into(),
        entries: vec![HeroSpawn {
            hero_template_id: "h1".into(),
            count: 1,
            spawn_room_id: room0.id,
            delay_ticks: 0,
        }],
        modifiers: Vec::new(),
    };

    let result = simulate_wave(dungeon, wave, 1, 10).expect("simulation should succeed");
    assert_eq!(SimulationOutcome::HeroesWin, result.outcome);
    assert!(result.final_dungeon.core_hp <= 0);
    assert_eq!(1, result.stats.heroes_spawned);
}

#[test]
fn monsters_can_defend_core() {
    let mut core_room = basic_room(0);
    let monster_stats = UnitStats {
        max_hp: 30,
        armor: 0,
        move_speed: 0.0,
        attack_damage: 25,
        attack_interval_ticks: 1,
        attack_range: 1,
    };
    core_room
        .monsters
        .push(monster(0, core_room.id, monster_stats, 30));

    let dungeon = DungeonState {
        rooms: vec![core_room.clone()],
        edges: vec![],
        core_room_id: core_room.id,
        core_hp: 50,
    };

    let wave = WaveConfig {
        id: "wave2".into(),
        entries: vec![HeroSpawn {
            hero_template_id: "h1".into(),
            count: 1,
            spawn_room_id: core_room.id,
            delay_ticks: 0,
        }],
        modifiers: Vec::new(),
    };

    let result = simulate_wave(dungeon, wave, 2, 5).expect("simulation should succeed");
    assert_eq!(SimulationOutcome::DungeonWin, result.outcome);
    assert_eq!(1, result.stats.heroes_killed);
}

#[test]
fn traps_apply_status_and_kill() {
    let mut room0 = basic_room(0);
    room0.traps.push(TrapInstance {
        id: TrapId(0),
        trigger_type: TrapTriggerType::OnEnter,
        cooldown_ticks: 0,
        cooldown_remaining: 0,
        max_charges: None,
        charges_used: 0,
        damage: 10,
        status_on_hit: Some(StatusInstance {
            kind: StatusKind::Poison,
            remaining_ticks: 1,
            magnitude: 15.0,
        }),
        tags: Vec::new(),
    });
    let room1 = basic_room(1);

    let dungeon = DungeonState {
        rooms: vec![room0.clone(), room1.clone()],
        edges: vec![(room0.id, room1.id)],
        core_room_id: room1.id,
        core_hp: 50,
    };

    let wave = WaveConfig {
        id: "wave3".into(),
        entries: vec![HeroSpawn {
            hero_template_id: "h1".into(),
            count: 1,
            spawn_room_id: room0.id,
            delay_ticks: 0,
        }],
        modifiers: Vec::new(),
    };

    let result = simulate_wave(dungeon, wave, 3, 5).expect("simulation should succeed");
    assert_eq!(SimulationOutcome::DungeonWin, result.outcome);
    assert_eq!(1, result.stats.heroes_killed);
}
