import { useState } from 'react';
import { Button, Card, Elevation, HTMLTable, ProgressBar, Tag } from '@blueprintjs/core';
import { T } from '../theme';
import { CHARACTER, LEVEL_HISTORY } from '../dummy-data';
import { mod, fmtNum, StepInput } from '../shared';
import { ClassSelectorDialog } from './ClassSelectorDialog';

interface ClassEntry {
  name: string;
  level: number;
  hitDie: number;
  bab: number;
  fort: number;
  ref: number;
  will: number;
  skillPoints: number;
  type: 'base' | 'prestige';
  maxLevel: number;
  primaryAbility: string;
  isSpellcaster: boolean;
}

function SectionLabel({ children, right }: { children: string; right?: React.ReactNode }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
      <div style={{ fontSize: 12, fontWeight: 700, color: T.accent }}>
        {children}
      </div>
      {right}
    </div>
  );
}

export function ClassesPanel() {
  const [classes, setClasses] = useState<ClassEntry[]>([...CHARACTER.classes]);
  const [xpInput, setXpInput] = useState(CHARACTER.xp.toString());
  const [xp, setXp] = useState(CHARACTER.xp);
  const [selectorOpen, setSelectorOpen] = useState(false);
  const [changingIndex, setChangingIndex] = useState<number | null>(null);

  const maxLevel = 60;
  const maxClasses = 4;
  const totalLevel = classes.reduce((s, c) => s + c.level, 0);
  const totalBab = classes.reduce((s, c) => s + c.bab, 0);
  const totalFort = classes.reduce((s, c) => s + c.fort, 0);
  const totalRef = classes.reduce((s, c) => s + c.ref, 0);
  const totalWill = classes.reduce((s, c) => s + c.will, 0);

  const xpNext = CHARACTER.xpNext;
  const xpLevel = Math.floor((-1 + Math.sqrt(1 + 8 * xp / 1000)) / 2) + 1;
  const hasLevelMismatch = xpLevel !== totalLevel;
  const xpDirty = xpInput !== xp.toString();

  const handleXpSubmit = () => {
    let val = parseInt(xpInput, 10);
    if (isNaN(val) || val < 0) return;
    if (val > 1_770_000) {
      val = 1_770_000;
      setXpInput('1770000');
    }
    setXp(val);
  };

  const handleXpReset = () => {
    setXpInput(xp.toString());
  };

  const handleAdjustLevel = (index: number, delta: number) => {
    setClasses(prev => prev.map((c, i) => {
      if (i !== index) return c;
      const newLevel = Math.max(1, Math.min(c.maxLevel, c.level + delta));
      return { ...c, level: newLevel };
    }));
  };

  const handleRemoveClass = (index: number) => {
    if (classes.length <= 1) return;
    setClasses(prev => prev.filter((_, i) => i !== index));
  };

  const handleOpenChangeClass = (index: number) => {
    setChangingIndex(index);
    setSelectorOpen(true);
  };

  const handleOpenAddClass = () => {
    setChangingIndex(null);
    setSelectorOpen(true);
  };

  const handleSelectClass = (cls: ClassEntry) => {
    if (changingIndex !== null) {
      setClasses(prev => prev.map((c, i) => i === changingIndex ? { ...cls, level: c.level } : c));
    } else {
      setClasses(prev => [...prev, { ...cls, level: 1 }]);
    }
    setSelectorOpen(false);
    setChangingIndex(null);
  };

  return (
    <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 12 }}>

      {/* XP + Classes */}
      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>

        {/* XP bar */}
        <div style={{ padding: '10px 16px', display: 'flex', alignItems: 'center', gap: 12, borderBottom: `1px solid ${T.borderLight}` }}>
          <span style={{ color: T.textMuted, fontSize: 12, fontWeight: 600 }}>XP</span>
          <input
            type="text"
            value={xpInput}
            onChange={(e) => {
              const v = e.target.value;
              if (v === '' || /^\d+$/.test(v)) setXpInput(v);
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleXpSubmit();
              if (e.key === 'Escape') handleXpReset();
            }}
            className="bp6-input"
            style={{ width: 110, textAlign: 'center', fontSize: 13, padding: '2px 8px', height: 26 }}
          />
          <Button small minimal icon="tick" intent="success" onClick={handleXpSubmit} disabled={!xpDirty} style={{ opacity: xpDirty ? 1 : 0.3 }} />
          <Button small minimal icon="cross" onClick={handleXpReset} disabled={!xpDirty} style={{ opacity: xpDirty ? 1 : 0.3 }} />
          <div style={{ flex: 1 }}>
            <ProgressBar value={xp / xpNext} intent="primary" stripes={false} animate={false} style={{ height: 4 }} />
          </div>
          <span style={{ fontSize: 11, color: T.textMuted, whiteSpace: 'nowrap' }}>
            Lvl {xpLevel} | {fmtNum(xpNext - xp)} to next
          </span>
          {hasLevelMismatch && (
            <Tag minimal round intent="warning" icon="warning-sign" style={{ fontSize: 10 }}>
              XP Lvl {xpLevel} / Class Lvl {totalLevel}
            </Tag>
          )}
        </div>

        {/* Classes table */}
        <div style={{ padding: '12px 16px 16px' }}>
          <SectionLabel>Classes</SectionLabel>
          <HTMLTable compact striped bordered style={{ width: '100%', tableLayout: 'fixed' }}>
            <colgroup>
              <col style={{ width: 140 }} />
              <col style={{ width: 110 }} />
              <col style={{ width: 60 }} />
              <col style={{ width: 60 }} />
              <col style={{ width: 60 }} />
              <col style={{ width: 60 }} />
              <col style={{ width: 50 }} />
              <col style={{ width: 50 }} />
              <col style={{ width: 72 }} />
            </colgroup>
            <thead>
              <tr>
                <th>Class</th>
                <th style={{ textAlign: 'center' }}>Level</th>
                <th style={{ textAlign: 'center' }}>BAB</th>
                <th style={{ textAlign: 'center' }}>Fort</th>
                <th style={{ textAlign: 'center' }}>Ref</th>
                <th style={{ textAlign: 'center' }}>Will</th>
                <th style={{ textAlign: 'center' }}>HD</th>
                <th style={{ textAlign: 'center' }}>SP</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {classes.map((cls, i) => (
                <tr key={`${cls.name}-${i}`}>
                  <td>
                    <Button small minimal rightIcon="edit" onClick={() => handleOpenChangeClass(i)}
                      style={{ fontWeight: 500, color: T.accent }}>
                      {cls.name}
                    </Button>
                  </td>
                  <td style={{ textAlign: 'center' }}>
                    <StepInput
                      value={cls.level}
                      onValueChange={(v) => handleAdjustLevel(i, v - cls.level)}
                      min={1}
                      max={Math.min(cls.maxLevel, maxLevel - totalLevel + cls.level)}
                      width={88}
                    />
                    {cls.type === 'prestige' && cls.maxLevel < 60 && (
                      <div style={{ fontSize: 10, color: cls.level >= cls.maxLevel ? T.negative : T.textMuted, marginTop: 1 }}>
                        {cls.level >= cls.maxLevel ? 'Max Level' : `${cls.maxLevel - cls.level} left`}
                      </div>
                    )}
                  </td>
                  <td style={{ textAlign: 'center', fontWeight: 500 }}>{mod(cls.bab)}</td>
                  <td style={{ textAlign: 'center', fontWeight: 500 }}>{mod(cls.fort)}</td>
                  <td style={{ textAlign: 'center', fontWeight: 500 }}>{mod(cls.ref)}</td>
                  <td style={{ textAlign: 'center', fontWeight: 500 }}>{mod(cls.will)}</td>
                  <td style={{ textAlign: 'center' }}>d{cls.hitDie}</td>
                  <td style={{ textAlign: 'center' }}>{cls.skillPoints}</td>
                  <td style={{ textAlign: 'center' }}>
                    {classes.length > 1 && (
                      <Button small minimal icon="trash" intent="danger" onClick={() => handleRemoveClass(i)}>Remove</Button>
                    )}
                  </td>
                </tr>
              ))}
              <tr style={{ fontWeight: 700 }}>
                <td style={{ color: T.textMuted }}>Totals</td>
                <td style={{ textAlign: 'center' }}>{totalLevel}<span style={{ color: T.textMuted, fontWeight: 400 }}>/60</span></td>
                <td style={{ textAlign: 'center' }}>{mod(totalBab)}</td>
                <td style={{ textAlign: 'center' }}>{mod(totalFort)}</td>
                <td style={{ textAlign: 'center' }}>{mod(totalRef)}</td>
                <td style={{ textAlign: 'center' }}>{mod(totalWill)}</td>
                <td />
                <td />
                <td />
              </tr>
            </tbody>
          </HTMLTable>
          {classes.length < maxClasses && totalLevel < maxLevel && (
            <button
              onClick={handleOpenAddClass}
              style={{
                width: '100%', marginTop: 8, padding: '8px 0',
                border: `2px dashed ${T.border}`, borderRadius: 4,
                background: 'transparent', color: T.accent,
                fontSize: 13, fontWeight: 500, cursor: 'pointer',
                transition: 'background 0.15s, border-color 0.15s',
              }}
              onMouseEnter={e => { e.currentTarget.style.background = `${T.accent}10`; e.currentTarget.style.borderColor = T.accent; }}
              onMouseLeave={e => { e.currentTarget.style.background = 'transparent'; e.currentTarget.style.borderColor = T.border; }}
            >
              + Add Class
            </button>
          )}
        </div>
      </Card>

      {/* Level History */}
      <Card elevation={Elevation.ONE} style={{ padding: '12px 16px 16px', background: T.surface }}>
        <SectionLabel>Level History</SectionLabel>
        <HTMLTable compact striped bordered style={{ width: '100%', tableLayout: 'fixed' }}>
          <colgroup>
            <col style={{ width: 64 }} />
            <col style={{ width: 90 }} />
            <col style={{ width: 56 }} />
            <col style={{ width: 50 }} />
            <col style={{ width: 72 }} />
            <col style={{ width: '25%' }} />
            <col />
          </colgroup>
          <thead>
            <tr>
              <th style={{ textAlign: 'center' }}>Level</th>
              <th>Class</th>
              <th style={{ textAlign: 'center' }}>HP</th>
              <th style={{ textAlign: 'center' }}>SP Left</th>
              <th style={{ textAlign: 'center' }}>Ability</th>
              <th>Skills</th>
              <th>Feats</th>
            </tr>
          </thead>
          <tbody>
            {[...LEVEL_HISTORY].reverse().map(lv => (
              <tr key={lv.level}>
                <td style={{ textAlign: 'center', fontWeight: 600 }}>{lv.level}</td>
                <td style={{ color: T.accent, fontWeight: 500 }}>{lv.className} {lv.classLevel}</td>
                <td style={{ textAlign: 'center', color: T.positive, fontWeight: 500 }}>+{lv.hpGained}</td>
                <td style={{ textAlign: 'center' }}>{lv.skillPointsRemaining}</td>
                <td style={{ textAlign: 'center', color: lv.abilityIncrease ? T.accent : T.textMuted, fontWeight: lv.abilityIncrease ? 600 : 400 }}>
                  {lv.abilityIncrease || '-'}
                </td>
                <td>
                  {lv.skillsGained.length > 0
                    ? lv.skillsGained.map(s => `${s.name} +${s.ranks}`).join(', ')
                    : <span style={{ color: T.textMuted }}>-</span>
                  }
                </td>
                <td>
                  {lv.featsGained.length > 0
                    ? lv.featsGained.join(', ')
                    : <span style={{ color: T.textMuted }}>-</span>
                  }
                </td>
              </tr>
            ))}
          </tbody>
        </HTMLTable>
      </Card>

      <ClassSelectorDialog
        isOpen={selectorOpen}
        onClose={() => { setSelectorOpen(false); setChangingIndex(null); }}
        onSelect={handleSelectClass}
        currentClasses={classes}
        isChanging={changingIndex !== null}
        totalLevel={totalLevel}
        maxLevel={maxLevel}
        maxClasses={maxClasses}
      />
    </div>
  );
}
