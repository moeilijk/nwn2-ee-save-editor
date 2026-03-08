import { AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { useTranslations } from '@/hooks/useTranslations';
import type { TranslatedError } from '@/lib/api/errors';

interface ErrorModalProps {
  isOpen: boolean;
  error: TranslatedError | null;
  onDismiss: () => void;
  onRetry?: () => void;
}

export function ErrorModal({ isOpen, error, onDismiss, onRetry }: ErrorModalProps) {
  const t = useTranslations();

  if (!isOpen || !error) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
      <div className="bg-[rgb(var(--color-surface-1))] border border-[rgb(var(--color-surface-border))] rounded-xl shadow-2xl w-full max-w-md flex flex-col overflow-hidden">
        <div className="p-6 flex flex-col items-center text-center gap-4">
          <div className="w-12 h-12 rounded-full bg-[rgb(var(--color-error)/0.1)] flex items-center justify-center">
            <AlertCircle className="w-6 h-6 text-[rgb(var(--color-error))]" />
          </div>
          <h2 className="text-lg font-semibold text-[rgb(var(--color-text-primary))]">
            {error.title}
          </h2>
          <p className="text-sm text-[rgb(var(--color-text-secondary))]">
            {error.message}
          </p>
          {error.recovery && (
            <p className="text-xs text-[rgb(var(--color-text-muted))] bg-[rgb(var(--color-surface-2))] rounded-lg px-3 py-2 w-full">
              {error.recovery}
            </p>
          )}
        </div>
        <div className="p-4 border-t border-[rgb(var(--color-surface-border))] flex justify-end gap-2 bg-[rgb(var(--color-surface-2))]">
          {onRetry && (
            <Button variant="secondary" onClick={onRetry}>
              {t('errors.modal.retry')}
            </Button>
          )}
          <Button variant="primary" onClick={onDismiss}>
            {t('errors.modal.dismiss')}
          </Button>
        </div>
      </div>
    </div>
  );
}
