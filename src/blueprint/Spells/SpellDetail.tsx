import { T, SPELL_SCHOOL_COLORS } from '../theme';
import type { SpellInfo } from '@/components/Spells/types';
import { DetailSection } from '../shared';
import { display } from '@/utils/dataHelpers';

interface SpellDetailProps {
  spell: SpellInfo | null;
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

  const schoolName = spell.school_name || spell.school;
  const schoolColor = SPELL_SCHOOL_COLORS[schoolName || ''] || T.textMuted;

  return (
    <div style={{ padding: '16px 20px', display: 'flex', flexDirection: 'column', gap: 12 }}>

      <div>
        <span style={{ fontWeight: 700, color: T.text }}>{display(spell.name)}</span>
        {schoolName && (
          <>
            <span style={{ color: T.textMuted }}> — </span>
            <span style={{ color: schoolColor, fontWeight: 500 }}>{schoolName}</span>
          </>
        )}
      </div>

      <DetailSection title="Spell Info">
        <div style={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
          <InfoRow label="Innate Level" value={spell.innate_level !== undefined ? String(spell.innate_level) : String(spell.level)} />
          {schoolName && <InfoRow label="School" value={schoolName} color={schoolColor} />}
          {spell.components && <InfoRow label="Component(s)" value={spell.components} />}
          {spell.range && <InfoRow label="Range" value={spell.range} />}
          {spell.target_type && <InfoRow label="Target" value={spell.target_type} />}
          {spell.cast_time && <InfoRow label="Cast Time" value={spell.cast_time} />}
          {spell.available_metamagic && <InfoRow label="Metamagic" value={spell.available_metamagic} />}
          {spell.is_domain_spell && <InfoRow label="Source" value="Domain Spell" color="#c62828" />}
          {memorizedCount !== undefined && memorizedCount > 0 && (
            <InfoRow label="Memorized" value={`${memorizedCount}x`} />
          )}
          {spell.available_classes && spell.available_classes.length > 0 && (
            <InfoRow label="Available To" value={spell.available_classes.join(', ')} />
          )}
        </div>
      </DetailSection>

      {spell.description && (
        <DetailSection title="Description">
          <div style={{ color: T.text, lineHeight: 1.6 }}>
            {spell.description}
          </div>
        </DetailSection>
      )}
    </div>
  );
}
