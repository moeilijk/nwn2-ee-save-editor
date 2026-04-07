import { T } from '../theme';

export function mod(n: number): string {
  return n >= 0 ? `+${n}` : `${n}`;
}

export function fmtNum(n: number): string {
  return n.toLocaleString();
}

export function ModCell({ value }: { value: number }) {
  if (value === 0) return <span style={{ color: T.textMuted }}>-</span>;
  return <span style={{ color: value > 0 ? T.positive : T.negative, fontWeight: 500 }}>{mod(value)}</span>;
}
