import { Component, type ReactNode } from 'react';
import { AlertCircle, RotateCcw } from 'lucide-react';
import { Button } from '@/components/ui/Button';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallbackTitle?: string;
  fallbackMessage?: string;
  fallbackRetry?: string;
}

interface ErrorBoundaryState {
  hasError: boolean;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(): ErrorBoundaryState {
    return { hasError: true };
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
