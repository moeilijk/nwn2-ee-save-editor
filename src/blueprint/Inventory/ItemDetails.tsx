import { Button, ButtonGroup, H4, Icon, ProgressBar } from '@blueprintjs/core';
import { T } from '../theme';
import { INVENTORY, BACKPACK } from '../dummy-data';
import { fmtNum } from '../shared';

type InventoryItem = typeof INVENTORY[number];
type BackpackItem = typeof BACKPACK[number];
export type AnyItem = InventoryItem | BackpackItem;

const RARITY_COLORS: Record<string, string> = {
  common: T.text,
  uncommon: T.positive,
  rare: '#1565c0',
  epic: '#6a1b9a',
};

function isEquippedItem(item: AnyItem): item is InventoryItem {
  return 'slot' in item;
}

interface ItemDetailsProps {
  item: AnyItem | null;
  onEdit?: () => void;
}

export function ItemDetails({ item, onEdit }: ItemDetailsProps) {
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
  const nameColor = RARITY_COLORS[item.rarity] || T.text;

  return (
    <div>
      {/* Header */}
      <div style={{ padding: '14px 16px 12px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
          <div>
            <H4 style={{ margin: 0, color: nameColor }}>{item.name}</H4>
            <span style={{ color: T.textMuted }}>
              {equipped ? `${item.slot} \u00b7 ${item.type}` : item.type}
            </span>
          </div>
          {equipped && (
            <ButtonGroup minimal>
              <Button icon="edit" small text="Edit" style={{ color: T.textMuted }} onClick={onEdit} />
              <Button icon="swap-horizontal" small text="Unequip" style={{ color: T.textMuted }} />
              <Button icon="trash" small intent="danger" text="Destroy" />
            </ButtonGroup>
          )}
        </div>
      </div>

      {/* Details */}
      <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
        <div style={{ fontWeight: 700, color: T.accent, marginBottom: 6 }}>Details</div>
        <div style={{ display: 'grid', gridTemplateColumns: '120px 1fr', rowGap: 4 }}>
          {equipped && <><span style={{ color: T.textMuted }}>Base Item</span><span style={{ fontWeight: 600 }}>{item.baseItem}</span></>}
          <span style={{ color: T.textMuted }}>Type</span><span style={{ fontWeight: 600 }}>{item.type}</span>
          <span style={{ color: T.textMuted }}>Weight</span><span style={{ fontWeight: 600 }}>{item.weight > 0 ? `${item.weight.toFixed(1)} lbs` : '-'}</span>
          <span style={{ color: T.textMuted }}>Value</span><span style={{ fontWeight: 600, color: item.value > 0 ? T.gold : T.text }}>{item.value > 0 ? `${fmtNum(item.value)} gp` : '-'}</span>
          {equipped && item.charges !== null && (
            <><span style={{ color: T.textMuted }}>Charges</span>
            <div>
              <span style={{ fontWeight: 600 }}>{item.charges.current} / {item.charges.max}</span>
              <ProgressBar
                value={item.charges.current / item.charges.max}
                intent={item.charges.current / item.charges.max > 0.5 ? 'primary' : item.charges.current / item.charges.max > 0.2 ? 'warning' : 'danger'}
                stripes={false} animate={false} style={{ height: 3, marginTop: 3 }}
              />
            </div></>
          )}
        </div>
      </div>

      {/* Description */}
      {equipped && item.description && (
        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontWeight: 700, color: T.accent, marginBottom: 8 }}>Description</div>
          <p style={{ margin: 0, lineHeight: 1.6, color: T.textMuted }}>
            {item.description}
          </p>
        </div>
      )}

      {/* Properties */}
      {equipped && item.properties.length > 0 && (
        <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
          <div style={{ fontWeight: 700, color: T.accent, marginBottom: 8 }}>Properties</div>
          <ul style={{ margin: 0, paddingLeft: 20, display: 'flex', flexDirection: 'column', gap: 4 }}>
            {item.properties.map((prop, i) => (
              <li key={i} style={{ color: T.text }}>{prop}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
