import type { DungeonState, SimulationResult, WaveConfig } from '../engine/types';

export type ReplaySchemaVersion = 1;

export interface StoredWaveReplay {
  waveId: string;
  seed: bigint;
  dungeon: DungeonState;
  wave: WaveConfig;
  result: SimulationResult;
}

export interface StoredRunReplay {
  schemaVersion: ReplaySchemaVersion;
  engineVersion: string;
  runId: string;
  createdAtIso: string;
  waves: StoredWaveReplay[];
}
