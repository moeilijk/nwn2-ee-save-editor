import { useState, useMemo, useEffect, useRef, useCallback } from 'react';
import {
  Button, Card, Elevation, HTMLTable, InputGroup,
  NonIdealState, Spinner, Switch, Tab, Tabs,
} from '@blueprintjs/core';
import { T, formatBytes } from '../theme';
import { KVRow, ParchmentDialog, StepInput } from '../shared';
import { useCharacterContext } from '@/contexts/CharacterContext';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import { gameStateAPI } from '@/services/gameStateApi';
import type {
  CompanionInfluenceData,
  ModuleInfo,
  ModuleVariablesResponse,
  CampaignVariablesResponse,
  CampaignSettingsResponse,
  CampaignBackupInfo,
} from '@/services/gameStateApi';

const MODIFIED = '#d97706';

function RestoreBackupDialog({
  isOpen,
  onClose,
  onRestore,
  backups,
  isLoading,
  isRestoring,
  error,
}: {
  isOpen: boolean;
  onClose: () => void;
  onRestore: (path: string) => void;
  backups: CampaignBackupInfo[];
  isLoading: boolean;
  isRestoring: boolean;
  error: string | null;
}) {
  const [selected, setSelected] = useState<string | null>(null);

  const selectedName = backups.find(b => b.path === selected)?.filename;

  return (
    <ParchmentDialog
      isOpen={isOpen}
      onClose={onClose}
      onOpened={() => setSelected(null)}
      title="Restore Backup"
      width={600}
      minHeight={480}
      footerActions={
        <Button
          text={isRestoring ? 'Restoring...' : 'Restore'}
          intent="primary"
          disabled={!selected || isRestoring}
          onClick={() => selected && onRestore(selected)}
        />
      }
      footerLeft={
        <span style={{ color: T.textMuted }}>
          Selected: {selectedName || 'None'}
        </span>
      }
    >
      <div style={{ display: 'flex', flexDirection: 'column', margin: -16 }}>
        <div style={{ padding: '8px 12px', borderBottom: `1px solid ${T.borderLight}`, color: T.textMuted }}>
          Select a backup to restore. This will overwrite the current campaign/module data.
        </div>
        {error && (
          <div style={{ margin: '8px 12px', padding: '6px 10px', background: '#c6282810', border: `1px solid #c6282840`, borderRadius: 3, color: T.negative }}>
            {error}
          </div>
        )}
        <div style={{ overflowY: 'auto', maxHeight: 380, paddingLeft: 16 }}>
          {isLoading ? (
            <div style={{ padding: 32, textAlign: 'center' }}>
              <Spinner size={24} />
            </div>
          ) : backups.length === 0 ? (
            <div style={{ padding: 32, textAlign: 'center', color: T.textMuted }}>No backups available.</div>
          ) : backups.map(b => {
            const isActive = selected === b.path;
            return (
              <div
                key={b.path}
                onClick={() => setSelected(b.path)}
                style={{
                  display: 'flex', alignItems: 'center', gap: 8,
                  padding: '8px 12px', cursor: 'pointer',
                  background: isActive ? `${T.accent}12` : 'transparent',
                  borderLeft: isActive ? `2px solid ${T.accent}` : '2px solid transparent',
                  borderBottom: `1px solid ${T.borderLight}`,
                }}
              >
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{
                    color: isActive ? T.accent : T.text,
                    fontWeight: isActive ? 600 : 400,
                    fontFamily: 'monospace',
                    overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
                  }}>
                    {b.filename}
                  </div>
                  <div style={{ color: T.textMuted, marginTop: 2 }}>
                    {new Date(b.created).toLocaleString()} &mdash; {formatBytes(b.size_bytes)}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </ParchmentDialog>
  );
}

function InfluenceSlider({ value, onChange }: { value: number; onChange: (v: number) => void }) {
  const pct = ((value + 100) / 200) * 100;
  const handleMouseDown = (e: React.MouseEvent<HTMLDivElement>) => {
    const track = e.currentTarget;
    const update = (clientX: number) => {
      const rect = track.getBoundingClientRect();
      const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
      onChange(Math.round(ratio * 200 - 100));
    };
    update(e.clientX);
    const onMove = (ev: MouseEvent) => update(ev.clientX);
    const onUp = () => { window.removeEventListener('mousemove', onMove); window.removeEventListener('mouseup', onUp); };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  };

  return (
    <div
      onMouseDown={handleMouseDown}
      style={{ flex: 1, height: 20, display: 'flex', alignItems: 'center', cursor: 'pointer', position: 'relative' }}
    >
      <div style={{ position: 'absolute', left: 0, right: 0, height: 4, background: '#d9d0c1', borderRadius: 2 }} />
      <div style={{ position: 'absolute', left: 0, width: `${pct}%`, height: 4, background: T.accent, borderRadius: 2 }} />
      <div style={{
        position: 'absolute', left: `calc(${pct}% - 7px)`,
        width: 14, height: 14, borderRadius: '50%',
        background: T.accent, border: '1px solid #8a4525',
        boxShadow: '0 1px 2px rgba(0,0,0,0.15)',
      }} />
    </div>
  );
}

const RECRUITMENT_COLOR: Record<string, string> = {
  recruited: T.positive,
  met: T.gold,
  not_recruited: T.textMuted,
};

function ReputationTab({ characterId }: { characterId: number }) {
  const [companions, setCompanions] = useState<Record<string, CompanionInfluenceData>>({});
  const initialRef = useRef<Record<string, CompanionInfluenceData> | null>(null);
  const [influences, setInfluences] = useState<Record<string, number>>({});
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setIsLoading(true);
    setError(null);
    gameStateAPI.getCompanionInfluence(characterId)
      .then(res => {
        setCompanions(res.companions);
        if (!initialRef.current) {
          initialRef.current = res.companions;
        }
        const map: Record<string, number> = {};
        Object.entries(res.companions).forEach(([id, data]) => {
          map[id] = data.influence ?? 0;
        });
        setInfluences(map);
      })
      .catch(err => setError(err instanceof Error ? err.message : 'Failed to load companion data'))
      .finally(() => setIsLoading(false));
  }, [characterId]);

  const hasChanges = Object.entries(influences).some(
    ([id, val]) => val !== (initialRef.current?.[id]?.influence ?? 0)
  );

  const handleSave = async () => {
    if (!hasChanges) return;
    setIsSaving(true);
    setError(null);
    try {
      const updates = Object.entries(influences)
        .filter(([id, val]) => val !== (companions[id]?.influence ?? 0))
        .map(([id, val]) => gameStateAPI.updateCompanionInfluence(characterId, id, val));
      await Promise.all(updates);
      const res = await gameStateAPI.getCompanionInfluence(characterId);
      setCompanions(res.companions);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save');
    } finally {
      setIsSaving(false);
    }
  };

  const handleRevertAll = () => {
    if (!initialRef.current) return;
    const map: Record<string, number> = {};
    Object.entries(initialRef.current).forEach(([id, data]) => {
      map[id] = data.influence ?? 0;
    });
    setInfluences(map);
  };

  if (isLoading) {
    return (
      <div style={{ padding: 48, textAlign: 'center' }}>
        <Spinner size={32} />
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ padding: 24 }}>
        <NonIdealState icon="error" title="Failed to load companions" description={error} />
      </div>
    );
  }

  const companionEntries = Object.entries(companions);

  if (companionEntries.length === 0) {
    return (
      <div style={{ padding: 24 }}>
        <NonIdealState icon="person" title="No companion data" description="No companion influence data found for this save." />
      </div>
    );
  }

  return (
    <>
      {(hasChanges || isSaving) && (
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, padding: '8px 16px', borderBottom: `1px solid ${T.borderLight}` }}>
          <Button small minimal icon="undo" onClick={handleRevertAll} style={{ color: MODIFIED }} disabled={isSaving}>Revert All</Button>
          <Button small intent="primary" loading={isSaving} onClick={handleSave}>Save</Button>
        </div>
      )}

      {companionEntries.map(([companionId, companion]) => {
        const current = influences[companionId] ?? (companion.influence ?? 0);
        const initial = initialRef.current?.[companionId]?.influence ?? 0;
        const isModified = current !== initial;
        const recruitColor = RECRUITMENT_COLOR[companion.recruitment] || T.textMuted;

        return (
          <div
            key={companionId}
            style={{
              borderBottom: `1px solid ${T.borderLight}`,
              padding: '10px 16px',
              borderLeft: isModified ? `3px solid ${MODIFIED}` : '3px solid transparent',
              background: isModified ? `${MODIFIED}10` : undefined,
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 4 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <span style={{ fontWeight: 600, color: T.text }}>{companion.name || companionId}</span>
                <span style={{ fontWeight: 600, color: recruitColor }}>{companion.recruitment.replace('_', ' ')}</span>
                {isModified && (
                  <span style={{ fontWeight: 600, color: MODIFIED }}>modified</span>
                )}
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <input
                  type="number"
                  className="bp6-input"
                  value={current}
                  min={-100}
                  max={100}
                  onChange={e => {
                    const v = parseInt(e.target.value, 10);
                    if (!isNaN(v)) setInfluences(prev => ({ ...prev, [companionId]: Math.max(-100, Math.min(100, v)) }));
                  }}
                  style={{
                    width: 56, height: 24, textAlign: 'center', fontWeight: 600,
                    background: T.surface, border: `1px solid ${isModified ? MODIFIED : T.borderLight}`,
                    borderRadius: 3, color: T.text,
                  }}
                />
                {isModified && (
                  <Button
                    small minimal icon="undo"
                    style={{ color: MODIFIED }}
                    onClick={() => setInfluences(prev => ({ ...prev, [companionId]: initial }))}
                  />
                )}
              </div>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <span style={{ color: T.textMuted, minWidth: 20 }}>-100</span>
              <InfluenceSlider value={current} onChange={v => setInfluences(prev => ({ ...prev, [companionId]: v }))} />
              <span style={{ color: T.textMuted, minWidth: 20, textAlign: 'right' }}>+100</span>
            </div>
          </div>
        );
      })}
    </>
  );
}

