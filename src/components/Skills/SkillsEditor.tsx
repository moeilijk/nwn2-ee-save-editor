
import { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { useTranslations } from '@/hooks/useTranslations';
import { Card, CardContent } from '@/components/ui/Card';
import { Input } from '@/components/ui/Input';
import { Button } from '@/components/ui/Button';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { display, formatModifier } from '@/utils/dataHelpers';
import { CharacterAPI } from '@/services/characterApi';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import type { SkillSummaryEntry } from '@/lib/bindings';

export default function SkillsEditor() {
  const t = useTranslations();
  const { character } = useCharacterContext();
  
  const skillsSubsystem = useSubsystem('skills');
  const { handleError } = useErrorHandler();

  const [isUpdating, setIsUpdating] = useState(false);
  const [updatingSkills, setUpdatingSkills] = useState<Set<number>>(new Set());
  const [localSkillOverrides, setLocalSkillOverrides] = useState<Record<number, number>>({})
  

  const [hoveredSkillId, setHoveredSkillId] = useState<number | null>(null);
  const [clickedButton, setClickedButton] = useState<string | null>(null);
  const [showFixedHeader, setShowFixedHeader] = useState(false);
  const [columnWidths, setColumnWidths] = useState<number[]>([]);
  const [tableWidth, setTableWidth] = useState<number>(0);
  const [tableLeft, setTableLeft] = useState<number>(0);
  const tableRef = useRef<HTMLTableElement>(null);
  const headerRef = useRef<HTMLTableRowElement>(null);
  const cardRef = useRef<HTMLDivElement>(null);

  const [searchTerm, setSearchTerm] = useState('');
  const [sortColumn, setSortColumn] = useState<'name' | 'total' | 'ranks' | null>('name');
  const [sortDirection, setSortDirection] = useState<'asc' | 'desc'>('asc');

  const { data: skillsData, isLoading, error, load: loadSkills } = skillsSubsystem;

  useEffect(() => {
    if (!character?.id) return;
    if (!skillsData && !isLoading) {
      loadSkills().catch(err => console.error('Failed to load skills:', err));
    }
  }, [character?.id, skillsData, isLoading, loadSkills]);

  useEffect(() => {
    setLocalSkillOverrides({});
  }, [skillsData]);
  const applyOverrides = (skillList: SkillSummaryEntry[]) => {
    return skillList.map(skill => {
      const overrideRanks = localSkillOverrides[skill.skill_id];
      if (overrideRanks !== undefined) {
        const rankDiff = overrideRanks - skill.ranks;
        return {
          ...skill,
          ranks: overrideRanks,
          total: skill.total + rankDiff
        };
      }
      return skill;
    });
  };

  const isValidSkillName = (name: string) => !name.startsWith('DEL_') && !name.startsWith('***');
  const classSkills = applyOverrides(skillsData?.class_skills?.filter(skill => isValidSkillName(skill.name)) || []);
  const crossClassSkills = applyOverrides(skillsData?.cross_class_skills?.filter(skill => isValidSkillName(skill.name)) || []);
  const skills = [...classSkills, ...crossClassSkills];

  const totalAvailable = skillsData?.total_available ?? 0;
  const totalSpentPoints = skillsData?.spent_points || 0;
  const pointsBalance = totalAvailable - totalSpentPoints;
  const availableSkillPoints = Math.max(0, pointsBalance);
  const overdrawnSkillPoints = pointsBalance < 0 ? Math.abs(pointsBalance) : 0;
  const _totalSkillPoints = totalAvailable;

  useEffect(() => {
    const handleScroll = () => {
      if (headerRef.current) {
        const rect = headerRef.current.getBoundingClientRect();
        setShowFixedHeader(rect.bottom < 87);
      }
    };

    const measureColumnWidths = () => {
      if (headerRef.current && cardRef.current) {
        const ths = headerRef.current.querySelectorAll('th');
        const widths = Array.from(ths).map(th => th.getBoundingClientRect().width);
        setColumnWidths(widths);

        const cardRect = cardRef.current.getBoundingClientRect();
        setTableWidth(cardRect.width);
        setTableLeft(cardRect.left);
      }
    };

    const scrollContainer = document.querySelector('main');

    if (scrollContainer) {
      scrollContainer.addEventListener('scroll', handleScroll);
    }
    window.addEventListener('resize', measureColumnWidths);

    setTimeout(() => {
      handleScroll();
      measureColumnWidths();
    }, 100);

    return () => {
      if (scrollContainer) {
        scrollContainer.removeEventListener('scroll', handleScroll);
      }
      window.removeEventListener('resize', measureColumnWidths);
    };
  }, []);

  const handleUpdateSkillRank = async (skillId: number, newRank: number) => {
    if (!character?.id) return;

    const skill = skills.find(s => s.skill_id === skillId);
    if (!skill) return;

    if (newRank < 0) return;

    setLocalSkillOverrides(prev => ({
      ...prev,
      [skillId]: newRank
    }));

    setUpdatingSkills(prev => new Set([...prev, skillId]));

    try {
      const updates = { [skillId]: newRank };
      await CharacterAPI.updateSkills(character.id, updates);
      await loadSkills({ silent: true });
    } catch (err) {
      handleError(err);

      setLocalSkillOverrides(prev => {
        const updated = { ...prev };
        delete updated[skillId];
        return updated;
      });
    } finally {
      setUpdatingSkills(prev => {
        const newSet = new Set(prev);
        newSet.delete(skillId);
        return newSet;
      });
    }
  };

  const handleButtonClick = (buttonType: 'increase' | 'decrease', skillId: number) => {
    const buttonKey = `${buttonType}-${skillId}`;
    setClickedButton(buttonKey);
    setTimeout(() => setClickedButton(null), 150);

    const skill = skills.find(s => s.skill_id === skillId);
    if (!skill) return;

    if (buttonType === 'increase') {
      handleUpdateSkillRank(skillId, skill.ranks + 1);
    } else {
      handleUpdateSkillRank(skillId, skill.ranks - 1);
    }
  };

  const resetAllSkills = async () => {
    if (!character?.id) return;

    setIsUpdating(true);

    try {
      await CharacterAPI.resetSkills(character.id);
      await loadSkills({ silent: true });
    } catch (err) {
      handleError(err);
    } finally {
      setIsUpdating(false);
    }
  };

  const handleSort = (column: 'name' | 'total' | 'ranks') => {
    if (sortColumn === column) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortColumn(column);
      setSortDirection('asc');
    }
  };

  const sortedAndFilteredSkills = [...skills]
    .filter(skill => 
      skill.name.toLowerCase().includes(searchTerm.toLowerCase())
    )
    .sort((a, b) => {
      if (!sortColumn) return 0;
      
      let compareValue = 0;
      switch (sortColumn) {
        case 'name':
          compareValue = a.name.localeCompare(b.name);
          break;
        case 'total':
          compareValue = a.total - b.total;
          break;
        case 'ranks':
          compareValue = a.ranks - b.ranks;
          break;
      }
      
      return sortDirection === 'asc' ? compareValue : -compareValue;
    });

  useEffect(() => {
    const measureColumnWidths = () => {
      if (headerRef.current && cardRef.current) {
        const ths = headerRef.current.querySelectorAll('th');
        const widths = Array.from(ths).map(th => th.getBoundingClientRect().width);
        setColumnWidths(widths);

        const cardRect = cardRef.current.getBoundingClientRect();
        setTableWidth(cardRect.width);
        setTableLeft(cardRect.left);
      }
    };

    setTimeout(measureColumnWidths, 0);
  }, [sortedAndFilteredSkills]);

  const FixedHeader = () => {
    if (!showFixedHeader || typeof document === 'undefined') return null;
    
    return createPortal(
      <div 
        className="fixed top-[87px] z-50"
        style={{ 
          left: `${tableLeft}px`, 
          width: `${tableWidth}px` 
        }}
      >
        <Card className="fixed-table-header rounded-t-none shadow-lg border-b-0">
          <CardContent className="p-0" style={{ paddingTop: '0', paddingBottom: '0' }}>
            <div className="overflow-x-auto">
              <table className="w-full" style={{ tableLayout: 'fixed' }}>
                <colgroup>
                  <col style={{ width: `${columnWidths[0]}px` }} />
                  <col style={{ width: `${columnWidths[1]}px` }} />
                  <col style={{ width: `${columnWidths[2]}px` }} />
                  <col style={{ width: `${columnWidths[3]}px` }} />
                  <col style={{ width: `${columnWidths[4]}px` }} />
                  <col style={{ width: `${columnWidths[5]}px` }} />
                </colgroup>
                <thead>
                  <tr className="border-b border-[rgb(var(--color-surface-border)/0.6)]">
                    <th 
                      className="text-left p-3 font-medium text-[rgb(var(--color-text-secondary))] cursor-pointer hover:text-[rgb(var(--color-text-primary))]"
                      onClick={() => handleSort('name')}
                    >
                      <div className="flex items-center space-x-1">
                        <span>{t('skills.skillName')}</span>
                        {sortColumn === 'name' && (
                          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={sortDirection === 'asc' ? "M5 15l7-7 7 7" : "M19 9l-7 7-7-7"} />
                          </svg>
                        )}
                      </div>
                    </th>
                    <th 
                      className="text-center p-3 font-medium text-[rgb(var(--color-text-secondary))] cursor-pointer hover:text-[rgb(var(--color-text-primary))]"
                      onClick={() => handleSort('total')}
                    >
                      <div className="flex items-center justify-center space-x-1">
                        <span>{t('skills.total')}</span>
                        {sortColumn === 'total' && (
                          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={sortDirection === 'asc' ? "M5 15l7-7 7 7" : "M19 9l-7 7-7-7"} />
                          </svg>
                        )}
                      </div>
                    </th>
                    <th 
                      className="text-center p-3 font-medium text-[rgb(var(--color-text-secondary))] cursor-pointer hover:text-[rgb(var(--color-text-primary))]"
                      onClick={() => handleSort('ranks')}
                    >
                      <div className="flex items-center justify-center space-x-1">
                        <span>{t('skills.ranks')}</span>
                        {sortColumn === 'ranks' && (
                          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={sortDirection === 'asc' ? "M5 15l7-7 7 7" : "M19 9l-7 7-7-7"} />
                          </svg>
                        )}
                      </div>
                    </th>
                    <th className="text-center p-3 font-medium text-[rgb(var(--color-text-secondary))]">{t('skills.ability')}</th>
                    <th className="text-center p-3 font-medium text-[rgb(var(--color-text-secondary))]">{t('skills.misc')}</th>
                    <th className="text-center p-3 font-medium text-[rgb(var(--color-text-secondary))]">{t('skills.class')}</th>
                  </tr>
                </thead>
              </table>
            </div>
          </CardContent>
        </Card>
      </div>,
      document.body
    );
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-[rgb(var(--color-primary))]"></div>
      </div>
    );
  }

  if (error) {
    return (
      <Card variant="error">
        <p className="text-error">{error}</p>
      </Card>
    );
  }

  if (!character) {
    return (
      <Card variant="warning">
        <p className="text-muted">No character loaded. Please import a save file to begin.</p>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-3 gap-3">
        <Card>
          <CardContent padding="p-3" className="text-center">
            <div className="text-xs text-[rgb(var(--color-text-muted))]">{t('skills.pointsSpent')}</div>
            <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">
              {display(totalSpentPoints)}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent padding="p-3" className="text-center">
            <div className="text-xs text-[rgb(var(--color-text-muted))]">{t('skills.pointsAvailable')}</div>
            <div className="text-xl font-bold text-[rgb(var(--color-primary))]">
              {display(availableSkillPoints)}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent padding="p-3" className="text-center">
            <div className="text-xs text-[rgb(var(--color-text-muted))]">Points Overdrawn</div>
            <div className={`text-xl font-bold ${overdrawnSkillPoints > 0 ? 'text-error' : 'text-[rgb(var(--color-text-muted))]'}`}>
              {display(overdrawnSkillPoints)}
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <Input
            type="text"
            placeholder={t('skills.searchSkills')}
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-64"
          />
        </div>
        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            size="sm"
            onClick={resetAllSkills}
            disabled={isUpdating}
          >
            {t('skills.reset')}
          </Button>
        </div>
      </div>

      <FixedHeader />

      <Card ref={cardRef} className="mt-5">
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table ref={tableRef} className="w-full" style={{ tableLayout: 'fixed' }}>
              <colgroup>
                <col style={{ width: '40%' }} />
                <col style={{ width: '10%' }} />
                <col style={{ width: '15%' }} />
                <col style={{ width: '10%' }} />
                <col style={{ width: '10%' }} />
                <col style={{ width: '15%' }} />
              </colgroup>
              <thead>
                <tr ref={headerRef} className="border-b border-[rgb(var(--color-surface-border)/0.6)]">
                  <th 
                    className="text-left p-3 font-medium text-[rgb(var(--color-text-secondary))] cursor-pointer hover:text-[rgb(var(--color-text-primary))]"
                    onClick={() => handleSort('name')}
                  >
                    <div className="flex items-center space-x-1">
                      <span>Skill Name</span>
                      {sortColumn === 'name' && (
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={sortDirection === 'asc' ? "M5 15l7-7 7 7" : "M19 9l-7 7-7-7"} />
                        </svg>
                      )}
                    </div>
                  </th>
                  <th 
                    className="text-center p-3 font-medium text-[rgb(var(--color-text-secondary))] cursor-pointer hover:text-[rgb(var(--color-text-primary))]"
                    onClick={() => handleSort('total')}
                  >
                    <div className="flex items-center justify-center space-x-1">
                      <span>Total</span>
                      {sortColumn === 'total' && (
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={sortDirection === 'asc' ? "M5 15l7-7 7 7" : "M19 9l-7 7-7-7"} />
                        </svg>
                      )}
                    </div>
                  </th>
                  <th 
                    className="text-center p-3 font-medium text-[rgb(var(--color-text-secondary))] cursor-pointer hover:text-[rgb(var(--color-text-primary))]"
                    onClick={() => handleSort('ranks')}
                  >
                    <div className="flex items-center justify-center space-x-1">
                      <span>Ranks</span>
                      {sortColumn === 'ranks' && (
                        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={sortDirection === 'asc' ? "M5 15l7-7 7 7" : "M19 9l-7 7-7-7"} />
                        </svg>
                      )}
                    </div>
                  </th>
                  <th className="text-center p-3 font-medium text-[rgb(var(--color-text-secondary))]">Ability</th>
                  <th className="text-center p-3 font-medium text-[rgb(var(--color-text-secondary))]">Misc</th>
                  <th className="text-center p-3 font-medium text-[rgb(var(--color-text-secondary))]">Class</th>
                </tr>
              </thead>
              <tbody>
                {sortedAndFilteredSkills.map((skill) => {
                  return (
                  <tr
                    key={skill.skill_id}
                    className="border-b border-[rgb(var(--color-surface-border)/0.3)] hover:bg-[rgb(var(--color-surface-1))] transition-colors"
                    onMouseEnter={() => setHoveredSkillId(skill.skill_id)}
                    onMouseLeave={() => setHoveredSkillId(null)}
                  >
                    <td className="p-3">
                      <div className="flex items-center space-x-2">
                        <span className="font-medium text-[rgb(var(--color-text-primary))]">{display(skill.name)}</span>
                        <span className="text-sm text-[rgb(var(--color-text-muted))]">({display(skill.ability)})</span>
                        {!skill.is_class_skill && (
                          <span className="text-xs px-1.5 py-0.5 rounded bg-[rgb(var(--color-surface-3))] text-[rgb(var(--color-text-muted))]">
                            2pt
                          </span>
                        )}
                        {skill.armor_check_penalty && (
                          <span
                            className="text-xs px-1.5 py-0.5 rounded bg-[rgb(var(--color-warning)/0.2)] text-[rgb(var(--color-warning))]"
                            title={t('skills.armorCheck')}
                          >
                            ACP
                          </span>
                        )}
                      </div>
                    </td>
                    <td className="p-3 text-center">
                      <span className="text-lg font-semibold text-[rgb(var(--color-primary))]">
                        {formatModifier(skill.total)}
                      </span>
                    </td>
                    <td className="p-3">
                      <div className={`flex items-center justify-center space-x-2 transition-opacity ${hoveredSkillId === skill.skill_id ? 'opacity-100' : 'opacity-60'}`}>
                        <Button
                          onClick={() => handleButtonClick('decrease', skill.skill_id)}
                          variant="outline"
                          size="sm"
                          disabled={(skill.ranks === 0) || updatingSkills.has(skill.skill_id)}
                          clicked={clickedButton === `decrease-${skill.skill_id}`}
                          aria-label={`Decrease ${skill.name}`}
                          title={`Decrease ${skill.name} (min: 0)`}
                          className="h-6 w-6 p-0 text-xs"
                        >
                          -
                        </Button>
                        <input
                          type="number"
                          value={skill.ranks}
                          onChange={(e) => handleUpdateSkillRank(skill.skill_id, parseInt(e.target.value) || 0)}
                          className="w-12 text-center h-6 text-sm border rounded font-medium bg-[rgb(var(--color-surface-2))] border-[rgb(var(--color-surface-border)/0.6)]"
                          disabled={updatingSkills.has(skill.skill_id)}
                        />
                        <Button
                          onClick={() => handleButtonClick('increase', skill.skill_id)}
                          variant="outline"
                          size="sm"
                          disabled={updatingSkills.has(skill.skill_id) || (!skill.is_class_skill && availableSkillPoints === 1)}
                          clicked={clickedButton === `increase-${skill.skill_id}`}
                          aria-label={`Increase ${skill.name}`}
                          title={`Increase ${skill.name} (cost: ${skill.is_class_skill ? '1' : '2'} points)`}
                          className="h-6 w-6 p-0 text-xs"
                        >
                          +
                        </Button>
                      </div>
                    </td>
                    <td className="p-3 text-center text-sm text-[rgb(var(--color-text-secondary))]">
                      {formatModifier(skill.modifier)}
                    </td>
                    <td className="p-3 text-center text-sm text-[rgb(var(--color-text-secondary))]">
                      {(() => {
                        const misc = (skill.feat_bonus || 0) + (skill.item_bonus || 0);
                        return misc ? formatModifier(misc) : display('-');
                      })()}
                    </td>
                    <td className="p-3 text-center">
                      {skill.is_class_skill && (
                        <span className="text-[rgb(var(--color-primary))]">✓</span>
                      )}
                    </td>
                  </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}