import { HTMLTable, NonIdealState } from '@blueprintjs/core';
import { fmtNum } from '../shared';
import { T } from '../theme';
import type { FullInventoryItem } from '@/lib/bindings';
import { display } from '@/utils/dataHelpers';

interface BackpackTableProps {
  items: FullInventoryItem[];
  selectedIndex: number | null;
  onSelectItem: (index: number) => void;
}

export function BackpackTable({ items, selectedIndex, onSelectItem }: BackpackTableProps) {
  if (items.length === 0) {
    return (
      <NonIdealState
        icon="box"
        title="Backpack is empty"
        description="No items in backpack."
      />
    );
  }

  return (
    <HTMLTable compact striped bordered interactive style={{ width: '100%', tableLayout: 'fixed' }}>
      <colgroup>
        <col />
        <col style={{ width: 72 }} />
        <col style={{ width: 44 }} />
        <col style={{ width: 64 }} />
        <col style={{ width: 72 }} />
      </colgroup>
      <thead>
        <tr>
          <th>Item</th>
          <th style={{ textAlign: 'center' }}>Type</th>
          <th style={{ textAlign: 'center' }}>Qty</th>
          <th style={{ textAlign: 'right' }}>Weight</th>
          <th style={{ textAlign: 'right' }}>Value</th>
        </tr>
      </thead>
      <tbody>
        {items.map((item) => (
          <tr
            key={item.index}
            onClick={() => onSelectItem(item.index)}
            style={selectedIndex === item.index ? { background: 'rgba(160, 82, 45, 0.1)' } : undefined}
          >
            <td style={{
              color: item.is_custom ? T.accent : T.text,
              fontWeight: item.is_custom ? 600 : 400,
              overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
            }}>
              {display(item.name)}
            </td>
            <td style={{ textAlign: 'center', color: T.textMuted, fontSize: 12 }}>{display(item.category)}</td>
            <td style={{ textAlign: 'center' }}>{item.stack_size > 1 ? item.stack_size : ''}</td>
            <td style={{ textAlign: 'right', color: T.textMuted, fontSize: 12 }}>
              {item.weight > 0 ? `${item.weight.toFixed(1)}` : '-'}
            </td>
            <td style={{ textAlign: 'right', color: item.value > 0 ? T.gold : T.textMuted, fontWeight: 600 }}>
              {item.value > 0 ? fmtNum(item.value) : '-'}
            </td>
          </tr>
        ))}
      </tbody>
    </HTMLTable>
  );
}
