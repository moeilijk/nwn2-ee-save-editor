import { useState, useMemo } from 'react';
import { Button, InputGroup, Tab, Tabs, Tag, HTMLTable } from '@blueprintjs/core';
import { ParchmentDialog } from '../shared';
import { T } from '../theme';
import { AVAILABLE_CLASSES } from '../dummy-data';

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

interface AvailableClass {
  id: number;
  name: string;
  label: string;
  type: 'base' | 'prestige';
  focus: string;
  maxLevel: number;
  hitDie: number;
  skillPoints: number;
  isSpellcaster: boolean;
  hasArcane: boolean;
  hasDivine: boolean;
  primaryAbility: string;
  babProgression: string;
  alignmentRestricted: boolean;
  description: string;
}

interface ClassSelectorDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (cls: ClassEntry) => void;
  currentClasses: ClassEntry[];
  isChanging: boolean;
  totalLevel: number;
  maxLevel: number;
  maxClasses: number;
}

function estimateStats(cls: AvailableClass): Omit<ClassEntry, 'level'> {
  const babMap: Record<string, number> = { high: 1, medium: 0, low: 0 };
  return {
    name: cls.name,
    hitDie: cls.hitDie,
    bab: babMap[cls.babProgression] ?? 0,
    fort: 0, ref: 0, will: 0,
    skillPoints: cls.skillPoints,
    type: cls.type,
    maxLevel: cls.maxLevel,
    primaryAbility: cls.primaryAbility,
    isSpellcaster: cls.isSpellcaster,
  };
}

export function ClassSelectorDialog({
  isOpen, onClose, onSelect, currentClasses, isChanging, totalLevel, maxLevel, maxClasses,
}: ClassSelectorDialogProps) {
  const [search, setSearch] = useState('');
  const [tab, setTab] = useState<string>('base');

  const allClasses = useMemo(() => {
    const result: AvailableClass[] = [];
    for (const group of Object.values(AVAILABLE_CLASSES.base)) {
      result.push(...group);
    }
    for (const group of Object.values(AVAILABLE_CLASSES.prestige)) {
      result.push(...group);
    }
    return result;
  }, []);

  const filtered = useMemo(() => {
    if (!search.trim()) return null;
    const q = search.toLowerCase();
    return allClasses.filter(c => c.name.toLowerCase().includes(q) || c.label.toLowerCase().includes(q));
  }, [search, allClasses]);

  const canSelect = (cls: AvailableClass): { ok: boolean; reason?: string } => {
    const hasClass = currentClasses.some(c => c.name === cls.name);
    if (hasClass && !isChanging) return { ok: false, reason: 'Already have this class' };
    if (!isChanging && currentClasses.length >= maxClasses) return { ok: false, reason: `Max ${maxClasses} classes` };
    if (!isChanging && totalLevel >= maxLevel) return { ok: false, reason: 'At max level' };
    if (cls.type === 'prestige' && totalLevel < 6) return { ok: false, reason: 'Requires level 6+' };
    return { ok: true };
  };

  const handleSelect = (cls: AvailableClass) => {
    const { ok } = canSelect(cls);
    if (!ok) return;
    onSelect({ ...estimateStats(cls), level: 1 });
  };

  const renderRow = (cls: AvailableClass) => {
    const { ok, reason } = canSelect(cls);
    return (
      <tr
        key={cls.id}
        onClick={() => ok && handleSelect(cls)}
        style={{
          cursor: ok ? 'pointer' : 'not-allowed',
          opacity: ok ? 1 : 0.5,
        }}
      >
        <td>
          <div style={{ fontWeight: 500, color: ok ? T.text : T.textMuted }}>{cls.name}</div>
          {!ok && reason && (
            <div style={{ fontSize: 10, color: T.negative }}>{reason}</div>
          )}
        </td>
        <td style={{ textAlign: 'center' }}>{cls.primaryAbility}</td>
        <td style={{ textAlign: 'center' }}>d{cls.hitDie}</td>
        <td style={{ textAlign: 'center' }}>{cls.skillPoints}</td>
        <td style={{ textAlign: 'center' }}>
          {cls.isSpellcaster && (
            <Tag minimal round style={{ fontSize: 10 }}>
              {cls.hasArcane ? 'Arcane' : 'Divine'}
            </Tag>
          )}
        </td>
        <td style={{ textAlign: 'center' }}>
          {cls.alignmentRestricted && <Tag minimal round intent="warning" style={{ fontSize: 10 }}>Restricted</Tag>}
        </td>
      </tr>
    );
  };

  const renderFocusGroup = (focusKey: string, classList: AvailableClass[]) => {
    if (!classList.length) return null;
    const info = AVAILABLE_CLASSES.focusInfo[focusKey as keyof typeof AVAILABLE_CLASSES.focusInfo];
    return (
      <div key={focusKey} style={{ marginBottom: 16 }}>
        <div style={{ fontSize: 12, fontWeight: 600, color: T.accent, marginBottom: 4 }}>
          {info?.name || focusKey} ({classList.length})
        </div>
        <HTMLTable compact bordered interactive style={{ width: '100%', tableLayout: 'fixed' }}>
          <colgroup>
            <col />
            <col style={{ width: 60 }} />
            <col style={{ width: 50 }} />
            <col style={{ width: 40 }} />
            <col style={{ width: 70 }} />
            <col style={{ width: 80 }} />
          </colgroup>
          <tbody>
            {classList.map(renderRow)}
          </tbody>
        </HTMLTable>
      </div>
    );
  };

  const baseGroups = AVAILABLE_CLASSES.base;
  const prestigeGroups = AVAILABLE_CLASSES.prestige;

  const totalClasses = allClasses.length;

  return (
    <ParchmentDialog
      isOpen={isOpen}
      onClose={onClose}
      title={isChanging ? 'Change Class' : 'Select Class'}
      width={720}
      footerActions={<></>}
      footerLeft={
        <div style={{ fontSize: 11, color: T.textMuted }}>
          {totalClasses} classes available | {currentClasses.length}/{maxClasses} slots used
        </div>
      }
    >
      <InputGroup
        leftIcon="search"
        placeholder="Search classes..."
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        style={{ marginBottom: 12 }}
      />

      {filtered ? (
        <div>
          <div style={{ fontSize: 12, color: T.textMuted, marginBottom: 8 }}>
            {filtered.length} results for &ldquo;{search}&rdquo;
          </div>
          <HTMLTable compact bordered interactive style={{ width: '100%', tableLayout: 'fixed' }}>
            <colgroup>
              <col />
              <col style={{ width: 60 }} />
              <col style={{ width: 50 }} />
              <col style={{ width: 40 }} />
              <col style={{ width: 70 }} />
              <col style={{ width: 80 }} />
            </colgroup>
            <tbody>
              {filtered.map(renderRow)}
            </tbody>
          </HTMLTable>
        </div>
      ) : (
        <Tabs id="class-type" selectedTabId={tab} onChange={(id) => setTab(id as string)}>
          <Tab id="base" title="Base Classes" panel={
            <div style={{ maxHeight: 400, overflowY: 'auto' }}>
              {Object.entries(baseGroups).map(([focus, list]) => renderFocusGroup(focus, list))}
            </div>
          } />
          <Tab id="prestige" title="Prestige Classes" panel={
            <div style={{ maxHeight: 400, overflowY: 'auto' }}>
              {Object.entries(prestigeGroups).map(([focus, list]) => renderFocusGroup(focus, list))}
            </div>
          } />
        </Tabs>
      )}
    </ParchmentDialog>
  );
}
