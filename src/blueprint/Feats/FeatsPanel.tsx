import { useState } from 'react';
import { InputGroup, Button } from '@blueprintjs/core';
import { T } from '../theme';
import { FEATS } from '../dummy-data';
import { SectionBar } from '../shared';

function FeatRow({ name, description }: { name: string; description: string }) {
  return (
    <div style={{
      display: 'flex', alignItems: 'baseline', gap: 12,
      padding: '8px 24px', borderBottom: `1px solid ${T.borderLight}`,
    }}>
      <span style={{ fontWeight: 600, fontSize: 13, color: T.text, minWidth: 200 }}>{name}</span>
      <span style={{ fontSize: 12, color: T.textMuted }}>{description}</span>
    </div>
  );
}

export function FeatsPanel() {
  const [filter, setFilter] = useState('');
  const match = (name: string) => !filter || name.toLowerCase().includes(filter.toLowerCase());

  const sections = [
    { title: 'General Feats', feats: FEATS.general },
    { title: 'Class Bonus Feats', feats: FEATS.classBonusFeats },
    { title: 'Background Feats', feats: FEATS.background },
    { title: 'Racial Feats', feats: FEATS.racial },
  ];

  return (
    <div>
      <div style={{ padding: '12px 24px' }}>
        <InputGroup leftIcon="search" placeholder="Filter feats..." value={filter}
          onChange={e => setFilter(e.target.value)}
          rightElement={filter ? <Button icon="cross" minimal small onClick={() => setFilter('')} /> : undefined}
          small style={{ maxWidth: 300 }}
        />
      </div>
      {sections.map(section => {
        const filtered = section.feats.filter(f => match(f.name));
        if (filtered.length === 0) return null;
        return (
          <div key={section.title}>
            <SectionBar title={`${section.title} (${filtered.length})`} />
            {filtered.map(f => <FeatRow key={f.name} name={f.name} description={f.description} />)}
          </div>
        );
      })}
    </div>
  );
}
