import { useState, useMemo } from 'react';
import { Button, InputGroup, Tab, Tabs, Tag, HTMLTable } from '@blueprintjs/core';
import { ParchmentDialog } from '../shared';
import { T } from '../theme';
import type { CategorizedClassesResponse, ClassInfo } from '@/hooks/useClassesLevel';

interface ClassSelectorDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (classInfo: ClassInfo) => void;
  currentClassIds: number[];
  categorizedClasses: CategorizedClassesResponse | null | undefined;
  isChanging: boolean;
  totalLevel: number;
  maxLevel: number;
  maxClasses: number;
  currentClassCount: number;
}

function canSelect(
  cls: ClassInfo,
  currentClassIds: number[],
  isChanging: boolean,
  totalLevel: number,
  maxLevel: number,
  maxClasses: number,
  currentClassCount: number,
): { ok: boolean; reason?: string } {
  const hasClass = currentClassIds.includes(cls.id);
  if (hasClass && !isChanging) return { ok: false, reason: 'Already have this class' };
  if (!isChanging && currentClassCount >= maxClasses) return { ok: false, reason: `Max ${maxClasses} classes` };
  if (!isChanging && totalLevel >= maxLevel) return { ok: false, reason: 'At max level' };
  if (cls.type === 'prestige' && totalLevel < 6) return { ok: false, reason: 'Requires level 6+' };
  return { ok: true };
}

export function ClassSelectorDialog({
  isOpen, onClose, onSelect, currentClassIds, categorizedClasses, isChanging,
  totalLevel, maxLevel, maxClasses, currentClassCount,
}: ClassSelectorDialogProps) {
  const [search, setSearch] = useState('');
  const [tab, setTab] = useState<string>('base');

  const allClasses = useMemo((): ClassInfo[] => {
    if (!categorizedClasses) return [];
    const result: ClassInfo[] = [];
    for (const focusClasses of Object.values(categorizedClasses.categories.base)) {
      result.push(...focusClasses);
    }
    for (const focusClasses of Object.values(categorizedClasses.categories.prestige)) {
      result.push(...focusClasses);
    }
    return result;
  }, [categorizedClasses]);

  const filtered = useMemo(() => {
    if (!search.trim()) return null;
    const q = search.toLowerCase();
    return allClasses.filter(c =>
      c.name.toLowerCase().includes(q) || c.label.toLowerCase().includes(q)
    );
  }, [search, allClasses]);

  const renderRow = (cls: ClassInfo) => {
    const { ok, reason } = canSelect(cls, currentClassIds, isChanging, totalLevel, maxLevel, maxClasses, currentClassCount);
    return (
      <tr
        key={cls.id}
        onClick={() => ok && onSelect(cls)}
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
        <td style={{ textAlign: 'center' }}>{cls.primary_ability}</td>
        <td style={{ textAlign: 'center' }}>d{cls.hit_die}</td>
        <td style={{ textAlign: 'center' }}>{cls.skill_points}</td>
        <td style={{ textAlign: 'center' }}>
          {cls.is_spellcaster && (
            <Tag minimal round style={{ fontSize: 10 }}>
              {cls.has_arcane ? 'Arcane' : 'Divine'}
            </Tag>
          )}
        </td>
        <td style={{ textAlign: 'center' }}>
          {cls.alignment_restricted && (
            <Tag minimal round intent="warning" style={{ fontSize: 10 }}>Restricted</Tag>
          )}
        </td>
      </tr>
    );
  };

  const renderFocusGroup = (focusKey: string, classList: ClassInfo[]) => {
    if (!classList.length) return null;
    const focusInfo = categorizedClasses?.focus_info[focusKey];
    return (
      <div key={focusKey} style={{ marginBottom: 16 }}>
        <div style={{ fontSize: 12, fontWeight: 600, color: T.accent, marginBottom: 4 }}>
          {focusInfo?.name || focusKey} ({classList.length})
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

  const baseGroups = categorizedClasses?.categories.base ?? {};
  const prestigeGroups = categorizedClasses?.categories.prestige ?? {};
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
          {totalClasses} classes available | {currentClassCount}/{maxClasses} slots used
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

      {!categorizedClasses ? (
        <div style={{ color: T.textMuted, fontSize: 12, padding: '8px 0' }}>
          Loading class data...
        </div>
      ) : filtered ? (
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

      <Button minimal style={{ marginTop: 8 }} onClick={onClose}>Cancel</Button>
    </ParchmentDialog>
  );
}
