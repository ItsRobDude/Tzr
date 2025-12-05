export type RoomId = number;
export type UnitId = number;
export type TrapId = number;

export type SimulationOutcome = 'DungeonWin' | 'HeroesWin' | 'Timeout';

export interface SimulationStats {
  ticks_run: number;
  heroes_spawned: number;
  heroes_killed: number;
  monsters_killed: number;
  total_damage_to_core: number;
}

export interface DungeonRoom {
  id: RoomId;
  traps: any[]; // TODO: refine to TrapInstance
  monsters: any[]; // TODO: refine to MonsterInstance
  tags: string[];
}

export interface DungeonState {
  rooms: DungeonRoom[];
  edges: [RoomId, RoomId][];
  core_room_id: RoomId;
  core_hp: number;
}

export interface HeroInstance {
  id: UnitId;
  faction: 'hero';
  stats: {
    max_hp: number;
    armor: number;
    move_speed: number;
    attack_damage: number;
    attack_interval_ticks: number;
    attack_range: number;
  };
  hp: number;
  room_id: RoomId;
  status_effects: unknown[];
  ai_behavior: 'aggressive' | string;
  attack_cooldown: number;
}

export type SimulationEvent =
  | { UnitSpawned: { tick: number; unit_id: UnitId; room_id: RoomId } }
  | { UnitMoved: { tick: number; unit_id: UnitId; from: RoomId; to: RoomId } }
  | { TrapTriggered: { tick: number; trap_id: TrapId; room_id: RoomId } }
  | { DamageApplied: { tick: number; source: UnitId | null; target: UnitId; amount: number } }
  | { StatusApplied: { tick: number; target: UnitId; kind: string } }
  | { UnitDied: { tick: number; unit_id: UnitId } }
  | { CoreDamaged: { tick: number; amount: number; core_hp_after: number } };

export interface SimulationResult {
  engine_version: string;
  outcome: SimulationOutcome;
  final_dungeon: DungeonState;
  final_heroes: HeroInstance[];
  stats: SimulationStats;
  events: SimulationEvent[];
}

export interface HeroSpawn {
  hero_template_id: string;
  count: number;
  spawn_room_id: RoomId;
  delay_ticks: number;
}

export interface WaveConfig {
  id: string;
  entries: HeroSpawn[];
  modifiers: string[];
}
