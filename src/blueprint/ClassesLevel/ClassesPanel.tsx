import { HTMLTable, Tag, ProgressBar } from '@blueprintjs/core';
import { T } from '../theme';
import { CHARACTER, LEVEL_HISTORY } from '../dummy-data';
import { SectionBar } from '../shared';

export function ClassesPanel() {
  return (
    <div>
      <div style={{ display: 'flex', gap: 12, padding: '16px 24px' }}>
        {CHARACTER.classes.map(c => (
          <div key={c.name} style={{
            padding: '12px 20px', background: T.surface,
            border: `1px solid ${T.borderLight}`, borderRadius: 4, textAlign: 'center',
          }}>
            <div style={{ fontSize: 18, fontWeight: 700, color: T.text }}>{c.level}</div>
            <div style={{ fontSize: 12, fontWeight: 600, color: T.accent, marginTop: 2 }}>{c.name}</div>
            <div style={{ fontSize: 10, color: T.textMuted }}>d{c.hitDie}</div>
          </div>
        ))}
      </div>

      <div style={{ padding: '0 24px 16px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 4 }}>
          <span style={{ fontSize: 12, color: T.textMuted }}>XP: {CHARACTER.xp.toLocaleString()}</span>
          <span style={{ fontSize: 12, color: T.textMuted }}>Next: {CHARACTER.xpNext.toLocaleString()}</span>
        </div>
        <ProgressBar value={CHARACTER.xp / CHARACTER.xpNext} intent="primary" stripes={false} animate={false} style={{ height: 4 }} />
      </div>

      <SectionBar title="Level Progression" />
      <div style={{ padding: '0 24px' }}>
        <HTMLTable compact striped bordered style={{ width: '100%', tableLayout: 'fixed' }}>
          <colgroup>
            <col style={{ width: 60 }} />
            <col style={{ width: 100 }} />
            <col style={{ width: 80 }} />
            <col />
            <col style={{ width: 80 }} />
            <col style={{ width: 100 }} />
          </colgroup>
          <thead>
            <tr>
              <th style={{ textAlign: 'center' }}>Level</th>
              <th>Class</th>
              <th style={{ textAlign: 'center' }}>HP</th>
              <th>Feats Gained</th>
              <th style={{ textAlign: 'center' }}>Skills</th>
              <th style={{ textAlign: 'center' }}>Ability</th>
            </tr>
          </thead>
          <tbody>
            {LEVEL_HISTORY.map(lv => (
              <tr key={lv.level}>
                <td style={{ textAlign: 'center', fontWeight: 600 }}>{lv.level}</td>
                <td style={{ color: T.accent, fontWeight: 500 }}>{lv.className}</td>
                <td style={{ textAlign: 'center' }}>+{lv.hpGained}</td>
                <td>
                  <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
                    {lv.featsGained.map(f => (
                      <Tag key={f} minimal round style={{ fontSize: 11, background: T.sectionBg, color: T.text, border: `1px solid ${T.sectionBorder}` }}>{f}</Tag>
                    ))}
                    {lv.featsGained.length === 0 && <span style={{ color: T.textMuted, fontSize: 12 }}>-</span>}
                  </div>
                </td>
                <td style={{ textAlign: 'center' }}>{lv.skillPoints}</td>
                <td style={{ textAlign: 'center', color: lv.abilityIncrease ? T.accent : T.textMuted, fontWeight: lv.abilityIncrease ? 600 : 400 }}>
                  {lv.abilityIncrease || '-'}
                </td>
              </tr>
            ))}
          </tbody>
        </HTMLTable>
      </div>
    </div>
  );
}
