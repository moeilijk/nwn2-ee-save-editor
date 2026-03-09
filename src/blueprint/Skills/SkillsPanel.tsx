import { useState } from 'react';
import { HTMLTable, InputGroup, Button, Tag } from '@blueprintjs/core';
import { T } from '../theme';
import { SKILLS } from '../dummy-data';
import { SectionBar, mod } from '../shared';

export function SkillsPanel() {
  const [filter, setFilter] = useState('');
  const filtered = SKILLS.filter(s => !filter || s.name.toLowerCase().includes(filter.toLowerCase()));

  return (
    <div>
      <div style={{ padding: '12px 24px' }}>
        <InputGroup
          leftIcon="search" placeholder="Filter skills..." value={filter}
          onChange={e => setFilter(e.target.value)}
          rightElement={filter ? <Button icon="cross" minimal small onClick={() => setFilter('')} /> : undefined}
          small style={{ maxWidth: 300 }}
        />
      </div>
      <SectionBar title="Skills" right={<span style={{ fontSize: 12, color: T.textMuted }}>Skill Points: <strong style={{ color: T.text }}>0 remaining</strong></span>} />
      <div style={{ padding: '0 24px' }}>
        <HTMLTable compact striped bordered interactive style={{ width: '100%', tableLayout: 'fixed' }}>
          <colgroup>
            <col /><col style={{ width: 72 }} /><col style={{ width: 72 }} /><col style={{ width: 72 }} /><col style={{ width: 80 }} /><col style={{ width: 72 }} /><col style={{ width: 90 }} />
          </colgroup>
          <thead>
            <tr>
              <th>Skill</th>
              <th style={{ textAlign: 'center' }}>Ability</th>
              <th style={{ textAlign: 'center' }}>Total</th>
              <th style={{ textAlign: 'center' }}>Ranks</th>
              <th style={{ textAlign: 'center' }}>Ability Mod</th>
              <th style={{ textAlign: 'center' }}>Misc</th>
              <th style={{ textAlign: 'center' }}>Class Skill</th>
            </tr>
          </thead>
          <tbody>
            {filtered.map(s => (
              <tr key={s.name}>
                <td><strong style={{ color: T.text }}>{s.name}</strong></td>
                <td style={{ textAlign: 'center', color: T.textMuted, fontSize: 12 }}>{s.ability}</td>
                <td style={{ textAlign: 'center', fontWeight: 700, color: s.total > 0 ? T.text : T.textMuted }}>{mod(s.total)}</td>
                <td style={{ textAlign: 'center' }}>{s.ranks}</td>
                <td style={{ textAlign: 'center', color: s.abilityMod > 0 ? T.positive : s.abilityMod < 0 ? T.negative : T.textMuted }}>{mod(s.abilityMod)}</td>
                <td style={{ textAlign: 'center', color: T.textMuted }}>{s.misc || '-'}</td>
                <td style={{ textAlign: 'center' }}>
                  {s.isClassSkill
                    ? <Tag minimal style={{ background: 'rgba(90,122,90,0.15)', color: T.positive, fontSize: 10 }}>Class</Tag>
                    : <span style={{ color: T.textMuted, fontSize: 11 }}>Cross</span>}
                </td>
              </tr>
            ))}
          </tbody>
        </HTMLTable>
      </div>
    </div>
  );
}
