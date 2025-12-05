import type { SimulationResult, DungeonState, WaveConfig } from './types';
import initWasm, { simulate_wave_wasm, engine_version_wasm } from '../../../engine/wasm/pkg/engine_wasm';

let initialized = false;
let initPromise: Promise<void> | null = null;
let cachedEngineVersion: string | null = null;

async function ensureInitialized(): Promise<void> {
  if (initialized) return;
  if (!initPromise) {
    initPromise = (async () => {
      await initWasm();
      cachedEngineVersion = engine_version_wasm();
      initialized = true;
      console.info('[engine] initialized, version', cachedEngineVersion);
    })();
  }
  await initPromise;
}

export async function getEngineVersion(): Promise<string> {
  await ensureInitialized();
  if (!cachedEngineVersion) {
    cachedEngineVersion = engine_version_wasm();
  }
  return cachedEngineVersion;
}

export function getEngineVersionSync(): string | null {
  return cachedEngineVersion;
}

export async function simulateWave(
  dungeon: DungeonState,
  wave: WaveConfig,
  seed: bigint,
  maxTicks: number,
): Promise<SimulationResult> {
  await ensureInitialized();
  const result = simulate_wave_wasm(dungeon, wave, seed, maxTicks);
  const plain = JSON.parse(JSON.stringify(result)) as SimulationResult;
  if (!plain.engine_version) {
    throw new Error('engine_version missing from SimulationResult');
  }
  return plain;
}
