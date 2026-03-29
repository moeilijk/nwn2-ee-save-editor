import { useState } from 'react';
import { Button, Icon, Popover, Tag, type IconName } from '@blueprintjs/core';
import { T } from '../theme';

interface PendingGain {
  label: string;
  count: number;
  icon: IconName;
  color: string;
  tagIntent: 'success' | 'primary' | 'warning' | 'none';
  tabId: string;
}

interface SpellClass {
  name: string;
  casterType: 'spontaneous' | 'spellbook';
  byLevel: Record<number, number>;
  total: number;
}

const DUMMY_GAINS: PendingGain[] = [
  { label: 'Skill Points', count: 8, icon: 'build', color: T.positive, tagIntent: 'success', tabId: 'skills' },
  { label: 'Feat Slots', count: 1, icon: 'star', color: T.accent, tagIntent: 'primary', tabId: 'feats' },
  { label: 'Ability Score Increase', count: 1, icon: 'properties', color: T.gold, tagIntent: 'warning', tabId: 'abilities' },
];

const DUMMY_SPELLS: SpellClass[] = [
  {
    name: 'Cleric',
    casterType: 'spontaneous',
    byLevel: { 0: 1, 2: 1, 3: 1 },
    total: 3,
  },
];

const SPELL_COLOR = '#5c7cfa';

interface LevelHelperProps {
  onNavigate?: (tabId: string) => void;
}

export function LevelHelper({ onNavigate }: LevelHelperProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [spellsExpanded, setSpellsExpanded] = useState(false);

  const gains = DUMMY_GAINS.filter(g => g.count > 0);
  const spellTotal = DUMMY_SPELLS.reduce((s, c) => s + c.total, 0);
  const hasSpells = spellTotal > 0;
  const totalPending = gains.reduce((s, g) => s + g.count, 0) + spellTotal;

  if (totalPending === 0) return null;

  const handleRowClick = (tabId: string) => {
    setIsOpen(false);
    onNavigate?.(tabId);
  };

  const rowStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '8px 10px',
    borderRadius: 4,
    cursor: 'pointer',
    background: T.surfaceAlt,
    transition: 'background 0.15s',
  };

  const hoverHandlers = {
    onMouseEnter: (e: React.MouseEvent<HTMLDivElement>) => { e.currentTarget.style.background = T.sectionBg; },
    onMouseLeave: (e: React.MouseEvent<HTMLDivElement>) => { e.currentTarget.style.background = T.surfaceAlt; },
  };

  const content = (
    <div style={{ width: 280, background: T.surface, borderRadius: 4, overflow: 'hidden' }}>
      <div style={{
        padding: '10px 14px',
        background: T.sectionBg,
        borderBottom: `1px solid ${T.sectionBorder}`,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
      }}>
        <span style={{ fontSize: 12, fontWeight: 700, color: T.accent, letterSpacing: '0.04em' }}>
          Pending Allocations
        </span>
        <Button minimal small icon="cross" onClick={() => setIsOpen(false)} />
      </div>

      <div style={{ padding: '8px 10px' }}>
        <p style={{ fontSize: 11, color: T.textMuted, margin: '0 0 8px 0' }}>
          You have pending gains to allocate:
        </p>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          {gains.map(gain => (
            <div
              key={gain.tabId}
              onClick={() => handleRowClick(gain.tabId)}
              style={rowStyle}
              {...hoverHandlers}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <Icon icon={gain.icon} size={14} style={{ color: gain.color }} />
                <span style={{ fontSize: 13, fontWeight: 500, color: T.text }}>{gain.label}</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                <Tag minimal round intent={gain.tagIntent} style={{ fontSize: 11, minWidth: 20, textAlign: 'center' }}>
                  {gain.count}
                </Tag>
                <Icon icon="chevron-right" size={12} style={{ color: T.textMuted }} />
              </div>
            </div>
          ))}

          {hasSpells && (
            <div>
              <div style={rowStyle} {...hoverHandlers}>
                <div
                  style={{ display: 'flex', alignItems: 'center', gap: 8, flex: 1 }}
                  onClick={() => handleRowClick('spells')}
                >
                  <Icon icon="flash" size={14} style={{ color: SPELL_COLOR }} />
                  <span style={{ fontSize: 13, fontWeight: 500, color: T.text }}>Spells to Learn</span>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                  <Tag minimal round style={{ fontSize: 11, minWidth: 20, textAlign: 'center', color: SPELL_COLOR }}>
                    {spellTotal}
                  </Tag>
                  <Button
                    minimal
                    small
                    icon={spellsExpanded ? 'chevron-up' : 'chevron-down'}
                    onClick={(e) => { e.stopPropagation(); setSpellsExpanded(!spellsExpanded); }}
                    style={{ minHeight: 16, minWidth: 16, padding: 0 }}
                  />
                </div>
              </div>

              {spellsExpanded && (
                <div style={{
                  marginLeft: 16,
                  paddingLeft: 12,
                  borderLeft: `2px solid ${T.border}`,
                  marginTop: 4,
                }}>
                  {DUMMY_SPELLS.map((cls, i) => (
                    <div key={i} style={{ marginBottom: i < DUMMY_SPELLS.length - 1 ? 8 : 0 }}>
                      <div style={{ fontSize: 12, fontWeight: 600, color: T.text, marginBottom: 2 }}>
                        {cls.name}
                        {cls.casterType === 'spellbook' && (
                          <span style={{ color: T.textMuted, fontWeight: 400, marginLeft: 4 }}>(Spellbook)</span>
                        )}
                      </div>
                      {cls.casterType === 'spellbook' ? (
                        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11, color: T.textMuted, paddingLeft: 8 }}>
                          <span>Free Spells</span>
                          <span style={{ fontWeight: 600, color: SPELL_COLOR }}>{cls.total}</span>
                        </div>
                      ) : (
                        Object.entries(cls.byLevel)
                          .sort(([a], [b]) => Number(a) - Number(b))
                          .map(([level, count]) => (
                            <div key={level} style={{ display: 'flex', justifyContent: 'space-between', fontSize: 11, color: T.textMuted, paddingLeft: 8 }}>
                              <span>{Number(level) === 0 ? 'Cantrips' : `Level ${level}`}</span>
                              <span style={{ fontWeight: 600, color: SPELL_COLOR }}>{count}</span>
                            </div>
                          ))
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );

  return (
    <div style={{ position: 'fixed', bottom: 24, right: 24, zIndex: 50 }}>
      <Popover
        content={content}
        isOpen={isOpen}
        onClose={() => setIsOpen(false)}
        placement="top-end"
        minimal
        popoverClassName="level-helper-popover"
      >
        <Button
          intent="primary"
          icon="warning-sign"
          onClick={() => setIsOpen(!isOpen)}
          style={{
            width: 48,
            height: 48,
            borderRadius: '50%',
            boxShadow: '0 4px 12px rgba(0,0,0,0.25)',
            position: 'relative',
          }}
        />
      </Popover>

      <div style={{
        position: 'absolute',
        top: -4,
        right: -4,
        background: T.negative,
        color: '#fff',
        fontSize: 11,
        fontWeight: 700,
        width: 22,
        height: 22,
        borderRadius: '50%',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        border: `2px solid ${T.surface}`,
        pointerEvents: 'none',
      }}>
        {totalPending}
      </div>
    </div>
  );
}
