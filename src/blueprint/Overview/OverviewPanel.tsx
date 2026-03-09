import { Button, H4, ProgressBar, Tag } from '@blueprintjs/core';
import { T } from '../theme';
import { CHARACTER, ABILITIES } from '../dummy-data';
import { SectionBar, KVRow, mod, fmtNum } from '../shared';

export function OverviewPanel() {
  const hpPct = CHARACTER.hp.current / CHARACTER.hp.max;

  return (
    <div>
      <div style={{ padding: '20px 24px 16px' }}>
        <div style={{ display: 'flex', alignItems: 'baseline', gap: 10, marginBottom: 4 }}>
          <H4 style={{ margin: 0, color: T.text }}>{CHARACTER.name}</H4>
          <Button icon="edit" minimal small style={{ color: T.textMuted }} />
        </div>
        <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          {CHARACTER.classes.map((c, i) => (
            <Tag key={i} minimal round style={{ background: T.sectionBg, color: T.accent, border: `1px solid ${T.sectionBorder}` }}>
              {c.name} {c.level}
            </Tag>
          ))}
          <span style={{ color: T.textMuted, fontSize: 12 }}>Level {CHARACTER.level}</span>
        </div>
      </div>

      <SectionBar title="Character" />
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', padding: '0 24px' }}>
        <div style={{ borderRight: `1px solid ${T.borderLight}`, paddingRight: 24, paddingTop: 12, paddingBottom: 12 }}>
          <KVRow label="Race" value={CHARACTER.race} />
          <KVRow label="Gender / Age" value={`${CHARACTER.gender}, ${CHARACTER.age} yrs`} />
          <KVRow label="Alignment" value={CHARACTER.alignment} />
          <KVRow label="Deity" value={<>{CHARACTER.deity} <Button icon="edit" minimal small style={{ color: T.textMuted, marginLeft: 4 }} /></>} />
          <KVRow label="Background" value={CHARACTER.background} />
        </div>
        <div style={{ paddingLeft: 24, paddingTop: 12, paddingBottom: 12 }}>
          <KVRow label="Hit Points" value={<>{CHARACTER.hp.current}<span style={{ color: T.textMuted }}> / {CHARACTER.hp.max}</span></>} />
          <ProgressBar value={hpPct} intent={hpPct > 0.5 ? 'success' : 'warning'} stripes={false} animate={false} style={{ height: 3, marginBottom: 8 }} />
          <KVRow label="Armor Class" value={CHARACTER.ac} />
          <KVRow label="Experience" value={fmtNum(CHARACTER.xp)} />
          <KVRow label="Gold" value={fmtNum(CHARACTER.gold)} color={T.gold} />
          <KVRow label="Speed" value={`${CHARACTER.speed} ft`} />
        </div>
      </div>

      <SectionBar title="Ability Scores" />
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(6, 1fr)', padding: '16px 24px', gap: 8 }}>
        {ABILITIES.map(a => (
          <div key={a.name} style={{
            textAlign: 'center',
            padding: '12px 8px',
            background: T.surface,
            border: `1px solid ${T.borderLight}`,
            borderRadius: 4,
          }}>
            <div style={{ fontSize: 10, fontWeight: 700, letterSpacing: '0.1em', color: T.textMuted, marginBottom: 4 }}>{a.name}</div>
            <div style={{ fontSize: 26, fontWeight: 700, color: T.text, lineHeight: 1 }}>{a.effective}</div>
            <div style={{ fontSize: 12, fontWeight: 600, color: a.modifier > 0 ? T.positive : a.modifier < 0 ? T.negative : T.textMuted, marginTop: 4 }}>
              {mod(a.modifier)}
            </div>
            <div style={{ fontSize: 10, color: T.textMuted, marginTop: 2 }}>base {a.base}</div>
          </div>
        ))}
      </div>

      <SectionBar title="Combat" />
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', padding: '12px 24px', gap: 16, textAlign: 'center' }}>
        {[
          { label: 'BAB', value: mod(CHARACTER.bab) },
          { label: 'Melee', value: mod(CHARACTER.melee) },
          { label: 'Ranged', value: mod(CHARACTER.ranged) },
          { label: 'Initiative', value: mod(CHARACTER.initiative) },
        ].map(s => (
          <div key={s.label}>
            <div style={{ fontSize: 10, fontWeight: 700, letterSpacing: '0.08em', color: T.textMuted, marginBottom: 4 }}>{s.label}</div>
            <div style={{ fontSize: 20, fontWeight: 700, color: T.text }}>{s.value}</div>
          </div>
        ))}
      </div>

      <div style={{ display: 'flex', gap: 12, padding: '0 24px 16px', justifyContent: 'center' }}>
        {[
          { label: 'Fort', value: CHARACTER.saves.fort },
          { label: 'Ref', value: CHARACTER.saves.ref },
          { label: 'Will', value: CHARACTER.saves.will },
        ].map(s => (
          <div key={s.label} style={{
            display: 'flex', alignItems: 'center', gap: 6,
            padding: '6px 14px',
            background: T.sectionBg,
            border: `1px solid ${T.sectionBorder}`,
            borderRadius: 4,
          }}>
            <span style={{ fontSize: 11, fontWeight: 700, color: T.textMuted, textTransform: 'uppercase' }}>{s.label}</span>
            <span style={{ fontSize: 16, fontWeight: 700, color: T.positive }}>{mod(s.value)}</span>
          </div>
        ))}
      </div>

      <SectionBar title="Biography" right={<Button icon="edit" minimal small style={{ color: T.textMuted }} />} />
      <div style={{ padding: '16px 24px' }}>
        <p style={{ margin: 0, fontSize: 13, lineHeight: 1.7, color: T.textMuted, fontStyle: 'italic' }}>
          {CHARACTER.biography}
        </p>
      </div>
    </div>
  );
}
