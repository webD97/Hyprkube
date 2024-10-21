import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App.tsx'
import './index.css'

import { ErrorBoundary, FallbackProps } from "react-error-boundary";
import { emit } from '@tauri-apps/api/event';

function fallbackRender(context: FallbackProps) {
  // Call resetErrorBoundary() to reset the error boundary and retry the render.

  return (
    <div role="alert">
      <p>Something went wrong:</p>
      <pre style={{ color: "red" }}>{context.error.message}</pre>
    </div>
  );
}

window.onbeforeunload = function () {
  emit('frontend-onbeforeunload');
};

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ErrorBoundary fallbackRender={fallbackRender}>
      <App />
    </ErrorBoundary>
  </StrictMode>,
)
