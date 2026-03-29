import { useState } from 'react';
import {
  Button, Icon, Menu, MenuItem, Popover, Switch, Tab, Tabs, Tag,
} from '@blueprintjs/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useTranslations } from '@/hooks/useTranslations';
import { useLocale } from '@/providers/LocaleProvider';
import { T } from '../theme';
import { KVRow, ParchmentDialog } from '../shared';

const DUMMY_PATHS = {
  game_folder: { path: 'C:/GOG Games/Neverwinter Nights 2 Complete', exists: true, source: 'auto' as const },
  documents_folder: { path: 'C:/Users/Player/Documents/Neverwinter Nights 2', exists: true, source: 'auto' as const },
  steam_workshop_folder: { path: null as string | null, exists: false, source: 'auto' as const },
  localvault_folder: { path: 'C:/Users/Player/Documents/Neverwinter Nights 2/localvault', exists: true, source: 'derived' as const },
  custom_override_folders: [
    { path: 'D:/NWN2 Mods/override', exists: true },
  ] as { path: string; exists: boolean }[],
  custom_hak_folders: [] as { path: string; exists: boolean }[],
};

const PATH_KEYS = {
  game: 'game_folder',
  documents: 'documents_folder',
  workshop: 'steam_workshop_folder',
} as const;

const CUSTOM_KEYS = {
  override: 'custom_override_folders',
  hak: 'custom_hak_folders',
} as const;

function SectionHeader({ title }: { title: string }) {
  return (
    <div style={{ fontWeight: 700, color: T.textMuted, borderBottom: `1px solid ${T.borderLight}`, paddingBottom: 4, marginBottom: 10 }}>
      {title}
    </div>
  );
}

function PathRow({ label, path, exists, autoDetected, onEdit, onReset }: {
  label: string;
  path: string | null;
  exists: boolean;
  autoDetected: boolean;
  onEdit?: () => void;
  onReset?: () => void;
}) {
  return (
    <div style={{
      display: 'flex', alignItems: 'center', justifyContent: 'space-between',
      padding: '10px 0', borderBottom: `1px solid ${T.borderLight}`,
    }}>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <Icon icon="folder-close" size={16} color={T.textMuted} />
          <span style={{ fontWeight: 600, color: T.text }}>{label}</span>
          <Tag minimal round intent={autoDetected ? 'primary' : 'warning'} style={{ background: autoDetected ? 'rgba(45, 114, 210, 0.1)' : 'rgba(184, 149, 47, 0.1)' }}>
            {autoDetected ? 'Auto-detected' : 'Manually Set'}
          </Tag>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginTop: 4 }}>
          <span style={{ fontFamily: 'monospace', color: T.textMuted, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            {path || '(Not configured)'}
          </span>
          {path && (
            <Icon icon={exists ? 'tick-circle' : 'error'} size={14} color={exists ? T.positive : T.negative} />
          )}
        </div>
      </div>
      <div style={{ display: 'flex', gap: 4, marginLeft: 12 }}>
        {!autoDetected && onReset && (
          <Button small minimal text="Reset" onClick={onReset} style={{ color: T.textMuted }} />
        )}
        {onEdit && (
          <Button small outlined text={path ? 'Change' : 'Set'} onClick={onEdit} />
        )}
      </div>
    </div>
  );
}

function CustomFolderList({ folders, label, onAdd, onRemove }: {
  folders: { path: string; exists: boolean }[];
  label: string;
  onAdd: () => void;
  onRemove: (path: string) => void;
}) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      {folders.map((folder) => (
        <div
          key={folder.path}
          style={{
            display: 'flex', alignItems: 'center', justifyContent: 'space-between',
            padding: '8px 12px', background: T.surfaceAlt, borderRadius: 4,
            border: `1px solid ${T.borderLight}`,
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, minWidth: 0, flex: 1 }}>
            <Icon icon="folder-close" size={14} color={T.textMuted} />
            <span style={{ fontFamily: 'monospace', color: T.text, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {folder.path}
            </span>
            <Icon icon={folder.exists ? 'tick-circle' : 'error'} size={14} color={folder.exists ? T.positive : T.negative} />
          </div>
          <Button small minimal icon="trash" onClick={() => onRemove(folder.path)} style={{ color: T.textMuted }} />
        </div>
      ))}
      <Button
        small
        outlined
        icon="plus"
        text={`Add ${label}`}
        onClick={onAdd}
        style={{ alignSelf: 'flex-start' }}
      />
    </div>
  );
}

