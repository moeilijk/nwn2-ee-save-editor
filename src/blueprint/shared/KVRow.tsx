import { T } from '../theme';

export function KVRow({ label, value, color }: { label: string; value: React.ReactNode; color?: string }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '5px 0' }}>
      <span style={{ color: T.textMuted, fontSize: 13 }}>{label}</span>
      <span style={{ fontWeight: 600, fontSize: 13, color: color || T.text }}>{value}</span>
    </div>
  );
}
