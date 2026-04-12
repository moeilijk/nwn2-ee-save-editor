import { Component, type ReactNode } from 'react';
import { Button, NonIdealState } from '@blueprintjs/core';
import { GiBrokenShield } from 'react-icons/gi';
import { T } from '../theme';
import { GameIcon } from './GameIcon';

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
          icon={<GameIcon icon={GiBrokenShield} size={40} color={T.negative} />}
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
