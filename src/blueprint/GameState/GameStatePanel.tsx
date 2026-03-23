import { useState, useMemo } from 'react';
import {
  Button, Card, Elevation, HTMLTable, InputGroup, Menu, MenuItem,
  Popover, Switch, Tab, Tabs,
} from '@blueprintjs/core';
import { T } from '../theme';
import { GAME_STATE } from '../dummy-data';
import { KVRow, ParchmentDialog, StepInput } from '../shared';

const MODIFIED = '#d97706';

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function RestoreBackupDialog({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  const [selected, setSelected] = useState<string | null>(null);

  const selectedName = GAME_STATE.backups.find(b => b.path === selected)?.filename;

  return (
    <ParchmentDialog
      isOpen={isOpen}
      onClose={onClose}
      onOpened={() => setSelected(null)}
      title="Restore Backup"
      width={600}
      minHeight={480}
      footerActions={
        <Button text="Restore" intent="primary" disabled={!selected} onClick={onClose} />
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
        <div style={{ overflowY: 'auto', maxHeight: 380, paddingLeft: 16 }}>
          {GAME_STATE.backups.length === 0 ? (
            <div style={{ padding: 32, textAlign: 'center', color: T.textMuted }}>No backups available.</div>
          ) : GAME_STATE.backups.map(b => {
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
                    {new Date(b.created).toLocaleString()} &mdash; {formatBytes(b.sizeBytes)}
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

// ─── Reputation Tab ──────────────────────────────────────────

function ReputationTab() {
  const [influences, setInfluences] = useState<Record<string, number>>(() => {
    const map: Record<string, number> = {};
    GAME_STATE.companions.forEach(c => { map[c.name] = c.influence; });
    return map;
  });

  const [initial] = useState(() => ({ ...influences }));
  const hasChanges = GAME_STATE.companions.some(c => influences[c.name] !== initial[c.name]);

  return (
    <>
      {hasChanges && (
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 8, padding: '8px 16px', borderBottom: `1px solid ${T.borderLight}` }}>
          <Button small minimal icon="undo" onClick={() => setInfluences({ ...initial })} style={{ color: MODIFIED }}>Revert All</Button>
          <Button small intent="primary">Save</Button>
        </div>
      )}

      {GAME_STATE.companions.map((c) => {
        const current = influences[c.name] ?? c.influence;
        const isModified = current !== initial[c.name];
        const recruitColor = RECRUITMENT_COLOR[c.recruitment] || T.textMuted;

        return (
          <div
            key={c.name}
            style={{
              borderBottom: `1px solid ${T.borderLight}`,
              padding: '10px 16px',
              borderLeft: isModified ? `3px solid ${MODIFIED}` : '3px solid transparent',
              background: isModified ? `${MODIFIED}10` : undefined,
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 4 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <span style={{ fontWeight: 600, color: T.text }}>{c.name}</span>
                <span style={{ fontWeight: 600, color: recruitColor }}>{c.recruitment.replace('_', ' ')}</span>
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
                    if (!isNaN(v)) setInfluences(prev => ({ ...prev, [c.name]: Math.max(-100, Math.min(100, v)) }));
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
                    onClick={() => setInfluences(prev => ({ ...prev, [c.name]: initial[c.name] }))}
                  />
                )}
              </div>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <span style={{ color: T.textMuted, minWidth: 20 }}>-100</span>
              <InfluenceSlider value={current} onChange={v => setInfluences(prev => ({ ...prev, [c.name]: v }))} />
              <span style={{ color: T.textMuted, minWidth: 20, textAlign: 'right' }}>+100</span>
            </div>
          </div>
        );
      })}
    </>
  );
}

// ─── Variable Table with modification tracking ───────────────

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
              <td style={{ fontFamily: 'monospace', color: T.textMuted }}>{v.value}</td>
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

function VariableSection({ integers, strings, floats, edits, onEdit, onRevert, onRevertAll, onSave, changeCount, showRestore, showWarning }: {
  integers: VarEntry[];
  strings: VarEntry[];
  floats: VarEntry[];
  edits: Record<string, string>;
  onEdit: (name: string, value: string) => void;
  onRevert: (name: string) => void;
  onRevertAll: () => void;
  onSave: () => void;
  changeCount: number;
  showRestore?: boolean;
  showWarning?: boolean;
}) {
  const [search, setSearch] = useState('');
  const [restoreOpen, setRestoreOpen] = useState(false);
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
          {showRestore && (
            <Button small minimal icon="history" style={{ color: T.textMuted }} onClick={() => setRestoreOpen(true)}>Restore Backup</Button>
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
      <RestoreBackupDialog isOpen={restoreOpen} onClose={() => setRestoreOpen(false)} />
    </>
  );
}

function useVariableEdits() {
  const [edits, setEdits] = useState<Record<string, string>>({});

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
  const onSave = () => setEdits({});

  return { edits, onEdit, onRevert, onRevertAll, onSave, changeCount: Object.keys(edits).length };
}

// ─── Module Tab ──────────────────────────────────────────────

function ModuleInfoSection() {
  const [selectedModule, setSelectedModule] = useState(
    GAME_STATE.modules.find(m => m.isCurrent)?.id || GAME_STATE.modules[0]?.id || '',
  );

  const selectedLabel = GAME_STATE.modules.find(m => m.id === selectedModule)?.name || 'Select module';
  const moduleMenu = (
    <Menu>
      {GAME_STATE.modules.map(m => (
        <MenuItem
          key={m.id}
          text={`${m.name}${m.isCurrent ? ' (Current)' : ''}`}
          active={selectedModule === m.id}
          onClick={() => setSelectedModule(m.id)}
        />
      ))}
    </Menu>
  );

  return (
    <>
      <div style={{ padding: '10px 16px', display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px' }}>
        <KVRow label="Module" value={
          <Popover content={moduleMenu} placement="bottom-end" minimal>
            <Button minimal rightIcon="caret-down" text={selectedLabel} style={{ fontWeight: 600 }} />
          </Popover>
        } />
        <KVRow label="Campaign" value={GAME_STATE.moduleInfo.campaign} />
        <KVRow label="Current Area" value={GAME_STATE.moduleInfo.currentArea} />
        <KVRow label="Entry Area" value={GAME_STATE.moduleInfo.entryArea} />
      </div>
    </>
  );
}

function ModuleVariablesSection() {
  const varEdits = useVariableEdits();
  return (
    <VariableSection
      integers={GAME_STATE.moduleVars.integers}
      strings={GAME_STATE.moduleVars.strings}
      floats={GAME_STATE.moduleVars.floats}
      showRestore
      {...varEdits}
    />
  );
}

// ─── Campaign Tab ────────────────────────────────────────────

function CampaignSettingsSection() {
  const s = GAME_STATE.campaignSettings;
  const [levelCap, setLevelCap] = useState(s.levelCap);
  const [xpCap, setXpCap] = useState(s.xpCap);
  const [compXp, setCompXp] = useState(s.companionXpWeight);
  const [henchXp, setHenchXp] = useState(s.henchmanXpWeight);
  const [attackNeutrals, setAttackNeutrals] = useState(!!s.attackNeutrals);
  const [autoXp, setAutoXp] = useState(!!s.autoXpAward);
  const [journalSync, setJournalSync] = useState(!!s.journalSync);
  const [noCharChange, setNoCharChange] = useState(!!s.noCharChanging);
  const [personalRep, setPersonalRep] = useState(!!s.usePersonalReputation);

  return (
    <>
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
      </div>

      <div style={{ borderTop: `1px solid ${T.borderLight}`, padding: '10px 16px' }}>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '4px 16px' }}>
          <KVRow label="Start Module" value={<span style={{ fontFamily: 'monospace' }}>{s.startModule}</span>} />
          <KVRow label="Module Count" value={String(s.moduleNames.length)} />
        </div>
        <div style={{ marginTop: 6, color: T.textMuted }}>
          <span style={{ fontWeight: 600 }}>Campaign File: </span>
          <span style={{ fontFamily: 'monospace' }}>{s.campaignFilePath}</span>
          {s.description && (
            <span> &mdash; {s.description}</span>
          )}
        </div>
      </div>
    </>
  );
}

function CampaignVariablesSection() {
  const varEdits = useVariableEdits();
  return (
    <VariableSection
      integers={GAME_STATE.campaignVars.integers}
      strings={GAME_STATE.campaignVars.strings}
      floats={GAME_STATE.campaignVars.floats}
      showRestore
      showWarning
      {...varEdits}
    />
  );
}

// ─── Main Panel ──────────────────────────────────────────────

export function GameStatePanel() {
  const [activeTab, setActiveTab] = useState<string>('reputation');

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

        {activeTab === 'reputation' && <ReputationTab />}
        {activeTab === 'moduleVars' && <ModuleInfoSection />}
        {activeTab === 'campaignSettings' && <CampaignSettingsSection />}
      </Card>

      {activeTab === 'moduleVars' && (
        <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>
          <ModuleVariablesSection />
        </Card>
      )}

      {activeTab === 'campaignSettings' && (
        <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden' }}>
          <CampaignVariablesSection />
        </Card>
      )}
    </div>
  );
}
