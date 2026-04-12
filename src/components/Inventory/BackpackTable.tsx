import { HTMLTable, NonIdealState } from '@blueprintjs/core';
import { GiSwapBag } from 'react-icons/gi';
import { GameIcon } from '../shared/GameIcon';
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
        icon={<GameIcon icon={GiSwapBag} size={40} />}
        title="Backpack is empty"
        description="No items in backpack."
      />
    );
  }

  return (
    <HTMLTable compact striped bordered interactive style={{ width: '100%', tableLayout: 'fixed' }}>
      <colgroup>
        <col />
        <col style={{ width: 120 }} />
        <col style={{ width: 56 }} />
        <col style={{ width: 84 }} />
        <col style={{ width: 116 }} />
      </colgroup>
      <thead>
        <tr>
          <th>Item</th>
          <th>Type</th>
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
            <td style={{ color: T.textMuted }}>{display(item.category)}</td>
            <td style={{ textAlign: 'center' }}>{item.stack_size > 1 ? item.stack_size : ''}</td>
            <td style={{ textAlign: 'right', color: T.textMuted }}>
              {item.weight > 0 ? `${item.weight.toFixed(1)} lbs` : '-'}
            </td>
            <td style={{ textAlign: 'right', color: T.textMuted }}>
              {item.value > 0 ? `${fmtNum(item.value)} gp` : '-'}
            </td>
          </tr>
        ))}
      </tbody>
    </HTMLTable>
  );
}
