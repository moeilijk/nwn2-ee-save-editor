import { useState } from 'react';
import { Button, Popover, Menu, MenuItem } from '@blueprintjs/core';
import { T, SPELL_SCHOOL_COLORS } from '../theme';
import type { SpellInfo } from '@/components/Spells/types';
import { DetailSection } from '../shared';
import { display } from '@/utils/dataHelpers';
import { useTranslations } from '@/hooks/useTranslations';
import { useIcon } from '@/hooks/useIcon';
import type { CasterClassInfo } from '@/utils/spellUtils';

interface SpellDetailProps {
  spell: SpellInfo | null;
  memorizedCount?: number;
  isOwned: boolean;
  editableClasses?: CasterClassInfo[];
  onAdd?: (spellId: number, classIndex: number, spellLevel: number) => Promise<void>;
  onRemove?: (spellId: number, classIndex: number, spellLevel: number) => Promise<void>;
}

function InfoRow({ label, value, color }: { label: string; value: string; color?: string }) {
  return (
    <div style={{ display: 'flex' }}>
      <span style={{ color: T.textMuted, minWidth: 180, marginRight: 16 }}>{label}</span>
      <span className="t-medium" style={{ color: color || T.text }}>{value}</span>
    </div>
  );
}

export function SpellDetail({ spell, memorizedCount, isOwned, editableClasses = [], onAdd, onRemove }: SpellDetailProps) {
  const t = useTranslations();
  const [busy, setBusy] = useState(false);
  const iconUrl = useIcon(spell?.icon);

  if (!spell) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: T.textMuted }}>
        Select a spell to view details
      </div>
    );
  }

  const handleAdd = async (cls: CasterClassInfo) => {
    if (!onAdd) return;
    setBusy(true);
    try { await onAdd(spell.id, cls.index, spell.level); } finally { setBusy(false); }
  };

  const handleRemove = async () => {
    if (!onRemove || spell.class_id === undefined) return;
    const cls = editableClasses.find(c => c.class_id === spell.class_id);
    if (!cls) return;
    setBusy(true);
    try { await onRemove(spell.id, cls.index, spell.level); } finally { setBusy(false); }
  };

  const canRemove = isOwned && spell.class_id !== undefined &&
    editableClasses.some(c => c.class_id === spell.class_id);

  const addableClasses = editableClasses.filter(c => c.can_edit_spells);

  const schoolName = spell.school_name || spell.school;
  const schoolColor = SPELL_SCHOOL_COLORS[schoolName || ''] || T.textMuted;

  return (
    <div style={{ padding: '16px 20px', display: 'flex', flexDirection: 'column', gap: 12 }}>

      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          {iconUrl && (
            <img
              src={iconUrl}
              alt=""
              width={32}
              height={32}
              style={{ borderRadius: 4, flexShrink: 0 }}
            />
          )}
          <div>
            <span className="t-bold" style={{ color: T.text }}>{display(spell.name)}</span>
            {schoolName && (
              <>
                <span style={{ color: T.textMuted }}> — </span>
                <span className="t-medium" style={{ color: schoolColor }}>{schoolName}</span>
              </>
            )}
          </div>
        </div>
        <div style={{ display: 'flex', gap: 6, flexShrink: 0 }}>
          {canRemove && onRemove && (
            <Button
              small
              intent="danger"
              icon="trash"
              text={t('placeholders.removeSpell')}
              loading={busy}
              onClick={handleRemove}
            />
          )}
          {!isOwned && onAdd && addableClasses.length > 0 && (
            addableClasses.length === 1 ? (
              <Button
                small
                intent="success"
                icon="plus"
                text={t('placeholders.addSpell')}
                loading={busy}
                onClick={() => handleAdd(addableClasses[0])}
              />
            ) : (
              <Popover
                placement="bottom-end"
                minimal
                content={
                  <Menu>
                    {addableClasses.map(cls => (
                      <MenuItem
                        key={cls.class_id}
                        text={cls.name}
                        onClick={() => handleAdd(cls)}
                      />
                    ))}
                  </Menu>
                }
              >
                <Button
                  small
                  intent="success"
                  icon="plus"
                  rightIcon="caret-down"
                  text={t('placeholders.addSpell')}
                  loading={busy}
                />
              </Popover>
            )
          )}
        </div>
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
          <div className="t-body" style={{ color: T.text }}>
            {spell.description}
          </div>
        </DetailSection>
      )}
    </div>
  );
}
