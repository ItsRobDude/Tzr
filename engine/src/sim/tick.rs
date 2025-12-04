use crate::error::SimError;
use crate::model::SimulationOutcome;
use crate::model::SimulationStats;
use crate::model::{
    AiBehavior, DungeonState, Faction, RoomId, StatusKind, TrapTriggerType, UnitId, UnitInstance,
    UnitStats, WaveConfig,
};
use crate::rng::Rng;
use crate::sim::events::SimulationEvent;
use crate::sim::pathfinding::shortest_path;
use crate::sim::{MAX_EVENTS, MAX_TICKS, MAX_UNITS};

pub struct SimState {
    pub tick: u32,
    pub dungeon: DungeonState,
    pub heroes: Vec<UnitInstance>,
    pub rng: Rng,
    pub events: Vec<SimulationEvent>,
    pub stats: SimulationStats,
    pub spawn_progress: Vec<u32>,
    next_unit_id: u32,
}

impl SimState {
    pub fn new(dungeon: DungeonState, wave: &WaveConfig, seed: u64) -> Result<Self, SimError> {
        let monster_count: usize = dungeon.rooms.iter().map(|r| r.monsters.len()).sum();
        if monster_count > MAX_UNITS {
            return Err(SimError::EntityLimit);
        }

        Ok(Self {
            tick: 0,
            dungeon,
            heroes: Vec::new(),
            rng: Rng::new(seed),
            events: Vec::new(),
            stats: SimulationStats {
                ticks_run: 0,
                heroes_spawned: 0,
                heroes_killed: 0,
                monsters_killed: 0,
                total_damage_to_core: 0,
            },
            spawn_progress: vec![0; wave.entries.len()],
            next_unit_id: 0,
        })
    }

    pub fn total_units(&self) -> usize {
        let monsters: usize = self.dungeon.rooms.iter().map(|r| r.monsters.len()).sum();
        monsters + self.heroes.len()
    }
}

fn push_event(events: &mut Vec<SimulationEvent>, event: SimulationEvent) -> Result<(), SimError> {
    if events.len() >= MAX_EVENTS {
        return Err(SimError::EntityLimit);
    }
    events.push(event);
    Ok(())
}

pub fn step_tick(
    state: &mut SimState,
    wave: &WaveConfig,
) -> Result<Option<SimulationOutcome>, SimError> {
    if state.tick >= MAX_TICKS {
        return Err(SimError::TickLimit);
    }

    spawn_heroes(state, wave)?;

    apply_status_effects(state)?;

    tick_trap_cooldowns(state);

    move_heroes(state)?;

    process_attacks(state)?;

    cleanup_dead(state);

    state.tick += 1;
    state.stats.ticks_run = state.tick;

    if state.dungeon.core_hp <= 0 {
        return Ok(Some(SimulationOutcome::HeroesWin));
    }

    if heroes_exhausted(state, wave) {
        return Ok(Some(SimulationOutcome::DungeonWin));
    }

    Ok(None)
}

fn spawn_heroes(state: &mut SimState, wave: &WaveConfig) -> Result<(), SimError> {
    for (idx, spawn) in wave.entries.iter().enumerate() {
        while state.spawn_progress[idx] < spawn.count && state.tick >= spawn.delay_ticks {
            if state.total_units() + 1 > MAX_UNITS {
                return Err(SimError::EntityLimit);
            }

            let stats = default_hero_stats();
            let unit = UnitInstance {
                id: UnitId(state.next_unit_id),
                faction: Faction::Hero,
                stats: stats.clone(),
                hp: stats.max_hp,
                room_id: spawn.spawn_room_id,
                status_effects: Vec::new(),
                ai_behavior: AiBehavior::Aggressive,
                attack_cooldown: 0,
            };
            state.next_unit_id += 1;
            state.stats.heroes_spawned += 1;
            let unit_id = unit.id;
            let room_id = unit.room_id;
            state.heroes.push(unit);
            push_event(
                &mut state.events,
                SimulationEvent::UnitSpawned {
                    tick: state.tick,
                    unit_id,
                    room_id,
                },
            )?;
            trigger_traps(state, room_id, TrapTriggerType::OnEnter, Some(unit_id))?;
            state.spawn_progress[idx] += 1;
        }
    }
    Ok(())
}

fn apply_status_effects(state: &mut SimState) -> Result<(), SimError> {
    for hero in state.heroes.iter_mut() {
        tick_statuses(&mut state.events, state.tick, hero)?;
    }
    for room in state.dungeon.rooms.iter_mut() {
        for monster in room.monsters.iter_mut() {
            tick_statuses(&mut state.events, state.tick, monster)?;
        }
    }
    Ok(())
}

fn tick_trap_cooldowns(state: &mut SimState) {
    for room in state.dungeon.rooms.iter_mut() {
        for trap in room.traps.iter_mut() {
            trap.cooldown_remaining = trap.cooldown_remaining.saturating_sub(1);
        }
    }
}

