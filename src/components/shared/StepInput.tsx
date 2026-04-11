import { Button } from '@blueprintjs/core';
import { T } from '../theme';

interface StepInputProps {
  value: number;
  onValueChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  width?: number;
  disabled?: boolean;
}

export function StepInput({
  value,
  onValueChange,
  min = -Infinity,
  max = Infinity,
  step = 1,
  width = 88,
  disabled = false,
}: StepInputProps) {
  const clamp = (v: number) => Math.max(min, Math.min(max, v));

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const raw = e.target.value;
    if (raw === '' || raw === '-') return;
    const parsed = Number(raw);
    if (!isNaN(parsed)) onValueChange(clamp(parsed));
  };

  return (
    <div style={{ display: 'inline-flex', alignItems: 'center', gap: 0, width }}>
      <Button
        small
        minimal
        icon="minus"
        disabled={disabled || value <= min}
        onClick={() => onValueChange(clamp(value - step))}
        className="step-input-btn"
        style={{ minWidth: 22, minHeight: 22, padding: 0 }}
      />
      <input
        type="text"
        inputMode="numeric"
        value={value}
        onChange={handleChange}
        disabled={disabled}
        style={{
          flex: 1,
          minWidth: 0,
          textAlign: 'center',
          fontSize: 14,
          fontWeight: 600,
          fontFamily: 'inherit',
          color: T.text,
          background: T.surface,
          border: `1px solid ${T.borderLight}`,
          borderRadius: 3,
          padding: '2px 4px',
          height: 24,
          outline: 'none',
        }}
      />
      <Button
        small
        minimal
        icon="plus"
        disabled={disabled || value >= max}
        onClick={() => onValueChange(clamp(value + step))}
        className="step-input-btn"
        style={{ minWidth: 22, minHeight: 22, padding: 0 }}
      />
    </div>
  );
}
