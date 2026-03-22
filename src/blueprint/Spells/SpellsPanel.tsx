import { Tag } from '@blueprintjs/core';
import { T } from '../theme';
import { SPELLS } from '../dummy-data';
import { SectionBar } from '../shared';

const SCHOOL_COLORS: Record<string, string> = {
  Abjuration: '#6a7fa0', Conjuration: '#7a9a6a', Enchantment: '#a07a9a',
  Evocation: '#b08040', Transmutation: '#8a8a5a', Necromancy: '#8a5a5a',
  Divination: '#5a8a8a', Illusion: '#9a8a6a',
};

export function SpellsPanel() {
  const caster = SPELLS.casterClasses[0];
  return (
    <div>
      <div style={{ padding: '16px 24px', borderBottom: `1px solid ${T.borderLight}` }}>
        <div style={{ display: 'flex', gap: 24 }}>
          <div>
            <div style={{ fontSize: 12, fontWeight: 600, color: T.textMuted, marginBottom: 4 }}>Caster Class</div>
            <div style={{ fontSize: 15, fontWeight: 700, color: T.accent }}>{caster.className}</div>
          </div>
          <div>
            <div style={{ fontSize: 12, fontWeight: 600, color: T.textMuted, marginBottom: 4 }}>Caster Level</div>
            <div style={{ fontSize: 15, fontWeight: 700, color: T.text }}>{caster.casterLevel}</div>
          </div>
          <div>
            <div style={{ fontSize: 12, fontWeight: 600, color: T.textMuted, marginBottom: 4 }}>Base Spell DC</div>
            <div style={{ fontSize: 15, fontWeight: 700, color: T.text }}>{caster.spellDC}</div>
          </div>
        </div>
        <div style={{ display: 'flex', gap: 6, marginTop: 12 }}>
          {caster.spellsPerDay.map((count, lvl) => count > 0 && (
            <div key={lvl} style={{
              padding: '4px 10px', background: T.sectionBg,
              border: `1px solid ${T.sectionBorder}`, borderRadius: 4, textAlign: 'center',
            }}>
              <div style={{ fontSize: 10, color: T.textMuted }}>Lvl {lvl}</div>
              <div style={{ fontSize: 14, fontWeight: 700, color: T.text }}>{count}</div>
            </div>
          ))}
        </div>
      </div>
      {SPELLS.known.map(group => (
        <div key={group.level}>
          <SectionBar title={group.level === 0 ? 'Cantrips' : `Level ${group.level} Spells`}
            right={<span style={{ fontSize: 11, color: T.textMuted }}>{group.spells.length} known</span>} />
          {group.spells.map(spell => (
            <div key={spell.name} style={{
              display: 'flex', alignItems: 'center', gap: 12,
              padding: '8px 24px', borderBottom: `1px solid ${T.borderLight}`,
            }}>
              <span style={{ fontWeight: 600, fontSize: 13, color: T.text, minWidth: 200 }}>{spell.name}</span>
              <Tag minimal round style={{
                fontSize: 10,
                background: `${SCHOOL_COLORS[spell.school] || T.textMuted}20`,
                color: SCHOOL_COLORS[spell.school] || T.textMuted,
                border: `1px solid ${SCHOOL_COLORS[spell.school] || T.textMuted}40`,
              }}>{spell.school}</Tag>
              <span style={{ flex: 1 }} />
              <span style={{ fontSize: 12, color: T.textMuted }}>{spell.description}</span>
              <div style={{
                padding: '2px 8px', borderRadius: 10, fontSize: 11, fontWeight: 600,
                background: `${T.accent}18`, color: T.accent, marginLeft: 8,
              }}>{spell.memorized}x</div>
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}
