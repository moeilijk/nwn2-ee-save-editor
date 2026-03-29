import { useState, useMemo } from 'react';
import { Button, Card, Elevation, HTMLTable, InputGroup, Tag } from '@blueprintjs/core';
import { T } from '../theme';
import { SKILLS } from '../dummy-data';
import { ModCell, mod, StepInput } from '../shared';

type SortCol = 'name' | 'total' | 'ranks' | null;
type SortDir = 'asc' | 'desc';

const BUDGET = { spent: 156, available: 12, overdrawn: 0 };

function SectionLabel({ children }: { children: string }) {
  return (
    <div style={{ fontSize: 12, fontWeight: 700, color: T.accent, marginBottom: 8 }}>
      {children}
    </div>
  );
}

function SortIcon({ active, dir }: { active: boolean; dir: SortDir }) {
  if (!active) return null;
  return (
    <span style={{ marginLeft: 4, fontSize: 10 }}>
      {dir === 'asc' ? '\u25B2' : '\u25BC'}
    </span>
  );
}

export function SkillsPanel() {
  const [filter, setFilter] = useState('');
  const [ranks, setRanks] = useState(() => SKILLS.map(s => s.ranks));
  const [sortCol, setSortCol] = useState<SortCol>('name');
  const [sortDir, setSortDir] = useState<SortDir>('asc');

  const handleSort = (col: SortCol) => {
    if (sortCol === col) {
      setSortDir(d => d === 'asc' ? 'desc' : 'asc');
    } else {
      setSortCol(col);
      setSortDir('asc');
    }
  };

  const skills = useMemo(() => {
    const withRanks = SKILLS.map((s, i) => {
      const r = ranks[i];
      const rankDiff = r - s.ranks;
      return { ...s, ranks: r, total: s.total + rankDiff, _idx: i };
    });

    const filtered = withRanks.filter(s =>
      !filter || s.name.toLowerCase().includes(filter.toLowerCase())
    );

    if (!sortCol) return filtered;

    return [...filtered].sort((a, b) => {
      let cmp = 0;
      if (sortCol === 'name') cmp = a.name.localeCompare(b.name);
      else if (sortCol === 'total') cmp = a.total - b.total;
      else if (sortCol === 'ranks') cmp = a.ranks - b.ranks;
      return sortDir === 'asc' ? cmp : -cmp;
    });
  }, [filter, ranks, sortCol, sortDir]);

  const handleRankChange = (idx: number, value: number) => {
    setRanks(prev => prev.map((r, i) => i === idx ? value : r));
  };

  const thSortable = (label: string, col: SortCol) => ({
    style: { textAlign: 'center' as const, cursor: 'pointer', userSelect: 'none' as const },
    onClick: () => handleSort(col),
  });

  return (
    <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 12 }}>
      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>

        <div style={{ display: 'flex', alignItems: 'center', gap: 16, padding: '10px 16px', borderBottom: `1px solid ${T.borderLight}` }}>
          <span style={{ color: T.textMuted }}>Spent: <strong style={{ color: T.text }}>{BUDGET.spent}</strong></span>
          <span style={{ color: T.textMuted }}>Available: <strong style={{ color: T.accent }}>{BUDGET.available}</strong></span>
          <span style={{ color: BUDGET.overdrawn > 0 ? T.negative : T.textMuted }}>Overdrawn: <strong>{BUDGET.overdrawn}</strong></span>
          <div style={{ flex: 1 }} />
          <InputGroup
            leftIcon="search" placeholder="Filter skills..." value={filter}
            onChange={e => setFilter(e.target.value)}
            rightElement={filter ? <Button icon="cross" minimal small onClick={() => setFilter('')} /> : undefined}
            style={{ maxWidth: 240 }}
          />
          <Button icon="reset" text="Reset" minimal style={{ color: T.textMuted }} />
        </div>

        <div style={{ padding: '12px 16px 16px' }}>
          <SectionLabel>Skills</SectionLabel>
          <HTMLTable compact striped bordered interactive style={{ width: '100%', tableLayout: 'fixed' }}>
            <colgroup>
              <col />
              <col style={{ width: 200 }} />
              <col style={{ width: 72 }} />
              <col style={{ width: 72 }} />
              <col style={{ width: 72 }} />
              <col style={{ width: 80 }} />
            </colgroup>
            <thead>
              <tr>
                <th {...thSortable('Skill', 'name')} style={{ textAlign: 'left', cursor: 'pointer', userSelect: 'none' }}>
                  Skill<SortIcon active={sortCol === 'name'} dir={sortDir} />
                </th>
                <th {...thSortable('Ranks', 'ranks')}>
                  Ranks<SortIcon active={sortCol === 'ranks'} dir={sortDir} />
                </th>
                <th style={{ textAlign: 'center' }}>Ability</th>
                <th style={{ textAlign: 'center' }}>Misc</th>
                <th {...thSortable('Total', 'total')}>
                  Total<SortIcon active={sortCol === 'total'} dir={sortDir} />
                </th>
                <th style={{ textAlign: 'center' }}>Class</th>
              </tr>
            </thead>
            <tbody>
              {skills.map(s => (
                <tr key={s.name}>
                  <td>
                    <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}>
                      <strong style={{ color: T.text, display: 'inline-block', width: 120 }}>{s.name}</strong>
                      <span style={{ color: T.textMuted, display: 'inline-block', width: 28 }}>{s.ability}</span>
                      {s.acp && (
                        <Tag minimal style={{ fontSize: 10, padding: '0 4px', lineHeight: '16px', background: T.sectionBg, color: T.accent }}>ACP</Tag>
                      )}
                      {!s.isClassSkill && (
                        <Tag minimal style={{ fontSize: 10, padding: '0 4px', lineHeight: '16px', background: T.sectionBg, color: T.textMuted }}>2pt</Tag>
                      )}
                    </span>
                  </td>
                  <td style={{ textAlign: 'center' }}>
                    <StepInput
                      value={s.ranks}
                      onValueChange={(v) => handleRankChange(s._idx, v)}
                      min={0} max={99} width={88}
                    />
                  </td>
                  <td style={{ textAlign: 'center' }}><ModCell value={s.abilityMod} /></td>
                  <td style={{ textAlign: 'center', color: T.textMuted }}>{s.misc || '-'}</td>
                  <td style={{ textAlign: 'center', fontWeight: 500 }}>{mod(s.total)}</td>
                  <td style={{ textAlign: 'center' }}>
                    {s.isClassSkill
                      ? <span style={{ color: T.positive, fontWeight: 500 }}>Class</span>
                      : <span style={{ color: T.textMuted }}>Cross</span>}
                  </td>
                </tr>
              ))}
            </tbody>
          </HTMLTable>
          <div style={{ marginTop: 8, display: 'flex', gap: 16, color: T.textMuted, fontSize: 12 }}>
            <span><Tag minimal style={{ fontSize: 10, padding: '0 4px', lineHeight: '16px', background: T.sectionBg, color: T.accent }}>ACP</Tag> Armor Check Penalty applies</span>
            <span><Tag minimal style={{ fontSize: 10, padding: '0 4px', lineHeight: '16px', background: T.sectionBg, color: T.textMuted }}>2pt</Tag> Cross-class skill (costs 2 points per rank)</span>
          </div>
        </div>
      </Card>
    </div>
  );
}
