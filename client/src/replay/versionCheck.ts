function parseEngineVersion(version: string): [number, number, number] {
  const [majorStr, minorStr, patchStr] = version.split('.');
  const major = Number(majorStr);
  const minor = Number(minorStr);
  const patch = Number(patchStr ?? '0');
  if (!Number.isFinite(major) || !Number.isFinite(minor) || !Number.isFinite(patch)) {
    throw new Error(`invalid engine version string: ${version}`);
  }
  return [major, minor, patch];
}

/**
 * Replays are compatible only when MAJOR and MINOR match.
 * PATCH is ignored, so 1.2.0 replays run fine on 1.2.3.
 */
export function isReplayCompatible(
  savedEngineVersion: string,
  currentEngineVersion: string,
): boolean {
  const [savedMajor, savedMinor] = parseEngineVersion(savedEngineVersion);
  const [currentMajor, currentMinor] = parseEngineVersion(currentEngineVersion);
  return savedMajor === currentMajor && savedMinor === currentMinor;
}

export { parseEngineVersion };
