import { T } from '../theme';

export function SectionBar({ title, right }: { title: string; right?: React.ReactNode }) {
  return (
    <div style={{
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'space-between',
      padding: '8px 16px',
      background: T.sectionBg,
      borderTop: `1px solid ${T.sectionBorder}`,
      borderBottom: `1px solid ${T.sectionBorder}`,
      marginBottom: 0,
    }}>
      <span className="t-section">{title}</span>
      {right}
    </div>
  );
}
