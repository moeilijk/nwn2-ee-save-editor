import { createContext, useContext, useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';

type FontSize = 'small' | 'medium' | 'large';

const VALID_SIZES: FontSize[] = ['small', 'medium', 'large'];

const CSS_CLASSES: Record<FontSize, string | null> = {
  small: 'font-size-small',
  medium: null,
  large: 'font-size-large',
};

interface FontSizeContextValue {
  fontSize: FontSize;
  setFontSize: (size: FontSize) => void;
}

const FontSizeContext = createContext<FontSizeContextValue>({
  fontSize: 'medium',
  setFontSize: () => {},
});

export const useFontSize = () => useContext(FontSizeContext);

function applyFontSizeClass(size: FontSize) {
  const root = document.documentElement;
  root.classList.remove('font-size-small', 'font-size-large');
  const cls = CSS_CLASSES[size];
  if (cls) root.classList.add(cls);
}

export function FontSizeProvider({ children }: { children: React.ReactNode }) {
  const [fontSize, setFontSizeState] = useState<FontSize>('medium');

  useEffect(() => {
    invoke<{ font_size: string }>('get_app_config')
      .then((config) => {
        const size = VALID_SIZES.includes(config.font_size as FontSize)
          ? (config.font_size as FontSize)
          : 'medium';
        setFontSizeState(size);
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    applyFontSizeClass(fontSize);
  }, [fontSize]);

  const setFontSize = useCallback((size: FontSize) => {
    setFontSizeState(size);
    invoke('update_app_config', { updates: { font_size: size } }).catch(() => {});
  }, []);

  const value = useMemo(() => ({ fontSize, setFontSize }), [fontSize, setFontSize]);

  return (
    <FontSizeContext.Provider value={value}>
      {children}
    </FontSizeContext.Provider>
  );
}
