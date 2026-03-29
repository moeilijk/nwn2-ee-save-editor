import { T, SPELL_SCHOOL_COLORS } from '../theme';
import type { DummySpell } from '../dummy-data';
import { DetailSection } from '../shared';

interface SpellDetailProps {
  spell: (DummySpell & { level: number; memorizedCount?: number }) | null;
  memorizedCount?: number;
}

function InfoRow({ label, value, color }: { label: string; value: string; color?: string }) {
  return (
    <div style={{ display: 'flex' }}>
      <span style={{ color: T.textMuted, minWidth: 180, marginRight: 16 }}>{label}</span>
      <span style={{ color: color || T.text, fontWeight: 500 }}>{value}</span>
    </div>
  );
}

export function SpellDetail({ spell, memorizedCount }: SpellDetailProps) {
  if (!spell) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: T.textMuted }}>
        Select a spell to view details
      </div>
    );
  }

  const schoolColor = SPELL_SCHOOL_COLORS[spell.school] || T.textMuted;

  return (
    <div style={{ padding: '16px 20px', display: 'flex', flexDirection: 'column', gap: 12 }}>

      <div>
        <span style={{ fontWeight: 700, color: T.text }}>{spell.name}</span>
        <span style={{ color: T.textMuted }}> — </span>
        <span style={{ color: schoolColor, fontWeight: 500 }}>{spell.school}</span>
      </div>


      <DetailSection title="Spell Info">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
          {spell.casterLevel && <InfoRow label="Caster Level(s)" value={spell.casterLevel} />}
          <InfoRow label="Innate Level" value={spell.innateLevel !== undefined ? String(spell.innateLevel) : String(spell.level)} />
          <InfoRow label="School" value={spell.school} color={schoolColor} />
          {spell.descriptor && <InfoRow label="Descriptor(s)" value={spell.descriptor} />}
          {spell.components && <InfoRow label="Component(s)" value={spell.components} />}
          {spell.range && <InfoRow label="Range" value={spell.range} />}
          {spell.area && <InfoRow label="Area of Effect / Target" value={spell.area} />}
          {spell.duration && <InfoRow label="Duration" value={spell.duration} />}
          {spell.save && <InfoRow label="Save" value={spell.save} />}
          {spell.spellResistance && <InfoRow label="Spell Resistance" value={spell.spellResistance} />}
          {spell.isDomain && <InfoRow label="Source" value="Domain Spell" color="#c62828" />}
          {memorizedCount !== undefined && memorizedCount > 0 && (
            <InfoRow label="Memorized" value={`${memorizedCount}x`} />
          )}
        </div>
      </DetailSection>


      <DetailSection title="Description">
        <div style={{ color: T.text, lineHeight: 1.6 }}>
          {spell.description}
        </div>
      </DetailSection>
    </div>
  );
}
