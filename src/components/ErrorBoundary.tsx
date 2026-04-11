import { Component, type ReactNode } from 'react';
import { AlertCircle, RotateCcw } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { invoke } from '@tauri-apps/api/core';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallbackTitle?: string;
  fallbackMessage?: string;
  fallbackRetry?: string;
}

interface ErrorBoundaryState {
  hasError: boolean;
  errorMessage?: string;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, errorMessage: error?.message ?? String(error) };
  }

  componentDidCatch(error: Error, info: { componentStack: string }) {
    invoke('log_js_error', {
      message: error.message,
      source: 'ErrorBoundary',
      stack: (error.stack ?? '') + '\nComponent:' + info.componentStack,
    }).catch(() => {});
  }

  handleRetry = () => {
    this.setState({ hasError: false });
  };

  render() {
    if (this.state.hasError) {
      const title = this.props.fallbackTitle ?? 'Something went wrong';
      const message = this.props.fallbackMessage ?? 'This section encountered an error and cannot be displayed.';
      const retry = this.props.fallbackRetry ?? 'Try Again';

      return (
        <div className="flex flex-col items-center justify-center p-8 gap-4 text-center">
          <div className="w-12 h-12 rounded-full bg-[rgb(var(--color-error)/0.1)] flex items-center justify-center">
            <AlertCircle className="w-6 h-6 text-[rgb(var(--color-error))]" />
          </div>
          <h3 className="text-lg font-semibold text-[rgb(var(--color-text-primary))]">
            {title}
          </h3>
          <p className="text-sm text-[rgb(var(--color-text-secondary))] max-w-md">
            {message}
          </p>
          {this.state.errorMessage && (
            <p className="text-xs font-mono text-red-400 bg-red-900/20 px-3 py-2 rounded max-w-lg break-all">
              {this.state.errorMessage}
            </p>
          )}
          <Button variant="secondary" onClick={this.handleRetry}>
            <RotateCcw className="w-4 h-4 mr-2" />
            {retry}
          </Button>
        </div>
      );
    }

    return this.props.children;
  }
}
