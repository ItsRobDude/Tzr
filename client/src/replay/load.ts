// client/src/replay/load.ts
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
  const rawVersion = (saved as any).engineVersion;
  const savedVersion = typeof rawVersion === 'string' ? rawVersion.trim() : '';

  // 1) Replays from before engineVersion existed, or with clearly missing string
  if (!savedVersion) {
    console.warn(
      `[replay] missing engineVersion on stored replay; clearing last run replay. ` +
        `(current engine=${currentVersion})`,
    );
    clearLastRunReplay();
    return { kind: 'incompatible', savedVersion, currentVersion };
  }

  // 2) Use isReplayCompatible, which internally guards against malformed semver
  const compatible = isReplayCompatible(savedVersion, currentVersion);
  if (!compatible) {
    console.warn(
      `[replay] incompatible engine version; saved=${savedVersion}, current=${currentVersion}. ` +
        `Clearing last run replay.`,
    );
    clearLastRunReplay();
    return { kind: 'incompatible', savedVersion, currentVersion };
  }

  // 3) Schema version gate (replay format), also treated as incompatible
  if (saved.schemaVersion !== 1) {
    console.warn(
      `[replay] unsupported schema version ${saved.schemaVersion}, clearing last run replay.`,
    );
    clearLastRunReplay();
    return { kind: 'incompatible', savedVersion, currentVersion };
  }

  return { kind: 'ok', replay: saved };
}
