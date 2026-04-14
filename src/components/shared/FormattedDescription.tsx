import { T } from '../theme';
import type { DescriptionSection } from '@/utils/descriptionParser';

interface FormattedDescriptionProps {
  sections: DescriptionSection[];
}

export function FormattedDescription({ sections }: FormattedDescriptionProps) {
  if (sections.length === 0) return null;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      {sections.map((section, i) => (
        <div key={i}>
          {section.label && (
            <span className="t-semibold" style={{ color: T.textMuted, marginRight: 6 }}>
              {section.label}:
            </span>
          )}
          <span className="t-body" style={{ color: T.text, lineHeight: 1.5 }}>
            {section.text}
          </span>
        </div>
      ))}
    </div>
  );
}