type VarEntry = { name: string; value: number | string };

function VariableTable({ variables, search, type, edits, onEdit, onRevert }: {
  variables: VarEntry[];
  search: string;
  type: 'int' | 'string' | 'float';
  edits: Record<string, string>;
  onEdit: (name: string, value: string) => void;
  onRevert: (name: string) => void;
}) {
  const filtered = useMemo(() =>
    variables.filter(v => v.name.toLowerCase().includes(search.toLowerCase())),
    [variables, search],
  );

  return (
    <HTMLTable compact striped bordered style={{ width: '100%', tableLayout: 'fixed' }}>
      <colgroup><col /><col style={{ width: 160 }} /><col style={{ width: 200 }} /><col style={{ width: 40 }} /></colgroup>
      <thead>
        <tr><th>Variable</th><th>Current</th><th>Value</th><th /></tr>
      </thead>
      <tbody>
        {filtered.length === 0 ? (
          <tr><td colSpan={4} style={{ textAlign: 'center', color: T.textMuted, padding: 20 }}>No variables found</td></tr>
        ) : filtered.map(v => {
          const isModified = v.name in edits;
          return (
            <tr key={v.name} style={{ borderLeft: isModified ? `3px solid ${MODIFIED}` : undefined, background: isModified ? `${MODIFIED}10` : undefined }}>
              <td style={{ fontFamily: 'monospace', color: T.text }}>{v.name}</td>
              <td style={{ fontFamily: 'monospace', color: T.textMuted }}>{String(v.value)}</td>
              <td>
                <input
                  className="bp6-input"
                  type={type === 'string' ? 'text' : 'number'}
                  value={isModified ? edits[v.name] : v.value}
                  onChange={e => onEdit(v.name, e.target.value)}
                  style={{
                    width: '100%', height: 24, fontFamily: 'monospace',
                    background: T.surface, border: `1px solid ${isModified ? MODIFIED : T.borderLight}`,
                    borderRadius: 3, color: T.text, padding: '2px 6px',
                  }}
                />
              </td>
              <td style={{ textAlign: 'center' }}>
                {isModified && (
                  <Button small minimal icon="undo" style={{ color: MODIFIED }} onClick={() => onRevert(v.name)} />
                )}
              </td>
            </tr>
          );
        })}
      </tbody>
    </HTMLTable>
  );
}

