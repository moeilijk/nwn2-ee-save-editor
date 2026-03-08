
import { createContext, useContext, useCallback, ReactNode } from 'react';
import { toast, Toaster } from 'sonner';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

interface ToastContextType {
  showToast: (message: string, type?: ToastType, duration?: number, description?: string) => void;
}

const ToastContext = createContext<ToastContextType | undefined>(undefined);

const toastFn: Record<ToastType, typeof toast.success> = {
  success: toast.success,
  error: toast.error,
  warning: toast.warning,
  info: toast.info,
};

export function ToastProvider({ children }: { children: ReactNode }) {
  const showToast = useCallback((message: string, type: ToastType = 'info', duration = 5000, description?: string) => {
    toastFn[type](message, { description, duration });
  }, []);

  return (
    <ToastContext.Provider value={{ showToast }}>
      {children}
      <Toaster
        position="bottom-right"
        toastOptions={{
          style: {
            background: 'rgb(var(--color-surface-3))',
            border: '1px solid rgb(var(--color-border))',
            color: 'rgb(var(--color-text-primary))',
            fontSize: '13px',
          },
          classNames: {
            description: 'toast-description',
            success: 'toast-success',
            error: 'toast-error',
            warning: 'toast-warning',
            info: 'toast-info',
          },
        }}
      />
    </ToastContext.Provider>
  );
}

export function useToast() {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within ToastProvider');
  }
  return context;
}
