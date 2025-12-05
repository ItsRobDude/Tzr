import assert from 'node:assert/strict';
import { afterEach, describe, it, mock } from 'node:test';

import * as storage from './storage';
import * as wasmEngine from '../engine/wasmEngine';
import { loadLastRunReplay } from './load';
import type { StoredRunReplay, StoredWaveReplay } from './types';

const baseWave: StoredWaveReplay = {
  waveId: 'wave-1',
  seed: 1n,
  dungeon: {
    rooms: [],
    edges: [],
    core_room_id: 0,
    core_hp: 0,
  },
  wave: {
    id: 'wave-1',
    entries: [],
    modifiers: [],
  },
  result: {
    engine_version: '1.0.0',
    outcome: 'Timeout',
    final_dungeon: {
      rooms: [],
      edges: [],
      core_room_id: 0,
      core_hp: 0,
    },
    final_heroes: [],
    stats: {
      ticks_run: 0,
      heroes_spawned: 0,
      heroes_killed: 0,
      monsters_killed: 0,
      total_damage_to_core: 0,
    },
    events: [],
  },
};

function makeReplay(engineVersion: string): StoredRunReplay {
  return {
    schemaVersion: 1,
    engineVersion,
    runId: 'run-1',
    createdAtIso: '2024-01-01T00:00:00.000Z',
    waves: [baseWave],
  };
}

afterEach(() => {
  mock.restoreAll();
});

describe('loadLastRunReplay', () => {
  it('clears incompatible replays', async () => {
    const saved = makeReplay('1.0.0');
    let cleared = false;

    mock.method(storage, 'loadLastRunReplayRaw', () => saved);
    mock.method(storage, 'clearLastRunReplay', () => {
      cleared = true;
    });
    mock.method(wasmEngine, 'getEngineVersion', async () => '2.1.0');

    const result = await loadLastRunReplay();

    assert.deepStrictEqual(result, {
      kind: 'incompatible',
      savedVersion: '1.0.0',
      currentVersion: '2.1.0',
    });
    assert.equal(cleared, true);
  });

  it('returns ok when compatible', async () => {
    const saved = makeReplay('1.2.0');
    let cleared = false;

    mock.method(storage, 'loadLastRunReplayRaw', () => saved);
    mock.method(storage, 'clearLastRunReplay', () => {
      cleared = true;
    });
    mock.method(wasmEngine, 'getEngineVersion', async () => '1.2.3');

    const result = await loadLastRunReplay();

    assert.equal(result.kind, 'ok');
    if (result.kind === 'ok') {
      assert.equal(result.replay, saved);
    }
    assert.equal(cleared, false);
  });

  it('returns none when no replay saved', async () => {
    mock.method(storage, 'loadLastRunReplayRaw', () => null);
    const result = await loadLastRunReplay();
    assert.deepStrictEqual(result, { kind: 'none' });
  });
});
