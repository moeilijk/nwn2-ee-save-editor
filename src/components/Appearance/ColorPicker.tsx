import { useRef, useCallback } from 'react';
import { T } from '../theme';
import type { TintChannel } from '@/lib/bindings';

interface ColorPickerProps {
  label: string;
  value: TintChannel;
  onChange: (value: TintChannel) => void;
}

function tintToHex(c: TintChannel): string {
  const hex = (n: number) => n.toString(16).padStart(2, '0');
  return `#${hex(c.r)}${hex(c.g)}${hex(c.b)}`;
}

function hexToTint(hex: string, a: number): TintChannel {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return { r, g, b, a };
}

export function ColorPicker({ label, value, onChange }: ColorPickerProps) {
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const handleChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const hex = e.target.value;
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(() => {
      onChange(hexToTint(hex, value.a));
    }, 300);
  }, [onChange, value.a]);

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
      <input
        type="color"
        value={tintToHex(value)}
        onChange={handleChange}
        style={{
          width: 28,
          height: 28,
          padding: 0,
          border: `1px solid ${T.borderLight}`,
          borderRadius: 3,
          cursor: 'pointer',
          background: 'none',
        }}
      />
      <span style={{ fontSize: 12, color: T.textMuted }}>{label}</span>
    </div>
  );
}
