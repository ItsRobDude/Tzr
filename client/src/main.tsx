import React from 'react';
import ReactDOM from 'react-dom/client';
import { EngineMetaProvider } from './engine/EngineMetaContext';
import { App } from './ui/App';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <EngineMetaProvider>
      <App />
    </EngineMetaProvider>
  </React.StrictMode>,
);
