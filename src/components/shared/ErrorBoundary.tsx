import { Component, type ReactNode } from 'react';
import { Button, Icon, NonIdealState } from '@blueprintjs/core';
import { T } from '../theme';

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
      return (
        <NonIdealState
          icon={<Icon icon="error" size={40} color={T.negative} />}
          title={this.props.fallbackTitle ?? 'Something went wrong'}
          description={this.props.fallbackMessage ?? 'This section encountered an error and cannot be displayed.'}
          action={
            <Button icon="refresh" text={this.props.fallbackRetry ?? 'Try Again'} onClick={this.handleRetry} />
          }
        />
      );
    }
    return this.props.children;
  }
}
