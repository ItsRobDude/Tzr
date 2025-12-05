// client/src/replay/load.test.ts
import assert from 'node:assert/strict';
import { afterEach, beforeEach, describe, it, mock } from 'node:test';

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

beforeEach(() => {
  mock.restoreAll();
  mock.method(wasmEngine, 'getEngineVersion', async () => '1.2.3');
});

afterEach(() => {
  mock.restoreAll();
});

describe('loadLastRunReplay', () => {
  it('returns kind=none when there is no stored replay', async () => {
    mock.method(storage, 'loadLastRunReplayRaw', () => null);
    const clearMock = mock.method(storage, 'clearLastRunReplay', () => undefined);

    const result = await loadLastRunReplay();

    assert.deepStrictEqual(result, { kind: 'none' });
    assert.equal(clearMock.mock.callCount(), 0);
  });

  it('treats missing engineVersion as incompatible and clears replay', async () => {
    mock.method(storage, 'loadLastRunReplayRaw', () => ({
      schemaVersion: 1,
      engineVersion: undefined,
    } as any));
    const clearMock = mock.method(storage, 'clearLastRunReplay', () => undefined);

    const result = await loadLastRunReplay();

    assert.equal(result.kind, 'incompatible');
    if (result.kind === 'incompatible') {
      assert.equal(result.savedVersion, '');
      assert.equal(result.currentVersion, '1.2.3');
    }
    assert.equal(clearMock.mock.callCount(), 1);
  });

  it('treats malformed engineVersion as incompatible and clears replay', async () => {
    mock.method(storage, 'loadLastRunReplayRaw', () => ({
      schemaVersion: 1,
      engineVersion: 'not-a-version',
    } as any));
    const clearMock = mock.method(storage, 'clearLastRunReplay', () => undefined);

    const result = await loadLastRunReplay();

    assert.equal(result.kind, 'incompatible');
    if (result.kind === 'incompatible') {
      assert.equal(result.savedVersion, 'not-a-version');
      assert.equal(result.currentVersion, '1.2.3');
    }
    assert.equal(clearMock.mock.callCount(), 1);
  });

  it('clears incompatible replays', async () => {
    const saved = makeReplay('1.0.0');
    mock.method(storage, 'loadLastRunReplayRaw', () => saved);
    const clearMock = mock.method(storage, 'clearLastRunReplay', () => undefined);
    mock.method(wasmEngine, 'getEngineVersion', async () => '2.1.0');

    const result = await loadLastRunReplay();

    assert.deepStrictEqual(result, {
      kind: 'incompatible',
      savedVersion: '1.0.0',
      currentVersion: '2.1.0',
    });
    assert.equal(clearMock.mock.callCount(), 1);
  });

  it('returns ok when compatible', async () => {
    const saved = makeReplay('1.2.0');
    mock.method(storage, 'loadLastRunReplayRaw', () => saved);
    const clearMock = mock.method(storage, 'clearLastRunReplay', () => undefined);
    mock.method(wasmEngine, 'getEngineVersion', async () => '1.2.3');

    const result = await loadLastRunReplay();

    assert.equal(result.kind, 'ok');
    if (result.kind === 'ok') {
      assert.equal(result.replay, saved);
    }
    assert.equal(clearMock.mock.callCount(), 0);
  });
});
