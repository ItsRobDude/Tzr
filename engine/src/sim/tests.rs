use crate::model::SimulationOutcome;
use crate::model::{
    AiBehavior, DungeonState, Faction, RoomId, StatusKind, TrapId, UnitId, UnitInstance,
    WaveConfig, dungeon::RoomState, status::StatusInstance, trap::TrapInstance,
    trap::TrapTriggerType, unit::UnitStats, wave::HeroSpawn,
};
use crate::sim::tick::{SimState, step_tick};
use crate::sim::{simulate_wave, test_fixtures};
use proptest::prelude::*;
use std::collections::HashSet;

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
fn simulations_can_timeout() {
    let room0 = basic_room(0);
    let room1 = basic_room(1);

    let dungeon = DungeonState {
        rooms: vec![room0.clone(), room1.clone()],
        edges: vec![(room0.id, room1.id)],
        core_room_id: room1.id,
        core_hp: 1_000,
    };

    let wave = WaveConfig {
        id: "timeout".into(),
        entries: vec![HeroSpawn {
            hero_template_id: "h1".into(),
            count: 1,
            spawn_room_id: room0.id,
            delay_ticks: 0,
        }],
        modifiers: Vec::new(),
    };

    let result = simulate_wave(dungeon, wave, 42, 1).expect("simulation should succeed");
    assert_eq!(SimulationOutcome::Timeout, result.outcome);
    assert_eq!(1, result.stats.heroes_spawned);
    assert_eq!(1, result.stats.ticks_run);
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
fn heroes_use_template_stats_when_spawning() {
    let room0 = basic_room(0);
    let room1 = basic_room(1);

    let dungeon = DungeonState {
        rooms: vec![room0.clone(), room1.clone()],
        edges: vec![(room0.id, room1.id)],
        core_room_id: room1.id,
        core_hp: 25,
    };

    let wave = WaveConfig {
        id: "wave-templates".into(),
        entries: vec![
            HeroSpawn {
                hero_template_id: "h1".into(),
                count: 1,
                spawn_room_id: room0.id,
                delay_ticks: 0,
            },
            HeroSpawn {
                hero_template_id: "h2".into(),
                count: 1,
                spawn_room_id: room0.id,
                delay_ticks: 0,
            },
        ],
        modifiers: Vec::new(),
    };

    let mut state = SimState::new(dungeon, &wave, 3).expect("state should initialize");
    step_tick(&mut state, &wave).expect("tick should succeed");

    assert_eq!(2, state.heroes.len());

    let h1_stats = UnitStats {
        max_hp: 20,
        armor: 0,
        move_speed: 1.0,
        attack_damage: 5,
        attack_interval_ticks: 1,
        attack_range: 1,
    };
    let h2_stats = UnitStats {
        max_hp: 24,
        armor: 1,
        move_speed: 0.9,
        attack_damage: 6,
        attack_interval_ticks: 1,
        attack_range: 1,
    };

    assert_eq!(
        h1_stats, state.heroes[0].stats,
        "h1 should use its template stats"
    );
    assert_eq!(
        h2_stats, state.heroes[1].stats,
        "h2 should use its template stats"
    );
}

#[test]
fn wave_modifiers_are_applied_to_heroes() {
    let room0 = basic_room(0);
    let room1 = basic_room(1);

    let dungeon = DungeonState {
        rooms: vec![room0.clone(), room1.clone()],
        edges: vec![(room0.id, room1.id)],
        core_room_id: room1.id,
        core_hp: 30,
    };

    let wave = WaveConfig {
        id: "wave-modifiers".into(),
        entries: vec![HeroSpawn {
            hero_template_id: "h1".into(),
            count: 1,
            spawn_room_id: room0.id,
            delay_ticks: 0,
        }],
        modifiers: vec!["enraged".into(), "reinforced".into()],
    };

    let mut state = SimState::new(dungeon, &wave, 7).expect("state should initialize");
    step_tick(&mut state, &wave).expect("tick should succeed");

    let hero = state.heroes.first().expect("hero should spawn");

    assert_eq!(25, hero.stats.max_hp, "reinforced should increase max hp");
    assert_eq!(1, hero.stats.armor, "reinforced should increase armor");
    assert_eq!(
        7, hero.stats.attack_damage,
        "enraged should increase attack damage"
    );
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

#[test]
fn timed_traps_apply_status_and_damage() {
    let mut room0 = basic_room(0);
    room0.traps.push(TrapInstance {
        id: TrapId(0),
        trigger_type: TrapTriggerType::Timed,
        cooldown_ticks: 1,
        cooldown_remaining: 0,
        max_charges: Some(2),
        charges_used: 0,
        damage: 5,
        status_on_hit: Some(StatusInstance {
            kind: StatusKind::Burn,
            remaining_ticks: 2,
            magnitude: 6.0,
        }),
        tags: Vec::new(),
    });

    let dungeon = DungeonState {
        rooms: vec![room0.clone()],
        edges: vec![],
        core_room_id: room0.id,
        core_hp: 50,
    };

    let wave = WaveConfig {
        id: "timed-trap".into(),
        entries: vec![HeroSpawn {
            hero_template_id: "h1".into(),
            count: 1,
            spawn_room_id: room0.id,
            delay_ticks: 0,
        }],
        modifiers: Vec::new(),
    };

    let result = simulate_wave(dungeon, wave, 5, 3).expect("simulation should succeed");

    assert_eq!(SimulationOutcome::DungeonWin, result.outcome);
    assert_eq!(1, result.stats.heroes_spawned);
    assert_eq!(1, result.stats.heroes_killed);
    assert!(result
        .events
        .iter()
        .any(|e| matches!(e, crate::sim::events::SimulationEvent::TrapTriggered { trap_id, .. } if *trap_id == TrapId(0))));
    assert!(result.events.iter().any(|e| matches!(
        e,
        crate::sim::events::SimulationEvent::StatusApplied {
            kind: StatusKind::Burn,
            ..
        }
    )));
}

#[test]
fn adjacent_rooms_allow_ranged_attacks() {
    let core_room = basic_room(0);
    let mut monster_room = basic_room(1);

    let monster_stats = UnitStats {
        max_hp: 10,
        armor: 0,
        move_speed: 0.0,
        attack_damage: 5,
        attack_interval_ticks: 1,
        attack_range: 1,
    };

    monster_room
        .monsters
        .push(monster(0, monster_room.id, monster_stats, 10));

    let dungeon = DungeonState {
        rooms: vec![core_room.clone(), monster_room.clone()],
        edges: vec![(core_room.id, monster_room.id)],
        core_room_id: core_room.id,
        core_hp: 100,
    };

    let wave = WaveConfig {
        id: "adjacent-range".into(),
        entries: vec![HeroSpawn {
            hero_template_id: "archer".into(),
            count: 1,
            spawn_room_id: core_room.id,
            delay_ticks: 0,
        }],
        modifiers: Vec::new(),
    };

    let mut state = SimState::new(dungeon, &wave, 7).expect("state should initialize");
    step_tick(&mut state, &wave).expect("tick should succeed");

    assert_eq!(15, state.heroes[0].hp, "hero should take ranged damage");
    assert_eq!(
        5, state.dungeon.rooms[1].monsters[0].hp,
        "monster should be damaged at range"
    );
}

#[test]
fn heroes_can_attack_core_from_adjacent_room() {
    let rooms = vec![basic_room(0), basic_room(1), basic_room(2)];

    let dungeon = DungeonState {
        rooms: rooms.clone(),
        edges: vec![(rooms[0].id, rooms[1].id), (rooms[1].id, rooms[2].id)],
        core_room_id: rooms[2].id,
        core_hp: 15,
    };

    let wave = WaveConfig {
        id: "adjacent-core".into(),
        entries: vec![HeroSpawn {
            hero_template_id: "ranger".into(),
            count: 1,
            spawn_room_id: rooms[0].id,
            delay_ticks: 0,
        }],
        modifiers: Vec::new(),
    };

    let mut state = SimState::new(dungeon, &wave, 9).expect("state should initialize");
    step_tick(&mut state, &wave).expect("tick should succeed");

    assert_eq!(
        rooms[1].id, state.heroes[0].room_id,
        "hero should move toward the core"
    );
    assert_eq!(
        10, state.dungeon.core_hp,
        "core should take ranged damage from adjacent room"
    );
}

#[test]
fn traps_recover_cooldown_without_additional_entries() {
    let mut room0 = basic_room(0);
    room0.traps.push(TrapInstance {
        id: TrapId(0),
        trigger_type: TrapTriggerType::OnEnter,
        cooldown_ticks: 2,
        cooldown_remaining: 0,
        max_charges: None,
        charges_used: 0,
        damage: 50,
        status_on_hit: None,
        tags: Vec::new(),
    });
    let room1 = basic_room(1);

    let dungeon = DungeonState {
        rooms: vec![room0.clone(), room1.clone()],
        edges: vec![(room0.id, room1.id)],
        core_room_id: room1.id,
        core_hp: 100,
    };

    let wave = WaveConfig {
        id: "wave-cooldowns".into(),
        entries: vec![
            HeroSpawn {
                hero_template_id: "h1".into(),
                count: 1,
                spawn_room_id: room0.id,
                delay_ticks: 0,
            },
            HeroSpawn {
                hero_template_id: "h2".into(),
                count: 1,
                spawn_room_id: room0.id,
                delay_ticks: 3,
            },
        ],
        modifiers: Vec::new(),
    };

    let result = simulate_wave(dungeon, wave, 4, 10).expect("simulation should succeed");

    assert_eq!(2, result.stats.heroes_spawned);
    assert_eq!(2, result.stats.heroes_killed);

    let trap = result
        .final_dungeon
        .rooms
        .iter()
        .find(|room| room.id == room0.id)
        .and_then(|room| room.traps.first())
        .expect("trap should remain in dungeon");

    assert_eq!(2, trap.charges_used);
}

fn unit_stats_strategy() -> impl Strategy<Value = UnitStats> {
    (
        5..=50i32,
        0..=10i32,
        0.5f32..=2.5,
        5..=25i32,
        1..=5u32,
        1..=3u32,
    )
        .prop_map(
            |(max_hp, armor, move_speed, attack_damage, attack_interval_ticks, attack_range)| {
                UnitStats {
                    max_hp,
                    armor,
                    move_speed,
                    attack_damage,
                    attack_interval_ticks,
                    attack_range,
                }
            },
        )
}

fn monster_group_strategy() -> impl Strategy<Value = Vec<UnitInstance>> {
    prop::collection::vec((unit_stats_strategy(), 1..=50i32), 0..=3).prop_map(|monsters| {
        monsters
            .into_iter()
            .enumerate()
            .map(|(idx, (stats, hp_guess))| UnitInstance {
                id: UnitId(idx as u32),
                faction: Faction::Monster,
                stats: stats.clone(),
                hp: hp_guess.clamp(1, stats.max_hp.max(1)),
                room_id: RoomId(0),
                status_effects: Vec::new(),
                ai_behavior: AiBehavior::Aggressive,
                attack_cooldown: 0,
            })
            .collect()
    })
}

fn dungeon_and_wave_strategy() -> impl Strategy<Value = (DungeonState, WaveConfig, u64)> {
    const MAX_ROOMS: usize = 4;
    prop::collection::vec(monster_group_strategy(), 1..=MAX_ROOMS).prop_flat_map(|monster_groups| {
        let room_count = monster_groups.len();
        let entries_strategy = prop::collection::vec((1..=3u32, 0..room_count, 0..=10u32), 1..=3);
        (
            Just(monster_groups),
            0..room_count,
            25..=250i32,
            entries_strategy,
            any::<u64>(),
        )
            .prop_map(|(mut monster_groups, core_idx, core_hp, entries, seed)| {
                let mut next_id = 0u32;
                let rooms: Vec<RoomState> = monster_groups
                    .iter_mut()
                    .enumerate()
                    .map(|(room_idx, monsters)| {
                        let room_id = RoomId(room_idx as u32);
                        for monster in monsters.iter_mut() {
                            monster.room_id = room_id;
                            monster.id = UnitId(next_id);
                            next_id += 1;
                        }
                        RoomState {
                            id: room_id,
                            traps: Vec::new(),
                            monsters: monsters.clone(),
                            tags: Vec::new(),
                        }
                    })
                    .collect();

                let edges = (0..rooms.len().saturating_sub(1))
                    .map(|idx| (RoomId(idx as u32), RoomId(idx as u32 + 1)))
                    .collect();

                let dungeon = DungeonState {
                    rooms: rooms.clone(),
                    edges,
                    core_room_id: RoomId(core_idx as u32),
                    core_hp,
                };

                let wave_entries = entries
                    .into_iter()
                    .enumerate()
                    .map(|(idx, (count, room_idx, delay_ticks))| HeroSpawn {
                        hero_template_id: format!("hero-{idx}"),
                        count,
                        spawn_room_id: RoomId(room_idx as u32),
                        delay_ticks,
                    })
                    .collect();

                let wave = WaveConfig {
                    id: "wave-proptest".into(),
                    entries: wave_entries,
                    modifiers: Vec::new(),
                };

                (dungeon, wave, seed)
            })
    })
}

proptest! {
    #[test]
    fn simulation_is_deterministic((dungeon, wave, seed) in dungeon_and_wave_strategy()) {
        let first = simulate_wave(dungeon.clone(), wave.clone(), seed, 250)
            .expect("first simulation should succeed");
        let second = simulate_wave(dungeon, wave, seed, 250)
            .expect("second simulation should succeed");

        prop_assert_eq!(first, second);
    }
}

proptest! {
    #[test]
    fn simulation_preserves_basic_invariants((dungeon, wave, seed) in dungeon_and_wave_strategy()) {
        let result = simulate_wave(dungeon, wave, seed, 250)
            .expect("simulation should succeed");

        prop_assert!(!result.final_dungeon.rooms.is_empty());
        let room_ids: HashSet<RoomId> = result.final_dungeon.rooms.iter().map(|r| r.id).collect();

        for hero in &result.final_heroes {
            prop_assert!(hero.hp >= 0);
            prop_assert!(hero.hp <= hero.stats.max_hp);
            prop_assert!(room_ids.contains(&hero.room_id));
        }

        for room in &result.final_dungeon.rooms {
            prop_assert!(room_ids.contains(&room.id));
            for monster in &room.monsters {
                prop_assert!(monster.hp >= 0);
                prop_assert!(monster.hp <= monster.stats.max_hp);
                prop_assert_eq!(monster.room_id, room.id);
            }
        }

        prop_assert!(result.stats.ticks_run <= 250);
    }
}

fn assert_snapshot(fixture: &test_fixtures::ScenarioFixture) {
    let expected = fixture.expected_result();
    let actual = fixture
        .run()
        .unwrap_or_else(|err| panic!("{} scenario failed: {err}", fixture.name));
    assert_eq!(expected, actual, "{} scenario drifted", fixture.name);
}

#[test]
fn movement_snapshot_is_stable() {
    assert_snapshot(&test_fixtures::movement_to_core());
}

#[test]
fn traps_snapshot_is_stable() {
    assert_snapshot(&test_fixtures::trapped_entry_hall());
}

#[test]
fn combat_snapshot_is_stable() {
    assert_snapshot(&test_fixtures::core_room_duel());
}
