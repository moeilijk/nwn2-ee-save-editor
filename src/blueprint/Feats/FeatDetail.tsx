import { Icon } from '@blueprintjs/core';
import { T, FEAT_TYPE_COLORS } from '../theme';
import type { DummyFeat } from '../dummy-data';
import { DetailSection } from '../shared';

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

  const typeColor = FEAT_TYPE_COLORS[feat.type] || T.textMuted;

  return (
    <div style={{ padding: '16px 20px', display: 'flex', flexDirection: 'column', gap: 12 }}>

      <div>
        <span style={{ fontWeight: 700, color: T.text }}>{feat.name}</span>
        <span style={{ color: T.textMuted }}> — </span>
        <span style={{ color: typeColor, fontWeight: 500 }}>{feat.type}</span>
      </div>


      <DetailSection title="Description">
        <div style={{ color: T.text, lineHeight: 1.6 }}>
          {feat.description}
        </div>
      </DetailSection>


      {feat.use && (
        <DetailSection title="Usage">
          <div style={{ color: T.text, lineHeight: 1.6 }}>
            {feat.use}
          </div>
        </DetailSection>
      )}


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