function VariableSection({
  integers, strings, floats, edits, onEdit, onRevert, onRevertAll, onSave, changeCount, showWarning,
  onRestoreClick,
}: {
  integers: VarEntry[];
  strings: VarEntry[];
  floats: VarEntry[];
  edits: Record<string, string>;
  onEdit: (name: string, value: string) => void;
  onRevert: (name: string) => void;
  onRevertAll: () => void;
  onSave: () => void;
  changeCount: number;
  showWarning?: boolean;
  onRestoreClick?: () => void;
}) {
  const [search, setSearch] = useState('');
  const total = integers.length + strings.length + floats.length;

  return (
    <>
      <div style={{ padding: '10px 16px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <span style={{ fontWeight: 600, color: T.textMuted }}>{total} variables</span>
          {changeCount > 0 && (
            <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
              <span style={{ fontWeight: 600, color: MODIFIED }}>{changeCount} modified</span>
              <Button small minimal icon="undo" onClick={onRevertAll} style={{ color: MODIFIED }}>Revert All</Button>
              <Button small intent="primary" onClick={onSave}>Save</Button>
            </div>
          )}
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          {onRestoreClick && (
            <Button small minimal icon="history" style={{ color: T.textMuted }} onClick={onRestoreClick}>Restore Backup</Button>
          )}
          <InputGroup
            small
            leftIcon="search"
            placeholder="Search..."
            value={search}
            onChange={e => setSearch(e.target.value)}
            style={{ width: 180, fontSize: 14 }}
          />
        </div>
      </div>
      {showWarning && changeCount > 0 && (
        <div style={{ margin: '0 16px', padding: '8px 12px', background: `${MODIFIED}12`, border: `1px solid ${MODIFIED}40`, borderRadius: 4, display: 'flex', alignItems: 'center', gap: 8 }}>
          <span style={{ color: MODIFIED, fontWeight: 600 }}>&#9888;</span>
          <span style={{ color: T.text }}>Changes are saved directly to disk. Campaign edits affect all saves in that campaign. Use Revert to restore original values.</span>
        </div>
      )}
      <div style={{ padding: '0 16px 16px' }}>
        <Tabs id={`vars-${total}`} defaultSelectedTabId="integers" renderActiveTabPanelOnly>
          <Tab id="integers" title={`Integers (${integers.length})`}
            panel={<div style={{ paddingTop: 8 }}><VariableTable variables={integers} search={search} type="int" edits={edits} onEdit={onEdit} onRevert={onRevert} /></div>}
          />
          <Tab id="strings" title={`Strings (${strings.length})`}
            panel={<div style={{ paddingTop: 8 }}><VariableTable variables={strings} search={search} type="string" edits={edits} onEdit={onEdit} onRevert={onRevert} /></div>}
          />
          <Tab id="floats" title={`Floats (${floats.length})`}
            panel={<div style={{ paddingTop: 8 }}><VariableTable variables={floats} search={search} type="float" edits={edits} onEdit={onEdit} onRevert={onRevert} /></div>}
          />
        </Tabs>
      </div>
    </>
  );
}

function useVariableEdits(onSaveCallback: (edits: Record<string, string>) => Promise<void>) {
  const { handleError } = useErrorHandler();
  const [edits, setEdits] = useState<Record<string, string>>({});
  const [isSaving, setIsSaving] = useState(false);

  const onEdit = (name: string, value: string) => {
    setEdits(prev => ({ ...prev, [name]: value }));
  };

  const onRevert = (name: string) => {
    setEdits(prev => {
      const next = { ...prev };
      delete next[name];
      return next;
    });
  };

  const onRevertAll = () => setEdits({});

  const onSave = async () => {
    setIsSaving(true);
    try {
      await onSaveCallback(edits);
      setEdits({});
    } catch (err) {
      handleError(err);
    } finally {
      setIsSaving(false);
    }
  };

  return { edits, onEdit, onRevert, onRevertAll, onSave, isSaving, changeCount: Object.keys(edits).length };
}

function useBackupDialog(characterId: number, onRestored: () => void) {
  const [isOpen, setIsOpen] = useState(false);
  const [backups, setBackups] = useState<CampaignBackupInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isRestoring, setIsRestoring] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const open = () => {
    setIsOpen(true);
    setIsLoading(true);
    setError(null);
    gameStateAPI.getCampaignBackups(characterId)
      .then(res => setBackups(res.backups))
      .catch(err => setError(err instanceof Error ? err.message : 'Failed to load backups'))
      .finally(() => setIsLoading(false));
  };

  const close = () => setIsOpen(false);

  const restore = async (path: string) => {
    setIsRestoring(true);
    setError(null);
    try {
      await gameStateAPI.restoreCampaignFromBackup(characterId, path);
      setIsOpen(false);
      onRestored();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to restore');
    } finally {
      setIsRestoring(false);
    }
  };

  return { isOpen, backups, isLoading, isRestoring, error, open, close, restore };
}

function recordToVarEntries(record: Record<string, number | string>): VarEntry[] {
  return Object.entries(record)
    .map(([name, value]) => ({ name, value }))
    .sort((a, b) => a.name.localeCompare(b.name));
}

function ModuleInfoSection({ characterId }: { characterId: number }) {
  const { handleError } = useErrorHandler();
  const [moduleInfo, setModuleInfo] = useState<ModuleInfo | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    setIsLoading(true);
    gameStateAPI.getModuleInfo(characterId)
      .then(info => setModuleInfo(info))
      .catch(handleError)
      .finally(() => setIsLoading(false));
  }, [characterId, handleError]);

  if (isLoading) {
    return <div style={{ padding: 16 }}><Spinner size={20} /></div>;
  }

  if (!moduleInfo) {
    return (
      <div style={{ padding: '10px 16px', display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px' }}>
        <KVRow label="Module" value="-" />
        <KVRow label="Campaign" value="-" />
        <KVRow label="Current Area" value="-" />
        <KVRow label="Entry Area" value="-" />
      </div>
    );
  }

  return (
    <div style={{ padding: '10px 16px', display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px' }}>
      <KVRow label="Module" value={moduleInfo.module_name || '-'} />
      <KVRow label="Campaign" value={moduleInfo.campaign || '-'} />
      <KVRow label="Current Area" value={moduleInfo.area_name || '-'} />
      <KVRow label="Entry Area" value={moduleInfo.entry_area || '-'} />
    </div>
  );
}

function ModuleVariablesSection({ characterId }: { characterId: number }) {
  const [vars, setVars] = useState<ModuleVariablesResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadVars = useCallback(() => {
    setIsLoading(true);
    setError(null);
    gameStateAPI.getModuleVariables(characterId)
      .then(data => setVars(data))
      .catch(err => setError(err instanceof Error ? err.message : 'Failed to load module variables'))
      .finally(() => setIsLoading(false));
  }, [characterId]);

  useEffect(() => { loadVars(); }, [loadVars]);

  const varEdits = useVariableEdits(async (edits) => {
    for (const [name, rawValue] of Object.entries(edits)) {
      const type = vars
        ? (name in vars.integers ? 'int' : name in vars.floats ? 'float' : 'string')
        : 'string';
      const value = type === 'int' ? parseInt(rawValue, 10) : type === 'float' ? parseFloat(rawValue) : rawValue;
      await gameStateAPI.updateModuleVariable(characterId, name, value, type);
    }
    loadVars();
  });

  const backupDialog = useBackupDialog(characterId, loadVars);

  const integers = vars ? recordToVarEntries(vars.integers) : [];
  const strings = vars ? recordToVarEntries(vars.strings) : [];
  const floats = vars ? recordToVarEntries(vars.floats) : [];

  if (isLoading) {
    return <div style={{ padding: 16, textAlign: 'center' }}><Spinner size={20} /></div>;
  }

  if (error) {
    return <div style={{ padding: 16, color: T.negative }}>{error}</div>;
  }

  return (
    <>
      <VariableSection
        integers={integers}
        strings={strings}
        floats={floats}
        onRestoreClick={backupDialog.open}
        {...varEdits}
      />
      <RestoreBackupDialog
        isOpen={backupDialog.isOpen}
        onClose={backupDialog.close}
        onRestore={backupDialog.restore}
        backups={backupDialog.backups}
        isLoading={backupDialog.isLoading}
        isRestoring={backupDialog.isRestoring}
        error={backupDialog.error}
      />
    </>
  );
}

function CampaignSettingsSection({ characterId }: { characterId: number }) {
  const [settings, setSettings] = useState<CampaignSettingsResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [levelCap, setLevelCap] = useState(0);
  const [xpCap, setXpCap] = useState(0);
  const [compXp, setCompXp] = useState(0);
  const [henchXp, setHenchXp] = useState(0);
  const [attackNeutrals, setAttackNeutrals] = useState(false);
  const [autoXp, setAutoXp] = useState(false);
  const [journalSync, setJournalSync] = useState(false);
  const [noCharChange, setNoCharChange] = useState(false);
  const [personalRep, setPersonalRep] = useState(false);

  const populateFromSettings = (s: CampaignSettingsResponse) => {
    setLevelCap(s.level_cap ?? 0);
    setXpCap(s.xp_cap ?? 0);
    setCompXp(s.companion_xp_weight ?? 0);
    setHenchXp(s.henchman_xp_weight ?? 0);
    setAttackNeutrals(!!s.attack_neutrals);
    setAutoXp(!!s.auto_xp_award);
    setJournalSync(!!s.journal_sync);
    setNoCharChange(!!s.no_char_changing);
    setPersonalRep(!!s.use_personal_reputation);
  };

  useEffect(() => {
    setIsLoading(true);
    setError(null);
    gameStateAPI.getCampaignSettings(characterId)
      .then(s => { setSettings(s); populateFromSettings(s); })
      .catch(err => setError(err instanceof Error ? err.message : 'Failed to load campaign settings'))
      .finally(() => setIsLoading(false));
  }, [characterId]);

  const handleSave = async () => {
    if (!settings) return;
    setIsSaving(true);
    setError(null);
    try {
      await gameStateAPI.updateCampaignSettings(characterId, {
        level_cap: levelCap,
        xp_cap: xpCap,
        companion_xp_weight: compXp,
        henchman_xp_weight: henchXp,
        attack_neutrals: attackNeutrals ? 1 : 0,
        auto_xp_award: autoXp ? 1 : 0,
        journal_sync: journalSync ? 1 : 0,
        no_char_changing: noCharChange ? 1 : 0,
        use_personal_reputation: personalRep ? 1 : 0,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save campaign settings');
    } finally {
      setIsSaving(false);
    }
  };

  if (isLoading) {
    return <div style={{ padding: 32, textAlign: 'center' }}><Spinner size={24} /></div>;
  }

  if (error && !settings) {
    return <div style={{ padding: 16 }}><NonIdealState icon="error" title="Failed to load campaign settings" description={error} /></div>;
  }

  if (!settings) {
    return <div style={{ padding: 16 }}><NonIdealState icon="info-sign" title="No campaign" description="No campaign is associated with this save." /></div>;
  }

  return (
    <>
      {error && (
        <div style={{ margin: '8px 16px', padding: '6px 10px', background: '#c6282810', border: `1px solid #c6282840`, borderRadius: 3, color: T.negative }}>
          {error}
        </div>
      )}
      <div style={{ padding: '10px 16px' }}>
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
          <div>
            <div style={{ fontWeight: 700, color: T.textMuted, borderBottom: `1px solid ${T.borderLight}`, paddingBottom: 4, marginBottom: 10 }}>
              Progression
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ color: T.textMuted }}>Level Cap</span>
                <StepInput value={levelCap} onValueChange={setLevelCap} min={1} max={40} width={100} />
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ color: T.textMuted }}>XP Cap</span>
                <StepInput value={xpCap} onValueChange={setXpCap} min={0} max={2000000} step={1000} width={120} />
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ color: T.textMuted }}>Companion XP Weight</span>
                <StepInput value={compXp} onValueChange={setCompXp} min={0} max={1} width={100} />
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span style={{ color: T.textMuted }}>Henchman XP Weight</span>
                <StepInput value={henchXp} onValueChange={setHenchXp} min={0} max={1} width={100} />
              </div>
            </div>
          </div>

          <div>
            <div style={{ fontWeight: 700, color: T.textMuted, borderBottom: `1px solid ${T.borderLight}`, paddingBottom: 4, marginBottom: 10 }}>
              Gameplay Flags
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              <Switch checked={attackNeutrals} onChange={() => setAttackNeutrals(v => !v)} label="Attack Neutrals" style={{ marginBottom: 0 }} />
              <Switch checked={autoXp} onChange={() => setAutoXp(v => !v)} label="Auto XP Award" style={{ marginBottom: 0 }} />
              <Switch checked={journalSync} onChange={() => setJournalSync(v => !v)} label="Journal Sync" style={{ marginBottom: 0 }} />
              <Switch checked={noCharChange} onChange={() => setNoCharChange(v => !v)} label="Lock Character Changes" style={{ marginBottom: 0 }} />
              <Switch checked={personalRep} onChange={() => setPersonalRep(v => !v)} label="Use Personal Reputation" style={{ marginBottom: 0 }} />
            </div>
          </div>
        </div>

        <div style={{ marginTop: 12, display: 'flex', justifyContent: 'flex-end' }}>
          <Button intent="primary" loading={isSaving} onClick={handleSave}>Save Settings</Button>
        </div>
      </div>

      <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px' }}>
          <KVRow label="Start Module" value={<span style={{ fontFamily: 'monospace' }}>{settings.start_module || '-'}</span>} />
          <KVRow label="Module Count" value={String(settings.module_names?.length ?? 0)} />
        </div>
        <div style={{ marginTop: 6, color: T.textMuted }}>
          <span style={{ fontWeight: 600 }}>Campaign File: </span>
          <span style={{ fontFamily: 'monospace' }}>{settings.campaign_file_path || '-'}</span>
          {settings.description && (
            <span> &mdash; {settings.description}</span>
          )}
        </div>
      </div>
    </>
  );
}

