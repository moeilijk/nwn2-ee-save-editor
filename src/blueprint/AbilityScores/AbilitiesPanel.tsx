import { useState } from 'react';
import { Button, Card, Elevation, HTMLTable } from '@blueprintjs/core';
import { T } from '../theme';
import { ABILITIES, SAVES_DETAIL, VITAL_STATS, AC_DETAIL } from '../dummy-data';
import { ModCell, mod, StepInput } from '../shared';
import { RespecDialog } from './RespecDialog';

function SectionLabel({ children }: { children: string }) {
  return (
    <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>
      {children}
    </div>
  );
}

export function AbilitiesPanel() {
  const [scores, setScores] = useState(ABILITIES.map(a => a.base));
  const [isRespecOpen, setIsRespecOpen] = useState(false);
  const [initMisc, setInitMisc] = useState(VITAL_STATS.initiative.base);
  const [acNatural, setAcNatural] = useState(AC_DETAIL[0].natural);

  const initTotal = initMisc + VITAL_STATS.initiative.dexMod + VITAL_STATS.initiative.feats;

  return (
    <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 12 }}>

      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 16, padding: '10px 16px', borderBottom: `1px solid ${T.borderLight}` }}>
          <span style={{ color: T.textMuted, fontSize: 14 }}>Spent: <strong style={{ color: T.text }}>32</strong></span>
          <span style={{ color: T.textMuted, fontSize: 14 }}>Available: <strong style={{ color: T.accent }}>0</strong></span>
          <Button icon="reset" text="Respec" small minimal style={{ color: T.textMuted }} onClick={() => setIsRespecOpen(true)} />
        </div>
        <div style={{ padding: '12px 16px 16px' }}>
          <SectionLabel>Ability Scores</SectionLabel>
          <HTMLTable compact striped bordered interactive style={{ width: '100%', tableLayout: 'fixed' }}>
            <colgroup>
              <col />
              <col style={{ width: 144 }} />
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
                <th style={{ textAlign: 'center' }}>Level</th>
                <th style={{ textAlign: 'center' }}>Racial</th>
                <th style={{ textAlign: 'center' }}>Equip</th>
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
                    <StepInput
                      value={scores[i]}
                      onValueChange={(v) => { const n = [...scores]; n[i] = v; setScores(n); }}
                      min={3} max={50} width={88}
                    />
                  </td>
                  <td style={{ textAlign: 'center' }}><ModCell value={a.level} /></td>
                  <td style={{ textAlign: 'center' }}><ModCell value={a.racial} /></td>
                  <td style={{ textAlign: 'center' }}><ModCell value={a.equip} /></td>
                  <td style={{ textAlign: 'center', fontWeight: 700, fontSize: 15, color: T.text }}>{a.effective}</td>
                  <td style={{
                    textAlign: 'center', fontSize: 12, fontWeight: 600,
                    color: a.modifier > 0 ? T.positive : a.modifier < 0 ? T.negative : T.textMuted,
                  }}>
                    {mod(a.modifier)}
                  </td>
                </tr>
              ))}
            </tbody>
          </HTMLTable>
        </div>
      </Card>

      <Card elevation={Elevation.ONE} style={{ padding: '12px 16px 16px', background: T.surface }}>
        <SectionLabel>Saving Throws & Initiative</SectionLabel>
        <HTMLTable compact striped bordered style={{ width: '100%', tableLayout: 'fixed' }}>
          <colgroup>
            <col />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 112 }} />
            <col style={{ width: 80 }} />
          </colgroup>
          <thead>
            <tr>
              <th></th>
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
                  <strong style={{ fontSize: 15, color: T.text }}>{mod(s.total)}</strong>
                </td>
              </tr>
            ))}
            <tr>
              <td><strong style={{ color: T.text }}>Initiative</strong></td>
              <td />
              <td style={{ textAlign: 'center' }}><ModCell value={VITAL_STATS.initiative.dexMod} /></td>
              <td />
              <td style={{ textAlign: 'center' }}><ModCell value={VITAL_STATS.initiative.feats} /></td>
              <td />
              <td style={{ textAlign: 'center' }}>
                <StepInput value={initMisc} onValueChange={v => setInitMisc(v)} min={-128} max={127} width={88} />
              </td>
              <td style={{ textAlign: 'center' }}>
                <strong style={{ fontSize: 15, color: T.text }}>{mod(initTotal)}</strong>
              </td>
            </tr>
          </tbody>
        </HTMLTable>
      </Card>

      <Card elevation={Elevation.ONE} style={{ padding: '12px 16px 16px', background: T.surface }}>
        <SectionLabel>Armor Class</SectionLabel>
        <HTMLTable compact striped bordered style={{ width: '100%', tableLayout: 'fixed' }}>
          <colgroup>
            <col />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 96 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: 80 }} />
          </colgroup>
          <thead>
            <tr>
              <th>Type</th>
              <th style={{ textAlign: 'center' }}>Base</th>
              <th style={{ textAlign: 'center' }}>DEX</th>
              <th style={{ textAlign: 'center' }}>Armor</th>
              <th style={{ textAlign: 'center' }}>Shield</th>
              <th style={{ textAlign: 'center' }}>Natural</th>
              <th style={{ textAlign: 'center' }}>Dodge</th>
              <th style={{ textAlign: 'center' }}>Deflect</th>
              <th style={{ textAlign: 'center' }}>Size</th>
              <th style={{ textAlign: 'center' }}>Misc</th>
              <th style={{ textAlign: 'center' }}>Total</th>
            </tr>
          </thead>
          <tbody>
            {AC_DETAIL.map(ac => (
              <tr key={ac.name}>
                <td><strong style={{ color: T.text }}>{ac.name}</strong></td>
                <td style={{ textAlign: 'center' }}>{ac.base}</td>
                <td style={{ textAlign: 'center' }}><ModCell value={ac.dex} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={ac.armor} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={ac.shield} /></td>
                <td style={{ textAlign: 'center' }}>
                  {ac.name === 'AC' ? (
                    <StepInput
                      value={acNatural}
                      onValueChange={v => setAcNatural(v)}
                      min={0} max={255} width={68}
                    />
                  ) : (
                    <ModCell value={ac.natural} />
                  )}
                </td>
                <td style={{ textAlign: 'center' }}><ModCell value={ac.dodge} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={ac.deflect} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={ac.size} /></td>
                <td style={{ textAlign: 'center' }}><ModCell value={ac.misc} /></td>
                <td style={{ textAlign: 'center' }}>
                  <strong style={{ fontSize: 15, color: T.text }}>{ac.total}</strong>
                </td>
              </tr>
            ))}
          </tbody>
        </HTMLTable>
      </Card>

      <RespecDialog isOpen={isRespecOpen} onClose={() => setIsRespecOpen(false)} />
    </div>
  );
}
