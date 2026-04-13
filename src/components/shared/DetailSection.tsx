import { T } from '../theme';

interface DetailSectionProps {
  title: string;
  children: React.ReactNode;
}

export function DetailSection({ title, children }: DetailSectionProps) {
  return (
    <div style={{ marginBottom: 4 }}>
      <div className="t-semibold" style={{
        color: T.accent,
        paddingBottom: 6,
        borderBottom: `1px solid ${T.borderLight}`,
        marginBottom: 8,
      }}>
        {title}
      </div>
      {children}
    </div>
  );
}