function GeneralTab() {
  const { locale, setLocale } = useLocale();
  const t = useTranslations();
  const [fontSize, setFontSize] = useState<string>('medium');
  const [debugExporting, setDebugExporting] = useState(false);
  const [debugResult, setDebugResult] = useState<{ success: boolean; message: string } | null>(null);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <div>
        <SectionHeader title="Language & Region" />
        <KVRow label="Language" value={
          <Popover
            content={<Menu><MenuItem text="English" active={locale === 'en'} onClick={() => setLocale('en')} /></Menu>}
            placement="bottom-end"
            minimal
          >
            <Button minimal rightIcon="caret-down" text={locale === 'en' ? 'English' : locale} style={{ fontWeight: 600 }} />
          </Popover>
        } />
      </div>

      <div>
        <SectionHeader title="Display" />
        <KVRow label="Font Size" value={
          <Popover
            content={
              <Menu>
                <MenuItem text="Small" active={fontSize === 'small'} onClick={() => setFontSize('small')} />
                <MenuItem text="Medium" active={fontSize === 'medium'} onClick={() => setFontSize('medium')} />
                <MenuItem text="Large" active={fontSize === 'large'} onClick={() => setFontSize('large')} />
              </Menu>
            }
            placement="bottom-end"
            minimal
          >
            <Button minimal rightIcon="caret-down" text={fontSize.charAt(0).toUpperCase() + fontSize.slice(1)} style={{ fontWeight: 600 }} />
          </Popover>
        } />
      </div>

      <div>
        <SectionHeader title={t('settings.debug.title')} />
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <Button
            outlined
            text={debugExporting ? t('settings.debug.exporting') : t('settings.debug.exportButton')}
            disabled={debugExporting}
            onClick={async () => {
              setDebugExporting(true);
              setDebugResult(null);
              setTimeout(() => {
                setDebugResult({ success: true, message: 'Debug log saved to: C:/Users/Player/Desktop/debug_log.txt' });
                setDebugExporting(false);
              }, 1000);
            }}
          />
        </div>
        {debugResult && (
          <div style={{
            marginTop: 8, padding: '8px 12px', borderRadius: 4,
            background: debugResult.success ? 'rgba(46, 125, 50, 0.1)' : 'rgba(198, 40, 40, 0.1)',
            border: `1px solid ${debugResult.success ? T.positive : T.negative}`,
            color: debugResult.success ? T.positive : T.negative,
          }}>
            {debugResult.message}
          </div>
        )}
      </div>
    </div>
  );
}