function CampaignVariablesSection({ characterId }: { characterId: number }) {
  const [vars, setVars] = useState<CampaignVariablesResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadVars = useCallback(() => {
    setIsLoading(true);
    setError(null);
    gameStateAPI.getCampaignVariables(characterId)
      .then(data => setVars(data))
      .catch(err => setError(err instanceof Error ? err.message : 'Failed to load campaign variables'))
      .finally(() => setIsLoading(false));
  }, [characterId]);

  useEffect(() => { loadVars(); }, [loadVars]);

  const varEdits = useVariableEdits(async (edits) => {
    for (const [name, rawValue] of Object.entries(edits)) {
      const type = vars
        ? (name in vars.integers ? 'int' : name in vars.floats ? 'float' : 'string')
        : 'string';
      const value = type === 'int' ? parseInt(rawValue, 10) : type === 'float' ? parseFloat(rawValue) : rawValue;
      await gameStateAPI.updateCampaignVariable(characterId, name, value, type);
    }
    loadVars();
  });

  const backupDialog = useBackupDialog(characterId, loadVars);

  const integers = vars ? recordToVarEntries(vars.integers) : [];
  const strings = vars ? recordToVarEntries(vars.strings) : [];
  const floats = vars ? recordToVarEntries(vars.floats) : [];

  if (isLoading) {
    return <div style={{ padding: 16, textAlign: 'center' }}><Spinner size={20} /></div>;
  }

  if (error) {
    return <div style={{ padding: 16, color: T.negative }}>{error}</div>;
  }

  return (
    <>
      <VariableSection
        integers={integers}
        strings={strings}
        floats={floats}
        showWarning
        onRestoreClick={backupDialog.open}
        {...varEdits}
      />
      <RestoreBackupDialog
        isOpen={backupDialog.isOpen}
        onClose={backupDialog.close}
        onRestore={backupDialog.restore}
        backups={backupDialog.backups}
        isLoading={backupDialog.isLoading}
        isRestoring={backupDialog.isRestoring}
        error={backupDialog.error}
      />
    </>
  );
}

