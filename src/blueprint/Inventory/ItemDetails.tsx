import { Button, ButtonGroup, H4, Icon } from '@blueprintjs/core';
import { T } from '../theme';
import { INVENTORY } from '../dummy-data';
import { SectionBar, KVRow, fmtNum } from '../shared';

type InventoryItem = typeof INVENTORY[number];

interface ItemDetailsProps {
  item: InventoryItem | null;
}

export function ItemDetails({ item }: ItemDetailsProps) {
  if (!item) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
        <div style={{ textAlign: 'center' }}>
          <Icon icon="select" size={32} style={{ color: T.border }} />
          <p style={{ marginTop: 8, color: T.textMuted }}>Select an item</p>
        </div>
      </div>
    );
  }

  return (
    <div>
      <div style={{ padding: '16px 24px', borderBottom: `1px solid ${T.borderLight}` }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
          <div>
            <H4 style={{ margin: 0, color: T.text }}>{item.name}</H4>
            <span style={{ fontSize: 12, color: T.textMuted }}>{item.slot} &middot; {item.type}</span>
          </div>
          <ButtonGroup minimal>
            <Button icon="edit" small text="Edit" style={{ color: T.textMuted }} />
            <Button icon="swap-horizontal" small text="Unequip" style={{ color: T.textMuted }} />
            <Button icon="trash" small intent="danger" text="Destroy" />
          </ButtonGroup>
        </div>
      </div>

      <SectionBar title="Details" />
      <div style={{ padding: '12px 24px' }}>
        <KVRow label="Weight" value={`${item.weight.toFixed(1)} lbs`} />
        <KVRow label="Value" value={`${fmtNum(item.value)} gp`} color={T.gold} />
      </div>

      <SectionBar title="Properties" />
      <div style={{ padding: '12px 24px', display: 'flex', flexDirection: 'column', gap: 6 }}>
        {item.properties.map((prop, i) => (
          <div key={i} style={{
            display: 'flex', alignItems: 'center', gap: 8,
            padding: '6px 12px',
            background: T.sectionBg,
            border: `1px solid ${T.sectionBorder}`,
            borderRadius: 4,
            fontSize: 13,
            color: T.text,
          }}>
            <Icon icon="small-tick" size={12} style={{ color: T.positive }} />
            {prop}
          </div>
        ))}
      </div>
    </div>
  );
}
