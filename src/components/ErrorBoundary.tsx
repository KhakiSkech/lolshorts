import React, { Component, ErrorInfo, ReactNode } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { AlertTriangle, RefreshCw } from 'lucide-react';

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

/**
 * Error Boundary Component
 *
 * Catches JavaScript errors in child component tree,
 * logs error information, and displays fallback UI
 *
 * Security: Does not expose sensitive error details in production
 * Accessibility: Provides clear error messaging and recovery options
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  private retryCount = 0;
  private maxRetries = 3;

  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return {
      hasError: true,
      error,
      errorInfo: null,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    this.setState({
      error,
      errorInfo,
    });

    // Log error for debugging (in production, send to error tracking service)
    console.error('Error Boundary caught an error:', error, errorInfo);

    // Call error handler if provided
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }

    // In production, you might send error to monitoring service
    if (import.meta.env.PROD) {
      // Example: sendToErrorTracking(error, errorInfo);
    }
  }

  handleRetry = () => {
    if (this.retryCount < this.maxRetries) {
      this.retryCount++;
      this.setState({
        hasError: false,
        error: null,
        errorInfo: null,
      });
    }
  };

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (this.state.hasError) {
      // Custom fallback UI
      if (this.props.fallback) {
        return <>{this.props.fallback}</>;
      }

      // Default error UI
      return (
        <div className="min-h-screen flex items-center justify-center p-4 bg-background">
          <Card className="w-full max-w-md border-destructive">
            <CardHeader className="text-center">
              <div className="mx-auto w-12 h-12 bg-destructive/10 rounded-full flex items-center justify-center mb-4">
                <AlertTriangle className="h-6 w-6 text-destructive" />
              </div>
              <CardTitle className="text-destructive">예기치 못한 오류</CardTitle>
              <CardDescription>
                애플리케이션에서 오류가 발생했습니다
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {import.meta.env.DEV && this.state.error && (
                <div className="bg-muted p-3 rounded-lg text-xs">
                  <p className="font-mono text-destructive mb-2">
                    {this.state.error.name}: {this.state.error.message}
                  </p>
                  {this.state.errorInfo && (
                    <p className="text-muted-foreground">
                      {this.state.errorInfo.componentStack}
                    </p>
                  )}
                </div>
              )}

              <div className="space-y-2">
                {this.retryCount < this.maxRetries && (
                  <Button
                    onClick={this.handleRetry}
                    className="w-full"
                    variant="default"
                    aria-label="다시 시도"
                  >
                    <RefreshCw className="h-4 w-4 mr-2" />
                    다시 시도 ({this.maxRetries - this.retryCount}회 남음)
                  </Button>
                )}

                <Button
                  onClick={this.handleReload}
                  className="w-full"
                  variant="outline"
                  aria-label="페이지 새로고침"
                >
                  페이지 새로고침
                </Button>
              </div>

              <p className="text-xs text-muted-foreground text-center">
                문제가 계속되면 고객지원에 문의하세요
              </p>
            </CardContent>
          </Card>
        </div>
      );
    }

    return this.props.children;
  }
}

// Hook for functional components
export function useErrorHandler() {
  const handleError = React.useCallback((error: Error, errorInfo?: string) => {
    console.error('Caught error:', error, errorInfo);

    // In production, send to error tracking service
    if (import.meta.env.PROD) {
      // Example: sendToErrorTracking(error, errorInfo);
    }
  }, []);

  return { handleError };
}

// Specialized Error Boundaries for different contexts

export function VideoErrorBoundary({ children }: { children: ReactNode }) {
  const handleError = React.useCallback((error: Error) => {
    console.error('Video playback error:', error);
    // Could send to video-specific error tracking
  }, []);

  return (
    <ErrorBoundary
      onError={handleError}
      fallback={
        <Card className="m-4">
          <CardContent className="p-6 text-center">
            <AlertTriangle className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
            <h3 className="font-medium mb-2">비디오 재생 오류</h3>
            <p className="text-sm text-muted-foreground mb-4">
              비디오를 재생하는 중 문제가 발생했습니다
            </p>
            <Button size="sm" onClick={() => window.location.reload()}>
              새로고침
            </Button>
          </CardContent>
        </Card>
      }
    >
      {children}
    </ErrorBoundary>
  );
}

export function FormErrorBoundary({ children }: { children: ReactNode }) {
  const handleError = React.useCallback((error: Error) => {
    console.error('Form error:', error);
    // Could send to form-specific error tracking
  }, []);

  return (
    <ErrorBoundary
      onError={handleError}
      fallback={
        <Card className="m-4 border-destructive">
          <CardContent className="p-4">
            <div className="flex items-center gap-2 text-destructive">
              <AlertTriangle className="h-4 w-4" />
              <span className="text-sm font-medium">양식 오류</span>
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              양식을 처리하는 중 문제가 발생했습니다
            </p>
          </CardContent>
        </Card>
      }
    >
      {children}
    </ErrorBoundary>
  );
}