function PathsTab() {
  const [paths, setPaths] = useState(DUMMY_PATHS);

  const selectFolder = async (title: string): Promise<string | null> => {
    try {
      const selected = await open({ directory: true, multiple: false, title });
      return selected as string | null;
    } catch { return null; }
  };

  const updatePath = async (type: keyof typeof PATH_KEYS) => {
    const titles: Record<string, string> = {
      game: 'Select NWN2 Game Folder',
      documents: 'Select NWN2 Documents Folder',
      workshop: 'Select Steam Workshop Folder',
    };
    const selected = await selectFolder(titles[type]);
    if (!selected) return;
    setPaths(prev => ({ ...prev, [PATH_KEYS[type]]: { path: selected, exists: true, source: 'manual' } }));
  };

  const resetPath = (type: keyof typeof PATH_KEYS) => {
    setPaths(prev => ({ ...prev, [PATH_KEYS[type]]: { ...prev[PATH_KEYS[type]], source: 'auto' } }));
  };

  const addCustomFolder = async (type: keyof typeof CUSTOM_KEYS) => {
    const selected = await selectFolder(`Select Custom ${type} Folder`);
    if (!selected) return;
    setPaths(prev => ({ ...prev, [CUSTOM_KEYS[type]]: [...prev[CUSTOM_KEYS[type]], { path: selected, exists: true }] }));
  };

  const removeCustomFolder = (type: keyof typeof CUSTOM_KEYS, path: string) => {
    setPaths(prev => ({ ...prev, [CUSTOM_KEYS[type]]: prev[CUSTOM_KEYS[type]].filter(f => f.path !== path) }));
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <div>
        <SectionHeader title="Main Paths" />
        <PathRow label="Game Installation Folder" path={paths.game_folder.path} exists={paths.game_folder.exists} autoDetected={paths.game_folder.source === 'auto'} onEdit={() => updatePath('game')} onReset={() => resetPath('game')} />
        <PathRow label="Documents Folder" path={paths.documents_folder.path} exists={paths.documents_folder.exists} autoDetected={paths.documents_folder.source === 'auto'} onEdit={() => updatePath('documents')} onReset={() => resetPath('documents')} />
        <PathRow label="Steam Workshop Folder" path={paths.steam_workshop_folder.path} exists={paths.steam_workshop_folder.exists} autoDetected={paths.steam_workshop_folder.source === 'auto'} onEdit={() => updatePath('workshop')} onReset={() => resetPath('workshop')} />
        <PathRow label="Character Vault (LocalVault)" path={paths.localvault_folder.path} exists={paths.localvault_folder.exists} autoDetected={paths.localvault_folder.source === 'derived'} />
      </div>

      <div>
        <SectionHeader title="Custom Override Folders" />
        <CustomFolderList folders={paths.custom_override_folders} label="Override Folder" onAdd={() => addCustomFolder('override')} onRemove={(p) => removeCustomFolder('override', p)} />
      </div>

      <div>
        <SectionHeader title="Custom HAK Folders" />
        <CustomFolderList folders={paths.custom_hak_folders} label="HAK Folder" onAdd={() => addCustomFolder('hak')} onRemove={(p) => removeCustomFolder('hak', p)} />
      </div>
    </div>
  );
}

function GameLaunchTab() {
  const [autoClose, setAutoClose] = useState(false);
  const [showLaunchDialog, setShowLaunchDialog] = useState(true);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <div>
        <SectionHeader title="Launch Behavior" />
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <Switch checked={showLaunchDialog} onChange={() => setShowLaunchDialog(v => !v)} label="Show launch dialog after saving" style={{ marginBottom: 0 }} />
          <Switch checked={autoClose} onChange={() => setAutoClose(v => !v)} label="Auto-close editor after launching game" style={{ marginBottom: 0 }} />
        </div>
      </div>
    </div>
  );
}

function SettingsContent() {
  const [activeTab, setActiveTab] = useState<string>('general');

  return (
    <>
      <div style={{ padding: '10px 16px 0' }}>
        <Tabs
          id="settings-tabs"
          selectedTabId={activeTab}
          onChange={(newTab) => setActiveTab(newTab as string)}
          renderActiveTabPanelOnly
          large
        >
          <Tab id="general" title="General" />
          <Tab id="paths" title="Game Paths" />
          <Tab id="launch" title="Game Launch" />
        </Tabs>
      </div>
      <div style={{ padding: 16, height: 420, overflowY: 'auto' }}>
        {activeTab === 'general' && <GeneralTab />}
        {activeTab === 'paths' && <PathsTab />}
        {activeTab === 'launch' && <GameLaunchTab />}
      </div>
    </>
  );
}

export function SettingsDialog({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  return (
    <ParchmentDialog
      isOpen={isOpen}
      onClose={onClose}
      title="Settings"
      width={680}
      minHeight={480}
      footerActions={
        <Button intent="primary" text="Done" onClick={onClose} />
      }
    >
      <div style={{ margin: -16 }}>
        <SettingsContent />
      </div>
    </ParchmentDialog>
  );
}
