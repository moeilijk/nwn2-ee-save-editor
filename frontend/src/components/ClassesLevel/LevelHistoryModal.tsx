
import { useEffect, useState, useCallback } from 'react';
import { useCharacterContext } from '@/contexts/CharacterContext';
import { useTranslations } from '@/hooks/useTranslations';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { invoke } from '@tauri-apps/api/core';
import { display } from '@/utils/dataHelpers';
import { ChevronDown, ChevronRight } from 'lucide-react';

const X = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
  </svg>
);

// Matches Rust ResolvedLevelHistoryEntry struct
interface LevelHistoryEntry {
  character_level: number;
  class_id: number;
  class_name: string;
  class_level: number;
  hp_gained: number;
  skill_points_remaining: number;
  ability_increase: string | null;
  feats_gained: { feat_id: number; name: string }[];
  skills_gained: { skill_id: number; name: string; ranks: number }[];
}

interface LevelHistoryModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function LevelHistoryModal({ isOpen, onClose }: LevelHistoryModalProps) {
  const { characterId } = useCharacterContext();
  const t = useTranslations();
  const [history, setHistory] = useState<LevelHistoryEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [expandedLevels, setExpandedLevels] = useState<Set<number>>(new Set());

  const fetchHistory = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<LevelHistoryEntry[]>('get_level_history');
      setHistory(data || []);
      if (data?.length) {
        setExpandedLevels(new Set(data.map((h: LevelHistoryEntry) => h.character_level)));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An unknown error occurred');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (isOpen && characterId) {
      fetchHistory();
    }
  }, [isOpen, characterId, fetchHistory]);


  const toggleLevel = (level: number) => {
    setExpandedLevels(prev => {
      const next = new Set(prev);
      if (next.has(level)) {
        next.delete(level);
      } else {
        next.add(level);
      }
      return next;
    });
  };

  const collapseAll = () => setExpandedLevels(new Set());
  const expandAll = () => setExpandedLevels(new Set(history.map(h => h.character_level)));

  if (!isOpen) return null;

  return (
    <div className="level-history-modal-overlay">
      <Card className="level-history-modal-container">
        <CardContent padding="p-0" className="flex flex-col h-full">
          <div className="level-history-modal-header">
            <div className="level-history-modal-header-row">
              <h3 className="level-history-modal-title">
                {t('classes.levelHistory')}
              </h3>
              <Button
                onClick={onClose}
                variant="ghost"
                size="sm"
                className="level-history-modal-close-button"
              >
                <X className="w-4 h-4" />
              </Button>
            </div>
          </div>

          <div className="level-history-modal-content">
            {loading ? (
              <div className="level-history-modal-loading">
                <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-[rgb(var(--color-primary))]"></div>
                <span>{t('common.loading')}</span>
              </div>
            ) : error ? (
              <div className="level-history-modal-error">
                {t('common.error')}: {error}
              </div>
            ) : history.length === 0 ? (
              <div className="level-history-modal-empty">
                {t('classes.noLevelHistory')}
              </div>
            ) : (
              <div className="level-history-modal-list">
                <div className="level-history-modal-controls">
                  <Button variant="outline" size="sm" onClick={expandAll}>
                    {t('common.expandAll')}
                  </Button>
                  <Button variant="outline" size="sm" onClick={collapseAll}>
                    {t('common.collapseAll')}
                  </Button>
                </div>

                {[...history].reverse().map((entry) => (
                  <Card
                    key={entry.character_level}
                    className="level-history-modal-level-card"
                  >
                    <button
                      onClick={() => toggleLevel(entry.character_level)}
                      className="level-history-modal-level-header"
                    >
                      <div className="level-history-modal-level-title">
                        {expandedLevels.has(entry.character_level) ? (
                          <ChevronDown className="w-4 h-4 text-[rgb(var(--color-text-muted))]" />
                        ) : (
                          <ChevronRight className="w-4 h-4 text-[rgb(var(--color-text-muted))]" />
                        )}
                        <span className="level-history-modal-level-number">
                          {t('classes.level')} {entry.character_level}
                        </span>
                      </div>
                    </button>

                    {expandedLevels.has(entry.character_level) && (
                      <div className="level-history-modal-level-details">
                        <div className="level-history-modal-section">
                          <div className="level-history-modal-section-title">
                            {t('classes.class')}
                          </div>
                          <ul className="level-history-modal-list">
                            <li className="level-history-modal-list-item">
                              {display(entry.class_name)} {t('classes.level')} {entry.class_level}
                            </li>
                          </ul>
                        </div>

                        <div className="level-history-modal-section">
                          <div className="level-history-modal-section-title">
                            {t('classes.stats')}
                          </div>
                          <ul className="level-history-modal-list">
                            <li className="level-history-modal-list-item">
                              {t('classes.hpGained')}: <span className="level-history-modal-hp">+{entry.hp_gained}</span>
                            </li>
                            <li className="level-history-modal-list-item">
                              {t('classes.skillPointsRemaining')}: <span>{entry.skill_points_remaining}</span>
                            </li>
                            {entry.ability_increase && (
                              <li className="level-history-modal-list-item">
                                {t('classes.abilityIncrease')}: <span className="level-history-modal-ability">{entry.ability_increase}</span>
                              </li>
                            )}
                          </ul>
                        </div>

                        {(entry.skills_gained?.length ?? 0) > 0 && (
                          <div className="level-history-modal-section">
                            <div className="level-history-modal-section-title">
                              {t('classes.skills')}
                            </div>
                            <ul className="level-history-modal-list">
                              {entry.skills_gained.map((skill, idx) => (
                                <li key={idx} className="level-history-modal-list-item">
                                  {display(skill.name)} <span className="level-history-modal-skill-rank">+{skill.ranks}</span>
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}

                        {(entry.feats_gained?.length ?? 0) > 0 && (
                          <div className="level-history-modal-section">
                            <div className="level-history-modal-section-title">
                              {t('classes.feats')}
                            </div>
                            <ul className="level-history-modal-list">
                              {entry.feats_gained.map((feat, idx) => (
                                <li key={idx} className="level-history-modal-list-item">
                                  {display(feat.name)}
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}

                        {/* TODO: spells_learned and spells_removed not yet in Rust backend */}

                        {(entry.skills_gained?.length ?? 0) === 0 &&
                         (entry.feats_gained?.length ?? 0) === 0 && (
                          <div className="level-history-modal-empty-level">
                            {t('classes.noGainsRecorded')}
                          </div>
                        )}
                      </div>
                    )}
                  </Card>
                ))}
              </div>
            )}
          </div>

          <div className="level-history-modal-footer">
            <span>{history.length} {t('classes.levelsRecorded')}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
