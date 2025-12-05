// client/src/replay/versionCheck.ts

/**
 * Parse a semver-ish engine version string.
 *
 * Handles:
 *  - "1.2.3"
 *  - "1.2"      -> patch = 0
 *  - "1"        -> minor = 0, patch = 0
 *  - "1.2.3-alpha.1"  -> core "1.2.3"
 *  - "1.2.3+build.5"  -> core "1.2.3"
 */
function parseEngineVersion(version: string): [number, number, number] {
  // strip pre-release and build metadata
  const core = version.split('-')[0].split('+')[0].trim();
  const parts = core.split('.');

  const major = Number(parts[0] ?? '0');
  const minor = Number(parts[1] ?? '0');
  const patch = Number(parts[2] ?? '0');

  if (!Number.isFinite(major) || !Number.isFinite(minor) || !Number.isFinite(patch)) {
    throw new Error(`invalid engine version string: ${version}`);
  }

  return [major, minor, patch];
}

/**
 * Replays are compatible only when MAJOR and MINOR match.
 * PATCH is ignored, so 1.2.0 replays run fine on 1.2.3.
 *
 * Any parse failure is treated as "not compatible" instead of throwing.
 */
export function isReplayCompatible(
  savedEngineVersion: string,
  currentEngineVersion: string,
): boolean {
  try {
    const [savedMajor, savedMinor] = parseEngineVersion(savedEngineVersion);
    const [currentMajor, currentMinor] = parseEngineVersion(currentEngineVersion);
    return savedMajor === currentMajor && savedMinor === currentMinor;
  } catch {
    return false;
  }
}
