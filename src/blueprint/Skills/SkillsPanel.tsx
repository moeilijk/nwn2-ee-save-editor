import { useState, useEffect, useMemo } from 'react';
import { Button, Card, Elevation, HTMLTable, InputGroup, NonIdealState, Spinner, Tag } from '@blueprintjs/core';
import { T } from '../theme';
import { ModCell, mod, StepInput } from '../shared';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { useSkillManagement } from '@/hooks/useSkillManagement';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import { applySkillOverrides, categorizeSkills, calculateSkillBudget, filterAndSortSkills } from '@/utils/skillUtils';
import { useTranslations } from '@/hooks/useTranslations';

type SortCol = 'name' | 'total' | 'ranks' | null;
type SortDir = 'asc' | 'desc';

function SortIcon({ active, dir }: { active: boolean; dir: SortDir }) {
  if (!active) return null;
  return (
    <span style={{ marginLeft: 4, fontSize: 10 }}>
      {dir === 'asc' ? '\u25B2' : '\u25BC'}
    </span>
  );
}

export function SkillsPanel() {
  const t = useTranslations();
  const { character } = useCharacterContext();
  const skillsSubsystem = useSubsystem('skills');
  const { updateSkills, resetSkills } = useSkillManagement();
  const { handleError } = useErrorHandler();

  const [filter, setFilter] = useState('');
  const [sortCol, setSortCol] = useState<SortCol>('name');
  const [sortDir, setSortDir] = useState<SortDir>('asc');
  const [localOverrides, setLocalOverrides] = useState<Record<number, number>>({});
  const [updatingSkills, setUpdatingSkills] = useState<Set<number>>(new Set());
  const [isResetting, setIsResetting] = useState(false);
  const [fixedTotalBudget, setFixedTotalBudget] = useState<number | null>(null);

  const { data: skillsData, isLoading, error } = skillsSubsystem;

  useEffect(() => {
    if (character?.id && !skillsData && !isLoading) {
      skillsSubsystem.load();
    }
  }, [character?.id, skillsData, isLoading, skillsSubsystem]);

  useEffect(() => {
    setLocalOverrides({});
  }, [skillsData]);

  useEffect(() => {
    setFixedTotalBudget(null);
  }, [character?.id]);

  useEffect(() => {
    if (skillsData && fixedTotalBudget === null) {
      setFixedTotalBudget(skillsData.total_available);
    }
  }, [skillsData, fixedTotalBudget]);

  const { allSkills, budget } = useMemo(() => {
    const skills = applySkillOverrides(
      categorizeSkills(skillsData?.class_skills, skillsData?.cross_class_skills),
      localOverrides
    );
    const totalAvailable = fixedTotalBudget ?? skillsData?.total_available ?? 0;
    return {
      allSkills: skills,
      budget: calculateSkillBudget(totalAvailable, skillsData?.spent_points || 0),
    };
  }, [skillsData, localOverrides, fixedTotalBudget]);

  const sorted = useMemo(
    () => filterAndSortSkills(allSkills, filter, sortCol, sortDir),
    [allSkills, filter, sortCol, sortDir]
  );

  const handleSort = (col: SortCol) => {
    if (sortCol === col) {
      setSortDir(d => d === 'asc' ? 'desc' : 'asc');
    } else {
      setSortCol(col);
      setSortDir('asc');
    }
  };

  const handleRankChange = async (skillId: number, newRank: number) => {
    if (!character?.id || newRank < 0) return;

    setLocalOverrides(prev => ({ ...prev, [skillId]: newRank }));
    setUpdatingSkills(prev => new Set([...prev, skillId]));

    try {
      await updateSkills({ [skillId]: newRank });
    } catch (err) {
      handleError(err);
      setLocalOverrides(prev => {
        const updated = { ...prev };
        delete updated[skillId];
        return updated;
      });
    } finally {
      setUpdatingSkills(prev => {
        const next = new Set(prev);
        next.delete(skillId);
        return next;
      });
    }
  };

  const handleReset = async () => {
    if (!character?.id) return;
    setIsResetting(true);
    try {
      await resetSkills();
      setFixedTotalBudget(null);
    } catch (err) {
      handleError(err);
    } finally {
      setIsResetting(false);
    }
  };

  const thSortable = (col: SortCol) => ({
    style: { textAlign: 'center' as const, cursor: 'pointer', userSelect: 'none' as const },
    onClick: () => handleSort(col),
  });

  if (isLoading && !skillsData) {
    return (
      <div style={{ padding: 16, display: 'flex', justifyContent: 'center', paddingTop: 64 }}>
        <Spinner size={30} />
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ padding: 16 }}>
        <NonIdealState icon="error" title="Failed to load skills" description={error} />
      </div>
    );
  }

  if (!character || !skillsData) {
    return (
      <div style={{ padding: 16 }}>
        <NonIdealState icon="person" title="No character loaded" description="Import a save file to begin editing." />
      </div>
    );
  }

  return (
    <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 12 }}>
      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>

        <div style={{ display: 'flex', alignItems: 'center', gap: 16, padding: '10px 16px', borderBottom: `1px solid ${T.borderLight}` }}>
          <span style={{ color: T.textMuted }}>Spent: <strong style={{ color: T.text }}>{budget.displayedSpent}</strong></span>
          <span style={{ color: T.textMuted }}>Available: <strong style={{ color: T.accent }}>{budget.available}</strong></span>
          <span style={{ color: budget.overdrawn > 0 ? T.negative : T.textMuted }}>Overdrawn: <strong>{budget.overdrawn}</strong></span>
          <div style={{ flex: 1 }} />
          <InputGroup
            leftIcon="search" placeholder={t('skills.searchSkills')} value={filter}
            onChange={e => setFilter(e.target.value)}
            rightElement={filter ? <Button icon="cross" minimal small onClick={() => setFilter('')} /> : undefined}
            style={{ maxWidth: 240 }}
          />
          <Button icon="reset" text={t('skills.reset')} minimal style={{ color: T.textMuted }}
            onClick={handleReset} disabled={isResetting} />
        </div>

        <div style={{ padding: '12px 16px 16px' }}>
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
                <th {...thSortable('name')} style={{ textAlign: 'left', cursor: 'pointer', userSelect: 'none' }}>
                  {t('skills.skillName')}<SortIcon active={sortCol === 'name'} dir={sortDir} />
                </th>
                <th {...thSortable('ranks')}>
                  {t('skills.ranks')}<SortIcon active={sortCol === 'ranks'} dir={sortDir} />
                </th>
                <th style={{ textAlign: 'center' }}>{t('skills.ability')}</th>
                <th style={{ textAlign: 'center' }}>{t('skills.misc')}</th>
                <th {...thSortable('total')}>
                  {t('skills.total')}<SortIcon active={sortCol === 'total'} dir={sortDir} />
                </th>
                <th style={{ textAlign: 'center' }}>{t('skills.class')}</th>
              </tr>
            </thead>
            <tbody>
              {sorted.map(s => {
                const misc = (s.feat_bonus || 0) + (s.item_bonus || 0);
                return (
                  <tr key={s.skill_id}>
                    <td>
                      <span style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}>
                        <strong style={{ color: T.text, display: 'inline-block', width: 120 }}>{s.name}</strong>
                        <span style={{ color: T.textMuted, display: 'inline-block', width: 28 }}>{s.ability}</span>
                        {s.armor_check_penalty && (
                          <Tag minimal style={{ fontSize: 10, padding: '0 4px', lineHeight: '16px', background: T.sectionBg, color: T.accent }}>ACP</Tag>
                        )}
                        {!s.is_class_skill && (
                          <Tag minimal style={{ fontSize: 10, padding: '0 4px', lineHeight: '16px', background: T.sectionBg, color: T.textMuted }}>2pt</Tag>
                        )}
                      </span>
                    </td>
                    <td style={{ textAlign: 'center' }}>
                      <StepInput
                        value={s.ranks}
                        onValueChange={(v) => handleRankChange(s.skill_id, v)}
                        min={0} max={99} width={88}
                        disabled={updatingSkills.has(s.skill_id)}
                      />
                    </td>
                    <td style={{ textAlign: 'center' }}><ModCell value={s.modifier} /></td>
                    <td style={{ textAlign: 'center', color: T.textMuted }}>{misc ? mod(misc) : '-'}</td>
                    <td style={{ textAlign: 'center', fontWeight: 500 }}>{mod(s.total)}</td>
                    <td style={{ textAlign: 'center' }}>
                      {s.is_class_skill
                        ? <span style={{ color: T.positive, fontWeight: 500 }}>Class</span>
                        : <span style={{ color: T.textMuted }}>Cross</span>}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </HTMLTable>
          <div style={{ marginTop: 8, display: 'flex', gap: 16, color: T.textMuted, fontSize: 12 }}>
            <span><Tag minimal style={{ fontSize: 10, padding: '0 4px', lineHeight: '16px', background: T.sectionBg, color: T.accent }}>ACP</Tag> {t('skills.armorCheck')}</span>
            <span><Tag minimal style={{ fontSize: 10, padding: '0 4px', lineHeight: '16px', background: T.sectionBg, color: T.textMuted }}>2pt</Tag> Cross-class skill (costs 2 points per rank)</span>
          </div>
        </div>
      </Card>
    </div>
  );
}
