import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import App from './App.tsx'
import './index.css'

import { ErrorBoundary } from "react-error-boundary";

function fallbackRender(context: any) {
  // Call resetErrorBoundary() to reset the error boundary and retry the render.

  return (
    <div role="alert">
      <p>Something went wrong:</p>
      <pre style={{ color: "red" }}>{context.error.message}</pre>
    </div>
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ErrorBoundary fallbackRender={fallbackRender}>
      <App />
    </ErrorBoundary>
  </StrictMode>,
)
