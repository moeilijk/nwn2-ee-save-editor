import { HTMLTable, NonIdealState } from '@blueprintjs/core';
import { GiSwapBag } from 'react-icons/gi';
import { GameIcon } from '../shared/GameIcon';
import { fmtNum } from '../shared';
import { T } from '../theme';
import type { FullInventoryItem } from '@/lib/bindings';
import { display } from '@/utils/dataHelpers';
import { useTranslations } from '@/hooks/useTranslations';

interface BackpackTableProps {
  items: FullInventoryItem[];
  selectedIndex: number | null;
  onSelectItem: (index: number) => void;
}

export function BackpackTable({ items, selectedIndex, onSelectItem }: BackpackTableProps) {
  const t = useTranslations();
  if (items.length === 0) {
    return (
      <NonIdealState
        icon={<GameIcon icon={GiSwapBag} size={40} />}
        title={t('inventory.backpackEmpty')}
        description={t('inventory.backpackEmptyDescription')}
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
          <th>{t('inventory.item')}</th>
          <th>{t('abilityScores.type')}</th>
          <th style={{ textAlign: 'center' }}>{t('inventory.qty')}</th>
          <th style={{ textAlign: 'right' }}>{t('inventory.weight')}</th>
          <th style={{ textAlign: 'right' }}>{t('inventory.value')}</th>
        </tr>
      </thead>
      <tbody>
        {items.map((item) => {
          const resolveCategory = (c: string) => {
            if (c === 'Armor & Clothing') return t('inventory.categories.armorAndClothing');
            if (c === 'Weapons') return t('inventory.categories.weapons');
            if (c === 'Magic Items') return t('inventory.categories.magicItems');
            if (c === 'Accessories') return t('inventory.categories.accessories');
            if (c === 'Miscellaneous') return t('inventory.categories.miscellaneous');
            return c;
          };

          return (
            <tr
              key={item.index}
              onClick={() => onSelectItem(item.index)}
              style={selectedIndex === item.index ? { background: 'rgba(160, 82, 45, 0.1)' } : undefined}
            >
              <td className={item.is_custom ? 't-semibold' : undefined} style={{
                color: item.is_custom ? T.accent : T.text,
                overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
              }}>
                {display(item.name)}
              </td>
              <td style={{ color: T.textMuted }}>{display(resolveCategory(item.category))}</td>
              <td style={{ textAlign: 'center' }}>{item.stack_size > 1 ? item.stack_size : ''}</td>
            <td style={{ textAlign: 'right', color: T.textMuted }}>
              {item.weight > 0 ? `${item.weight.toFixed(1)} lbs` : '-'}
            </td>
            <td style={{ textAlign: 'right', color: T.textMuted }}>
              {item.value > 0 ? `${fmtNum(item.value)} gp` : '-'}
            </td>
          </tr>
          );
        })}
      </tbody>
    </HTMLTable>
  );
}
