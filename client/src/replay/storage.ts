import type { StoredRunReplay } from './types';

const LAST_RUN_KEY = 'abyssal:lastRunReplay';

const bigintReplacer = (_key: string, value: unknown) =>
  typeof value === 'bigint' ? value.toString() : value;

function reviveBigInts(key: string, value: unknown) {
  if (key === 'seed' && typeof value === 'string') {
    return BigInt(value);
  }
  return value;
}

export function saveLastRunReplay(replay: StoredRunReplay): void {
  const payload = JSON.stringify(replay, bigintReplacer);
  window.localStorage.setItem(LAST_RUN_KEY, payload);
}

export function loadLastRunReplayRaw(): StoredRunReplay | null {
  const raw = window.localStorage.getItem(LAST_RUN_KEY);
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw, reviveBigInts) as StoredRunReplay;
    return parsed;
  } catch (err) {
    console.warn('[replay] failed to parse last run replay, clearing', err);
    window.localStorage.removeItem(LAST_RUN_KEY);
    return null;
  }
}

export function clearLastRunReplay(): void {
  window.localStorage.removeItem(LAST_RUN_KEY);
}
