import { Icon } from '@blueprintjs/core';
import { T } from '../theme';
import type { DummyFeat } from '../dummy-data';
import { DetailSection } from '../shared';

const TYPE_COLORS: Record<string, string> = {
  Combat: '#d84315', General: '#43a047', Class: '#1e88e5', Proficiency: '#6d4c41',
  Metamagic: '#8e24aa', Divine: '#f9a825', Background: '#00897b', Racial: '#00acc1',
  Epic: '#e53935',
};

interface FeatDetailProps {
  feat: DummyFeat | null;
}

export function FeatDetail({ feat }: FeatDetailProps) {
  if (!feat) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: T.textMuted }}>
        Select a feat to view details
      </div>
    );
  }

  const typeColor = TYPE_COLORS[feat.type] || T.textMuted;

  return (
    <div style={{ padding: '16px 20px', display: 'flex', flexDirection: 'column', gap: 12 }}>
      {/* Header */}
      <div>
        <span style={{ fontWeight: 700, color: T.text }}>{feat.name}</span>
        <span style={{ color: T.textMuted }}> — </span>
        <span style={{ color: typeColor, fontWeight: 500 }}>{feat.type}</span>
      </div>

      {/* Description */}
      <DetailSection title="Description">
        <div style={{ color: T.text, lineHeight: 1.6 }}>
          {feat.description}
        </div>
      </DetailSection>

      {/* Usage */}
      {feat.use && (
        <DetailSection title="Usage">
          <div style={{ color: T.text, lineHeight: 1.6 }}>
            {feat.use}
          </div>
        </DetailSection>
      )}

      {/* Prerequisites */}
      {feat.prerequisites.length > 0 && (
        <DetailSection title="Prerequisites">
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {feat.prerequisites.map(p => (
              <div key={p.name} style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <Icon
                  icon={p.met ? 'tick' : 'cross'}
                  size={14}
                  style={{ color: p.met ? T.positive : T.negative }}
                />
                <span style={{ color: T.text }}>{p.name}</span>
                {p.current !== undefined && p.required !== undefined && (
                  <span style={{ color: T.textMuted }}>
                    ({p.current}/{p.required})
                  </span>
                )}
              </div>
            ))}
          </div>
        </DetailSection>
      )}
    </div>
  );
}
