import { getEngineVersion } from '../engine/wasmEngine';
import type { StoredRunReplay } from './types';
import { loadLastRunReplayRaw, clearLastRunReplay } from './storage';
import { isReplayCompatible } from './versionCheck';

export type LoadReplayResult =
  | { kind: 'none' }
  | { kind: 'incompatible'; savedVersion: string; currentVersion: string }
  | { kind: 'ok'; replay: StoredRunReplay };

export async function loadLastRunReplay(): Promise<LoadReplayResult> {
  const saved = loadLastRunReplayRaw();
  if (!saved) {
    return { kind: 'none' };
  }

  const currentVersion = await getEngineVersion();
  const savedVersion = saved.engineVersion;

  if (!isReplayCompatible(savedVersion, currentVersion)) {
    console.warn(
      `[replay] incompatible engine version; saved=${savedVersion}, current=${currentVersion}. Clearing last run replay.`,
    );
    clearLastRunReplay();
    return { kind: 'incompatible', savedVersion, currentVersion };
  }

  if (saved.schemaVersion !== 1) {
    console.warn(`[replay] unsupported schema version ${saved.schemaVersion}, clearing.`);
    clearLastRunReplay();
    return { kind: 'incompatible', savedVersion, currentVersion };
  }

  return { kind: 'ok', replay: saved };
}
