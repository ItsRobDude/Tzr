import assert from 'node:assert/strict';
import { test } from 'node:test';
import fs from 'node:fs';
import path from 'node:path';

import { simulate_wave_wasm } from '../pkg/engine_wasm';

type Scenario = {
  name: string;
  dungeon: unknown;
  wave: unknown;
  seed: bigint;
  maxTicks: number;
  snapshot: any;
};

const fixturePath = (file: string) =>
  path.resolve(__dirname, '../../src/sim/test_fixtures', file);

const readSnapshot = (file: string) =>
  JSON.parse(fs.readFileSync(fixturePath(file), 'utf8'));

const room = (id: number) => ({ id, traps: [] as unknown[], monsters: [] as unknown[], tags: [] as string[] });

const scenarios: Scenario[] = [
  {
    name: 'movement_to_core',
    dungeon: {
      rooms: [room(0), room(1), room(2)],
      edges: [
        [0, 1],
        [1, 2],
      ],
      core_room_id: 2,
      core_hp: 15,
    },
    wave: {
      id: 'movement-wave',
      entries: [
        {
          hero_template_id: 'scout',
          count: 1,
          spawn_room_id: 0,
          delay_ticks: 0,
        },
      ],
      modifiers: [] as unknown[],
    },
    seed: 1n,
    maxTicks: 20,
    snapshot: readSnapshot('movement_to_core_snapshot.json'),
  },
  {
    name: 'trapped_entry_hall',
    dungeon: {
      rooms: [
        {
          ...room(0),
          traps: [
            {
              id: 0,
              trigger_type: 'on_enter',
              cooldown_ticks: 0,
              cooldown_remaining: 0,
              max_charges: 1,
              charges_used: 0,
              damage: 12,
              status_on_hit: {
                kind: 'poison',
                remaining_ticks: 2,
                magnitude: 6.0,
              },
              tags: [],
            },
          ],
        },
        room(1),
      ],
      edges: [[0, 1]],
      core_room_id: 1,
      core_hp: 50,
    },
    wave: {
      id: 'trapped-entry',
      entries: [
        {
          hero_template_id: 'thief',
          count: 1,
          spawn_room_id: 0,
          delay_ticks: 0,
        },
      ],
      modifiers: [] as unknown[],
    },
    seed: 2n,
    maxTicks: 10,
    snapshot: readSnapshot('trapped_entry_hall_snapshot.json'),
  },
  {
    name: 'core_room_duel',
    dungeon: {
      rooms: [
        {
          id: 1,
          traps: [],
          monsters: [
            {
              id: 0,
              faction: 'monster',
              stats: {
                max_hp: 25,
                armor: 1,
                move_speed: 0.0,
                attack_damage: 7,
                attack_interval_ticks: 1,
                attack_range: 1,
              },
              hp: 25,
              room_id: 1,
              status_effects: [],
              ai_behavior: 'aggressive',
              attack_cooldown: 0,
            },
          ],
          tags: [],
        },
      ],
      edges: [] as unknown[],
      core_room_id: 1,
      core_hp: 40,
    },
    wave: {
      id: 'boss-room',
      entries: [
        {
          hero_template_id: 'champion',
          count: 1,
          spawn_room_id: 1,
          delay_ticks: 0,
        },
      ],
      modifiers: [] as unknown[],
    },
    seed: 3n,
    maxTicks: 15,
    snapshot: readSnapshot('core_room_duel_snapshot.json'),
  },
];

for (const scenario of scenarios) {
  test(`simulate_wave round trips for ${scenario.name}`, () => {
    const result = simulate_wave_wasm(
      scenario.dungeon,
      scenario.wave,
      scenario.seed,
      scenario.maxTicks,
    );

    const roundTripped = JSON.parse(JSON.stringify(result));

    assert.deepStrictEqual(roundTripped, result, 'serialization should be lossless');
    assert.ok(Array.isArray(roundTripped.events), 'events should be present');
    assert.ok(roundTripped.events.length > 0, 'should emit at least one event');

    assert.deepStrictEqual(
      roundTripped.outcome,
      scenario.snapshot.outcome,
      'outcome should match snapshot',
    );
    assert.deepStrictEqual(roundTripped.final_dungeon, scenario.snapshot.final_dungeon);
    assert.deepStrictEqual(roundTripped.final_heroes, scenario.snapshot.final_heroes);
    assert.deepStrictEqual(roundTripped.stats, scenario.snapshot.stats);
    assert.deepStrictEqual(roundTripped.events, scenario.snapshot.events);
  });
}
