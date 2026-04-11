import { Tag } from '@blueprintjs/core';
import { T } from '../theme';
import type { DummyFeat } from '../dummy-data';

interface FeatExpandedDetailProps {
  feat: DummyFeat;
}

export function FeatExpandedDetail({ feat }: FeatExpandedDetailProps) {
  return (
    <div>
      <div style={{ fontSize: 12, color: T.text, lineHeight: 1.5, marginBottom: feat.prerequisites.length > 0 ? 10 : 0 }}>
        {feat.description}
      </div>
      {feat.prerequisites.length > 0 && (
        <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
          {feat.prerequisites.map(p => (
            <Tag
              key={p.name} minimal round
              icon={p.met ? 'tick' : 'cross'}
              style={{
                fontSize: 11,
                color: p.met ? T.positive : T.negative,
                background: p.met ? `${T.positive}12` : `${T.negative}12`,
                border: `1px solid ${p.met ? T.positive : T.negative}30`,
              }}
            >
              {p.name}
              {p.current !== undefined && p.required !== undefined && (
                <span style={{ marginLeft: 4, opacity: 0.7 }}>({p.current}/{p.required})</span>
              )}
            </Tag>
          ))}
        </div>
      )}
    </div>
  );
}
