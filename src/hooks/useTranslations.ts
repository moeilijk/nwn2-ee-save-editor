import { useCallback } from 'react';
import { useIntl } from 'react-intl';

export function useTranslations() {
  const intl = useIntl();

  return useCallback((key: string, values?: Record<string, unknown>) => {
    const keys = key.split('.');
    let message: unknown = intl.messages;

    for (const k of keys) {
      if (message && typeof message === 'object' && k in message) {
        message = (message as Record<string, unknown>)[k];
      } else {
        return key;
      }
    }

    if (typeof message === 'string') {
      return values ? intl.formatMessage({ id: key, defaultMessage: message }, values as Record<string, string | number>) : message;
    }

    return key;
  }, [intl]);
}