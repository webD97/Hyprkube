import { invoke } from '@tauri-apps/api/core';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import AppShell from './AppShell.tsx';
import BackendPanicListener from './components/BackendPanicListener/index.tsx';
import './index.css';

const consoleLog = console.log;

console.log = function (...data: unknown[]) {
  void invoke('log_stdout', {
    line: data.map(field => {
      if (typeof field === "object") {
        return JSON.stringify(field);
      }

      // eslint-disable-next-line @typescript-eslint/no-base-to-string
      return field?.toString();
    }).join('')
  });
  consoleLog(...data);
}

createRoot(document.getElementById('root')!).render(
  <>
    <BackendPanicListener />
    <StrictMode>
      <AppShell />
    </StrictMode>
  </>,
);
