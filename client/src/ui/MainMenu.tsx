import React from 'react';
import { useEngineMeta } from '../engine/EngineMetaContext';
import { loadLastRunReplay } from '../replay/load';

export function MainMenu() {
  const { engineVersion } = useEngineMeta();

  async function handleReplayClick() {
    const result = await loadLastRunReplay();
    switch (result.kind) {
      case 'none':
        alert('No replay available for the last run.');
        break;
      case 'incompatible':
        alert(
          `Cannot replay last run.\n` +
            `Recorded with engine ${result.savedVersion}, but current engine is ${result.currentVersion}.`,
        );
        break;
      case 'ok':
        // Integrate with replay flow when available
        console.info('[replay] ready to load last run replay', result.replay);
        break;
    }
  }

  return (
    <div className="main-menu">
      <button onClick={handleReplayClick}>Replay Last Run</button>

      <div className="version-strip">Engine {engineVersion}</div>
    </div>
  );
}
