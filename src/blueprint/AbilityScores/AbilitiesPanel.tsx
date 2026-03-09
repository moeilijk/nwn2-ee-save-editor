import { useState } from 'react';
import { Button, HTMLTable, NumericInput, Tag } from '@blueprintjs/core';
import { T } from '../theme';
import { ABILITIES, SAVES_DETAIL } from '../dummy-data';
import { SectionBar, ModCell, mod } from '../shared';

export function AbilitiesPanel() {
  const [scores, setScores] = useState(ABILITIES.map(a => a.base));

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 16, padding: '12px 24px', borderBottom: `1px solid ${T.borderLight}` }}>
        <span style={{ fontSize: 13, fontWeight: 700, color: T.accent }}>Point Buy</span>
        <div style={{ width: 1, height: 16, background: T.border }} />
        <span style={{ color: T.textMuted, fontSize: 12 }}>Spent: <strong style={{ color: T.text }}>32</strong></span>
        <span style={{ color: T.textMuted, fontSize: 12 }}>Available: <strong style={{ color: T.accent }}>0</strong></span>
        <div style={{ flex: 1 }} />
        <Button icon="reset" text="Reset" small minimal style={{ color: T.textMuted }} />
      </div>

      <SectionBar title="Ability Scores" />
      <div style={{ padding: '0 24px' }}>
        <HTMLTable compact striped bordered interactive style={{ width: '100%', tableLayout: 'fixed' }}>
          <colgroup>
            <col />
            <col style={{ width: 80 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 80 }} />
            <col style={{ width: 72 }} />
          </colgroup>
          <thead>
            <tr>
              <th>Ability</th>
              <th style={{ textAlign: 'center' }}>Base</th>
              <th style={{ textAlign: 'center' }}>Racial</th>
              <th style={{ textAlign: 'center' }}>Equip</th>
              <th style={{ textAlign: 'center' }}>Level</th>
              <th style={{ textAlign: 'center' }}>Enhance</th>
              <th style={{ textAlign: 'center' }}>Effective</th>
              <th style={{ textAlign: 'center' }}>Mod</th>
            </tr>
          </thead>
          <tbody>
            {ABILITIES.map((a, i) => (
              <tr key={a.name}>
                <td>
                  <strong style={{ color: T.text }}>{a.name}</strong>
                  <span style={{ marginLeft: 6, fontSize: 12, color: T.textMuted }}>{a.full}</span>
                </td>
                <td style={{ textAlign: 'center' }}>
                  <NumericInput
                    value={scores[i]}
                    onValueChange={(v) => { const n = [...scores]; n[i] = v; setScores(n); }}
                    min={3} max={50} style={{ width: 56 }} buttonPosition="none" fill={false} small
                  />
                </td>
                <td style={{ textAlign: 'center' }}><ModCell value={a.racial} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={a.equip} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={a.level} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={a.enhance} /></td>
                <td style={{ textAlign: 'center', fontWeight: 700, fontSize: 15, color: T.text }}>{a.effective}</td>
                <td style={{ textAlign: 'center' }}>
                  <span style={{
                    display: 'inline-block',
                    padding: '2px 8px',
                    borderRadius: 10,
                    fontSize: 12,
                    fontWeight: 600,
                    background: a.modifier > 0 ? '#e8f5e8' : a.modifier < 0 ? '#fde8e8' : T.sectionBg,
                    color: a.modifier > 0 ? T.positive : a.modifier < 0 ? T.negative : T.textMuted,
                  }}>
                    {mod(a.modifier)}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </HTMLTable>
      </div>

      <SectionBar title="Saving Throws" />
      <div style={{ padding: '0 24px' }}>
        <HTMLTable compact striped bordered style={{ width: '100%', tableLayout: 'fixed' }}>
          <colgroup>
            <col />
            <col style={{ width: 80 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 80 }} />
            <col style={{ width: 72 }} />
          </colgroup>
          <thead>
            <tr>
              <th>Save</th>
              <th style={{ textAlign: 'center' }}>Base</th>
              <th style={{ textAlign: 'center' }}>Ability</th>
              <th style={{ textAlign: 'center' }}>Equip</th>
              <th style={{ textAlign: 'center' }}>Feat</th>
              <th style={{ textAlign: 'center' }}>Racial</th>
              <th style={{ textAlign: 'center' }}>Misc</th>
              <th style={{ textAlign: 'center' }}>Total</th>
            </tr>
          </thead>
          <tbody>
            {SAVES_DETAIL.map(s => (
              <tr key={s.name}>
                <td><strong style={{ color: T.text }}>{s.name}</strong></td>
                <td style={{ textAlign: 'center' }}>{mod(s.base)}</td>
                <td style={{ textAlign: 'center' }}><ModCell value={s.ability} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={s.equip} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={s.feat} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={s.racial} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={s.misc} /></td>
                <td style={{ textAlign: 'center' }}>
                  <span style={{
                    display: 'inline-block', padding: '2px 10px', borderRadius: 10,
                    fontWeight: 700, fontSize: 13,
                    background: `${T.accent}18`, color: T.accent,
                  }}>
                    {mod(s.total)}
                  </span>
                </td>
              </tr>
            ))}
          </tbody>
        </HTMLTable>
      </div>

      <SectionBar title="Alignment" />
      <div style={{ display: 'flex', gap: 32, padding: '16px 24px' }}>
        <div>
          <div style={{ fontSize: 11, fontWeight: 600, color: T.textMuted, textTransform: 'uppercase', marginBottom: 6 }}>Law - Chaos</div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <NumericInput value={25} min={0} max={100} style={{ width: 64 }} small />
            <Tag minimal round style={{ background: T.sectionBg, color: T.text, border: `1px solid ${T.sectionBorder}` }}>Chaotic</Tag>
          </div>
        </div>
        <div>
          <div style={{ fontSize: 11, fontWeight: 600, color: T.textMuted, textTransform: 'uppercase', marginBottom: 6 }}>Good - Evil</div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <NumericInput value={85} min={0} max={100} style={{ width: 64 }} small />
            <Tag minimal round style={{ background: '#e8f5e8', color: T.positive, border: '1px solid #c8e0c8' }}>Good</Tag>
          </div>
        </div>
      </div>
    </div>
  );
}
