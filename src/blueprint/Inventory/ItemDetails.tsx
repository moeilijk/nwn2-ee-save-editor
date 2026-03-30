import { Button, ButtonGroup, H4, Icon } from '@blueprintjs/core';
import { useTranslations } from '@/hooks/useTranslations';
import { T } from '../theme';
import { fmtNum } from '../shared';
import type { FullEquippedItem, FullInventoryItem } from '@/lib/bindings';
import { display } from '@/utils/dataHelpers';

export type AnyItem =
  | (FullEquippedItem & { _kind: 'equipped' })
  | (FullInventoryItem & { _kind: 'inventory' });

export function makeEquippedItem(item: FullEquippedItem): AnyItem {
  return { ...item, _kind: 'equipped' };
}

export function makeInventoryItem(item: FullInventoryItem): AnyItem {
  return { ...item, _kind: 'inventory' };
}

function isEquippedItem(item: AnyItem): item is FullEquippedItem & { _kind: 'equipped' } {
  return item._kind === 'equipped';
}

interface ItemDetailsProps {
  item: AnyItem | null;
  onEdit?: () => void;
  onUnequip?: (slot: string) => void;
  onDelete?: (index: number) => void;
}

export function ItemDetails({ item, onEdit, onUnequip, onDelete }: ItemDetailsProps) {
  const t = useTranslations();

  if (!item) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
        <div style={{ textAlign: 'center' }}>
          <Icon icon="hand-right" size={36} style={{ color: T.border }} />
          <p style={{ marginTop: 10, color: T.textMuted }}>Select an item</p>
          <p style={{ marginTop: 2, color: T.border }}>Choose from equipped or backpack</p>
        </div>
      </div>
    );
  }

  const equipped = isEquippedItem(item);
  const name = display(item.name);
  const description = item.description;
  const weight = item.weight;
  const value = item.value;
  const baseItemName = item.base_item_name;
  const properties = item.decoded_properties;
  const baseAc = item.base_ac;

  return (
    <div>
      <div style={{ padding: '14px 16px 12px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
          <div>
            <H4 style={{ margin: 0, color: T.text }}>{name}</H4>
            <span style={{ color: T.textMuted }}>
              {equipped ? `${item.slot} \u00b7 ${baseItemName}` : baseItemName}
            </span>
          </div>
          {equipped && (
            <ButtonGroup minimal>
              <Button icon="edit" small text={t('common.edit')} style={{ color: T.textMuted }} onClick={onEdit} />
              <Button
                icon="swap-horizontal"
                small
                text="Unequip"
                style={{ color: T.textMuted }}
                onClick={() => onUnequip?.(item.slot)}
              />
              <Button icon="trash" small intent="danger" text="Destroy" />
            </ButtonGroup>
          )}
          {!equipped && (
            <ButtonGroup minimal>
              <Button
                icon="trash"
                small
                intent="danger"
                text="Delete"
                onClick={() => onDelete?.((item as FullInventoryItem).index)}
              />
            </ButtonGroup>
          )}
        </div>
      </div>

      <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
        <div style={{ fontWeight: 700, color: T.accent, marginBottom: 6 }}>{t('inventory.itemDetails')}</div>
        <div style={{ display: 'grid', gridTemplateColumns: '120px 1fr', rowGap: 4 }}>
          <span style={{ color: T.textMuted }}>{t('inventory.baseItem')}</span>
          <span style={{ fontWeight: 600 }}>{display(baseItemName)}</span>

          <span style={{ color: T.textMuted }}>{t('inventory.weight')}</span>
          <span style={{ fontWeight: 600 }}>{weight > 0 ? `${weight.toFixed(1)} lbs` : '-'}</span>

          <span style={{ color: T.textMuted }}>{t('inventory.value')}</span>
          <span style={{ fontWeight: 600, color: value > 0 ? T.gold : T.text }}>
            {value > 0 ? `${fmtNum(value)} gp` : '-'}
          </span>

          {baseAc !== null && baseAc !== undefined && (
            <>
              <span style={{ color: T.textMuted }}>{t('inventory.baseArmorClass')}</span>
              <span style={{ fontWeight: 600 }}>{baseAc}</span>
            </>
          )}

          {!equipped && (() => {
            const inv = item as FullInventoryItem;
            return (
              <>
                {inv.stack_size > 1 && (
                  <>
                    <span style={{ color: T.textMuted }}>{t('inventory.stack')}</span>
                    <span style={{ fontWeight: 600 }}>{inv.stack_size}</span>
                  </>
                )}
                {inv.charges !== null && inv.charges !== undefined && (
                  <>
                    <span style={{ color: T.textMuted }}>{t('inventory.charges')}</span>
                    <span style={{ fontWeight: 600 }}>{inv.charges}</span>
                  </>
                )}
                <span style={{ color: T.textMuted }}>{t('inventory.identified')}</span>
                <span style={{ fontWeight: 600, color: inv.identified ? T.text : T.negative }}>
                  {inv.identified ? t('inventory.identified') : t('inventory.unidentified')}
                </span>
                {inv.plot && (
                  <>
                    <span style={{ color: T.textMuted }}>{t('inventory.plotItem')}</span>
                    <span style={{ fontWeight: 600, color: T.gold }}>{t('inventory.plotItem')}</span>
                  </>
                )}
                {inv.cursed && (
                  <>
                    <span style={{ color: T.textMuted }}>{t('inventory.cursedItem')}</span>
                    <span style={{ fontWeight: 600, color: T.negative }}>{t('inventory.cursedItem')}</span>
                  </>
                )}
                {inv.stolen && (
                  <>
                    <span style={{ color: T.textMuted }}>{t('inventory.stolenItem')}</span>
                    <span style={{ fontWeight: 600, color: T.gold }}>{t('inventory.stolenItem')}</span>
                  </>
                )}
              </>
            );
          })()}
        </div>
      </div>

      {description && (
        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontWeight: 700, color: T.accent, marginBottom: 8 }}>{t('inventory.description')}</div>
          <p style={{ margin: 0, lineHeight: 1.6, color: T.textMuted }}>{description}</p>
        </div>
      )}

      {properties && properties.length > 0 && (
        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontWeight: 700, color: T.accent, marginBottom: 8 }}>{t('inventory.properties')}</div>
          <ul style={{ margin: 0, paddingLeft: 20, display: 'flex', flexDirection: 'column', gap: 4 }}>
            {properties.map((prop, i) => (
              <li key={i} style={{ color: T.text }}>{prop.display_string}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
