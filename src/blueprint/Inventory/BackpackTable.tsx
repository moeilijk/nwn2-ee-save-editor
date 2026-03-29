import { HTMLTable } from '@blueprintjs/core';
import { BACKPACK } from '../dummy-data';
import { fmtNum } from '../shared';
import { T, RARITY_COLORS } from '../theme';

interface BackpackTableProps {
  selectedIndex: number | null;
  onSelectItem: (index: number) => void;
}

export function BackpackTable({ selectedIndex, onSelectItem }: BackpackTableProps) {
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
        {BACKPACK.map((item, i) => (
          <tr
            key={i}
            onClick={() => onSelectItem(i)}
            style={selectedIndex === i ? { background: 'rgba(160, 82, 45, 0.1)' } : undefined}
          >
            <td style={{ color: RARITY_COLORS[item.rarity], fontWeight: item.rarity !== 'common' ? 600 : 400, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{item.name}</td>
            <td style={{ textAlign: 'center', color: T.textMuted, fontSize: 12 }}>{item.type}</td>
            <td style={{ textAlign: 'center' }}>{item.qty}</td>
            <td style={{ textAlign: 'right', color: T.textMuted, fontSize: 12 }}>{item.weight > 0 ? `${item.weight.toFixed(1)}` : '-'}</td>
            <td style={{ textAlign: 'right', color: item.value > 0 ? T.gold : T.textMuted, fontWeight: 600 }}>{item.value > 0 ? fmtNum(item.value) : '-'}</td>
          </tr>
        ))}
      </tbody>
    </HTMLTable>
  );
}
