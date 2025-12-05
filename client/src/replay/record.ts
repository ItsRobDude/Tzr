import { getEngineVersion } from '../engine/wasmEngine';
import type { StoredRunReplay, StoredWaveReplay } from './types';

export async function buildStoredRunReplay(
  runId: string,
  waves: StoredWaveReplay[],
): Promise<StoredRunReplay> {
  const engineVersion = await getEngineVersion();
  return {
    schemaVersion: 1,
    engineVersion,
    runId,
    createdAtIso: new Date().toISOString(),
    waves,
  };
}