fn tick_statuses(
    events: &mut Vec<SimulationEvent>,
    tick: u32,
    unit: &mut UnitInstance,
) -> Result<(), SimError> {
    let mut damage = 0;
    for status in unit.status_effects.iter_mut() {
        if matches!(status.kind, StatusKind::Poison | StatusKind::Burn) {
            damage += status.magnitude as i32;
        }
        if status.remaining_ticks > 0 {
            status.remaining_ticks -= 1;
        }
    }
    unit.status_effects.retain(|s| s.remaining_ticks > 0);
    if damage > 0 {
        apply_damage(events, tick, None, unit, damage)?;
    }
    Ok(())
}

fn move_heroes(state: &mut SimState) -> Result<(), SimError> {
    let mut movements = Vec::new();
    for hero in state.heroes.iter_mut() {
        if hero.hp <= 0 {
            continue;
        }
        if is_stunned(hero) {
            continue;
        }

        let Some(path) = shortest_path(
            hero.room_id,
            state.dungeon.core_room_id,
            &state.dungeon.edges,
        ) else {
            continue;
        };

        if path.len() < 2 {
            continue;
        }

        let speed = effective_move_speed(hero);
        if speed <= 0.0 {
            continue;
        }

        let steps = speed.floor() as usize;
        let steps = steps.max(1);
        let steps = steps.min(path.len() - 1);
        let new_room = path[steps];
        let old_room = hero.room_id;
        hero.room_id = new_room;
        movements.push((hero.id, old_room, new_room));
    }

    for (unit_id, from, to) in movements {
        push_event(
            &mut state.events,
            SimulationEvent::UnitMoved {
                tick: state.tick,
                unit_id,
                from,
                to,
            },
        )?;
        trigger_traps(state, from, TrapTriggerType::OnExit, Some(unit_id))?;
        trigger_traps(state, to, TrapTriggerType::OnEnter, Some(unit_id))?;
    }
    Ok(())
}

fn process_attacks(state: &mut SimState) -> Result<(), SimError> {
    for room in state.dungeon.rooms.iter_mut() {
        // Monster attacks
        for monster in room.monsters.iter_mut() {
            if monster.hp <= 0 {
                continue;
            }
            if monster.attack_cooldown > 0 {
                monster.attack_cooldown -= 1;
                continue;
            }
            if is_stunned(monster) {
                continue;
            }
            if let Some(target) = state.heroes.iter_mut().find(|h| {
                h.hp > 0
                    && rooms_within_attack_range(
                        room.id,
                        h.room_id,
                        monster.stats.attack_range,
                        &state.dungeon.edges,
                    )
            }) {
                let dmg = effective_damage(monster);
                apply_damage(&mut state.events, state.tick, Some(monster.id), target, dmg)?;
                monster.attack_cooldown = monster.stats.attack_interval_ticks;
            }
        }
    }

    // Hero attacks (including core)
    for hero in state.heroes.iter_mut() {
        if hero.hp <= 0 {
            continue;
        }
        if hero.attack_cooldown > 0 {
            hero.attack_cooldown -= 1;
            continue;
        }
        if is_stunned(hero) {
            continue;
        }

        if state
            .dungeon
            .rooms
            .iter()
            .any(|r| r.id == hero.room_id)
        {
            if let Some(target) = state
                .dungeon
                .rooms
                .iter_mut()
                .find_map(|target_room| {
                    if !rooms_within_attack_range(
                        hero.room_id,
                        target_room.id,
                        hero.stats.attack_range,
                        &state.dungeon.edges,
                    ) {
                        return None;
                    }

                    target_room.monsters.iter_mut().find(|m| m.hp > 0)
                })
            {
                let dmg = effective_damage(hero);
                apply_damage(&mut state.events, state.tick, Some(hero.id), target, dmg)?;
                hero.attack_cooldown = hero.stats.attack_interval_ticks;
            } else if rooms_within_attack_range(
                hero.room_id,
                state.dungeon.core_room_id,
                hero.stats.attack_range,
                &state.dungeon.edges,
            ) {
                let dmg = effective_damage(hero);
                state.dungeon.core_hp -= dmg;
                state.stats.total_damage_to_core += dmg;
                push_event(
                    &mut state.events,
                    SimulationEvent::CoreDamaged {
                        tick: state.tick,
                        amount: dmg,
                        core_hp_after: state.dungeon.core_hp,
                    },
                )?;
                hero.attack_cooldown = hero.stats.attack_interval_ticks;
            }
        }
    }

    Ok(())
}

fn rooms_within_attack_range(
    origin: RoomId,
    target: RoomId,
    range: u32,
    edges: &[(RoomId, RoomId)],
) -> bool {
    range as usize >= shortest_path(origin, target, edges)
        .map(|path| path.len().saturating_sub(1))
        .unwrap_or(usize::MAX)
}

