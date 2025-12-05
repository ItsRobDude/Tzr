import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { getEngineVersion } from './wasmEngine';

type EngineMeta = {
  engineVersion: string;
};

const EngineMetaContext = createContext<EngineMeta | null>(null);

export function EngineMetaProvider({ children }: { children: ReactNode }) {
  const [meta, setMeta] = useState<EngineMeta | null>(null);

  useEffect(() => {
    let cancelled = false;

    (async () => {
      const engineVersion = await getEngineVersion();
      if (cancelled) return;
      setMeta({ engineVersion });
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  if (!meta) {
    return <div className="splash">Loading engine…</div>;
  }

  return <EngineMetaContext.Provider value={meta}>{children}</EngineMetaContext.Provider>;
}

export function useEngineMeta(): EngineMeta {
  const ctx = useContext(EngineMetaContext);
  if (!ctx) {
    throw new Error('useEngineMeta must be used inside EngineMetaProvider');
  }
  return ctx;
}
