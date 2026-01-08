import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import dayjs from 'dayjs';
import durationPlugin from 'dayjs/plugin/duration';
import localizedFormatPlugin from "dayjs/plugin/localizedFormat";
import relativeTimePlugin from 'dayjs/plugin/relativeTime';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import AppShell from './AppShell.tsx';
import BackendPanicListener from './components/BackendPanicListener/index.tsx';
import './index.css';

dayjs.extend(relativeTimePlugin);
dayjs.extend(durationPlugin);
dayjs.extend(localizedFormatPlugin);

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

const queryClient = new QueryClient();

createRoot(document.getElementById('root')!).render(
  <>
    <BackendPanicListener />
    <StrictMode>
      <QueryClientProvider client={queryClient}>
        <AppShell />
      </QueryClientProvider>
    </StrictMode>
  </>,
);