fn cleanup_dead(state: &mut SimState) {
    let mut hero_idx = 0;
    while hero_idx < state.heroes.len() {
        if state.heroes[hero_idx].hp <= 0 {
            let unit_id = state.heroes[hero_idx].id;
            state.stats.heroes_killed += 1;
            let _ = push_event(
                &mut state.events,
                SimulationEvent::UnitDied {
                    tick: state.tick,
                    unit_id,
                },
            );
            state.heroes.remove(hero_idx);
        } else {
            hero_idx += 1;
        }
    }

    for room in state.dungeon.rooms.iter_mut() {
        let mut monster_idx = 0;
        while monster_idx < room.monsters.len() {
            if room.monsters[monster_idx].hp <= 0 {
                let unit_id = room.monsters[monster_idx].id;
                state.stats.monsters_killed += 1;
                let _ = push_event(
                    &mut state.events,
                    SimulationEvent::UnitDied {
                        tick: state.tick,
                        unit_id,
                    },
                );
                room.monsters.remove(monster_idx);
            } else {
                monster_idx += 1;
            }
        }
    }
}

fn heroes_exhausted(state: &SimState, wave: &WaveConfig) -> bool {
    let done_spawning = wave
        .entries
        .iter()
        .enumerate()
        .all(|(i, spawn)| state.spawn_progress[i] >= spawn.count || spawn.count == 0);

    done_spawning && state.heroes.is_empty()
}

fn trigger_traps(
    state: &mut SimState,
    room_id: RoomId,
    trigger: TrapTriggerType,
    target: Option<UnitId>,
) -> Result<(), SimError> {
    if let Some(room) = state.dungeon.rooms.iter_mut().find(|r| r.id == room_id) {
        for trap in room.traps.iter_mut() {
            if trap.trigger_type != trigger {
                continue;
            }
            if trap.cooldown_remaining > 0 {
                continue;
            }
            if let Some(max_charges) = trap.max_charges {
                if trap.charges_used >= max_charges {
                    continue;
                }
            }
            trap.charges_used += 1;
            trap.cooldown_remaining = trap.cooldown_ticks;
            push_event(
                &mut state.events,
                SimulationEvent::TrapTriggered {
                    tick: state.tick,
                    trap_id: trap.id,
                    room_id,
                },
            )?;
            if let Some(target_id) = target {
                if let Some(hero) = state.heroes.iter_mut().find(|h| h.id == target_id) {
                    apply_damage(&mut state.events, state.tick, None, hero, trap.damage)?;
                    if let Some(status) = trap.status_on_hit.clone() {
                        hero.status_effects.push(status.clone());
                        push_event(
                            &mut state.events,
                            SimulationEvent::StatusApplied {
                                tick: state.tick,
                                target: hero.id,
                                kind: status.kind,
                            },
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn apply_damage(
    events: &mut Vec<SimulationEvent>,
    tick: u32,
    source: Option<UnitId>,
    target: &mut UnitInstance,
    raw_amount: i32,
) -> Result<(), SimError> {
    let damage = (raw_amount - effective_armor(target)).max(1);
    target.hp -= damage;
    push_event(
        events,
        SimulationEvent::DamageApplied {
            tick,
            source,
            target: target.id,
            amount: damage,
        },
    )?;
    Ok(())
}

fn effective_damage(unit: &UnitInstance) -> i32 {
    let bonus: f32 = unit
        .status_effects
        .iter()
        .filter(|s| matches!(s.kind, StatusKind::BuffDamage))
        .map(|s| s.magnitude)
        .sum();
    (unit.stats.attack_damage as f32 + bonus).round() as i32
}

fn effective_armor(unit: &UnitInstance) -> i32 {
    let bonus: f32 = unit
        .status_effects
        .iter()
        .filter(|s| matches!(s.kind, StatusKind::BuffArmor))
        .map(|s| s.magnitude)
        .sum();
    (unit.stats.armor as f32 + bonus).round() as i32
}

fn effective_move_speed(unit: &UnitInstance) -> f32 {
    let slow: f32 = unit
        .status_effects
        .iter()
        .filter(|s| matches!(s.kind, StatusKind::Slow))
        .map(|s| s.magnitude)
        .sum();
    (unit.stats.move_speed * (1.0 - slow)).max(0.0)
}

fn is_stunned(unit: &UnitInstance) -> bool {
    unit.status_effects
        .iter()
        .any(|s| matches!(s.kind, StatusKind::Stun))
}

fn default_hero_stats() -> UnitStats {
    UnitStats {
        max_hp: 20,
        armor: 0,
        move_speed: 1.0,
        attack_damage: 5,
        attack_interval_ticks: 1,
        attack_range: 1,
    }
}
