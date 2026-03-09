import { HTMLTable } from '@blueprintjs/core';
import { BACKPACK } from '../dummy-data';
import { fmtNum } from '../shared';

export function BackpackTable() {
  return (
    <HTMLTable compact striped style={{ width: '100%' }}>
      <thead>
        <tr>
          <th>Item</th>
          <th style={{ textAlign: 'center' }}>Qty</th>
          <th style={{ textAlign: 'right' }}>Value</th>
        </tr>
      </thead>
      <tbody>
        {BACKPACK.map((item, i) => (
          <tr key={i}>
            <td>{item.name}</td>
            <td style={{ textAlign: 'center' }}>{item.qty}</td>
            <td style={{ textAlign: 'right' }}>{fmtNum(item.value)}</td>
          </tr>
        ))}
      </tbody>
    </HTMLTable>
  );
}