export function GameStatePanel() {
  const { character } = useCharacterContext();
  const characterId = character?.id;
  const [activeTab, setActiveTab] = useState<string>('reputation');

  if (!characterId) {
    return (
      <div style={{ padding: 48 }}>
        <NonIdealState icon="person" title="No character loaded" description="Load a save file to view game state." />
      </div>
    );
  }

  return (
    <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 12 }}>
      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>
        <div style={{ padding: '10px 16px 0' }}>
          <Tabs
            id="gamestate-tabs"
            selectedTabId={activeTab}
            onChange={(newTab) => setActiveTab(newTab as string)}
            renderActiveTabPanelOnly
            large
          >
            <Tab id="reputation" title="Companion Influence" />
            <Tab id="moduleVars" title="Module & Variables" />
            <Tab id="campaignSettings" title="Campaign & Variables" />
          </Tabs>
        </div>

        {activeTab === 'reputation' && <ReputationTab characterId={characterId} />}
        {activeTab === 'moduleVars' && <ModuleInfoSection characterId={characterId} />}
        {activeTab === 'campaignSettings' && <CampaignSettingsSection characterId={characterId} />}
      </Card>

      {activeTab === 'moduleVars' && (
        <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>
          <ModuleVariablesSection characterId={characterId} />
        </Card>
      )}

      {activeTab === 'campaignSettings' && (
        <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>
          <CampaignVariablesSection characterId={characterId} />
        </Card>
      )}
    </div>
  );
}
