import { invoke } from '@tauri-apps/api/core';

function logToBackend(message: string, source?: string, stack?: string) {
  invoke('log_js_error', { message, source, stack }).catch(() => {});
}

export function setupErrorLogging() {
  // Unhandled JS errors
  window.onerror = (message, source, _lineno, _colno, error) => {
    logToBackend(String(message), source, error?.stack);
    return false;
  };

  // Unhandled promise rejections
  window.onunhandledrejection = (event) => {
    const err = event.reason;
    const message = err instanceof Error ? err.message : String(err);
    const stack = err instanceof Error ? err.stack : undefined;
    logToBackend(`Unhandled rejection: ${message}`, undefined, stack);
  };
}
