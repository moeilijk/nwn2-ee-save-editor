import { useState, useEffect } from 'react';
import {
  Button, Menu, MenuItem, Popover, Switch, Tab, Tabs, Tag,
} from '@blueprintjs/core';
import { GiFullFolder, GiCheckMark, GiBrokenShield } from 'react-icons/gi';
import { GameIcon } from '../shared/GameIcon';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useTranslations } from '@/hooks/useTranslations';
import { useLocale } from '@/providers/LocaleProvider';
import { pathService, PathConfig } from '@/lib/api/paths';
import { T } from '../theme';
import { KVRow, ParchmentDialog } from '../shared';

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
  const t = useTranslations();
  return (
    <div style={{
      display: 'flex', alignItems: 'center', justifyContent: 'space-between',
      padding: '10px 0', borderBottom: `1px solid ${T.borderLight}`,
    }}>
      <div style={{ flex: 1, minWidth: 0 }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <GameIcon icon={GiFullFolder} size={16} color={T.textMuted} />
          <span style={{ fontWeight: 600, color: T.text }}>{label}</span>
          <Tag minimal round intent={autoDetected ? 'primary' : 'warning'} style={{ background: autoDetected ? 'rgba(45, 114, 210, 0.1)' : 'rgba(184, 149, 47, 0.1)' }}>
            {autoDetected ? t('settings.paths.autoDetected') : t('settings.paths.manuallySet')}
          </Tag>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginTop: 4 }}>
          <span style={{ fontFamily: 'monospace', color: T.textMuted, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            {path || t('settings.paths.notConfigured')}
          </span>
          {path && (
            <GameIcon icon={exists ? GiCheckMark : GiBrokenShield} size={14} color={exists ? T.positive : T.negative} />
          )}
        </div>
      </div>
      <div style={{ display: 'flex', gap: 4, marginLeft: 12 }}>
        {!autoDetected && onReset && (
          <Button small minimal text={t('settings.paths.reset')} onClick={onReset} style={{ color: T.textMuted }} />
        )}
        {onEdit && (
          <Button small outlined text={path ? t('settings.paths.change') : t('settings.paths.set')} onClick={onEdit} />
        )}
      </div>
    </div>
  );
}

function CustomFolderList({ folders, addLabel, onAdd, onRemove }: {
  folders: { path: string; exists: boolean }[];
  addLabel: string;
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
            <GameIcon icon={GiFullFolder} size={14} color={T.textMuted} />
            <span style={{ fontFamily: 'monospace', color: T.text, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {folder.path}
            </span>
            <GameIcon icon={folder.exists ? GiCheckMark : GiBrokenShield} size={14} color={folder.exists ? T.positive : T.negative} />
          </div>
          <Button small minimal icon="trash" onClick={() => onRemove(folder.path)} style={{ color: T.textMuted }} />
        </div>
      ))}
      <Button
        small
        outlined
        icon="plus"
        text={addLabel}
        onClick={onAdd}
        style={{ alignSelf: 'flex-start' }}
      />
    </div>
  );
}

function GeneralTab() {
  const { setLocale } = useLocale();
  const t = useTranslations();
  const [language, setLanguage] = useState('en');
  const [fontSize, setFontSize] = useState('medium');
  const [debugExporting, setDebugExporting] = useState(false);
  const [debugResult, setDebugResult] = useState<{ success: boolean; message: string } | null>(null);

  useEffect(() => {
    invoke<{ language: string; font_size: string }>('get_app_config')
      .then((config) => {
        setLanguage(config.language);
        setFontSize(config.font_size);
      })
      .catch(() => {});
  }, []);

  const updateSetting = (key: string, value: string) => {
    invoke('update_app_config', { updates: { [key]: value } }).catch(() => {});
  };

  const handleLanguageChange = (newLocale: string) => {
    setLanguage(newLocale);
    setLocale(newLocale);
    updateSetting('language', newLocale);
  };

  const handleFontSizeChange = (size: string) => {
    setFontSize(size);
    updateSetting('font_size', size);
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <div>
        <SectionHeader title={t('settings.general.languageRegion')} />
        <KVRow label={t('settings.general.language')} value={
          <Popover
            content={<Menu><MenuItem text={t('settings.general.languageEnglish')} active={language === 'en'} onClick={() => handleLanguageChange('en')} /></Menu>}
            placement="bottom-end"
            minimal
          >
            <Button minimal rightIcon="caret-down" text={language === 'en' ? t('settings.general.languageEnglish') : language} style={{ fontWeight: 600 }} />
          </Popover>
        } />
      </div>

      <div>
        <SectionHeader title={t('settings.general.display')} />
        <KVRow label={t('settings.general.fontSize')} value={
          <Popover
            content={
              <Menu>
                <MenuItem text={t('settings.general.fontSizeSmall')} active={fontSize === 'small'} onClick={() => handleFontSizeChange('small')} />
                <MenuItem text={t('settings.general.fontSizeMedium')} active={fontSize === 'medium'} onClick={() => handleFontSizeChange('medium')} />
                <MenuItem text={t('settings.general.fontSizeLarge')} active={fontSize === 'large'} onClick={() => handleFontSizeChange('large')} />
              </Menu>
            }
            placement="bottom-end"
            minimal
          >
            <Button minimal rightIcon="caret-down" text={t(`settings.general.fontSize${fontSize.charAt(0).toUpperCase() + fontSize.slice(1)}`)} style={{ fontWeight: 600 }} />
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
              try {
                const filePath = await invoke<string>('export_debug_log');
                setDebugResult({ success: true, message: `${t('settings.debug.exportSuccess')} ${filePath}` });
              } catch {
                setDebugResult({ success: false, message: t('settings.debug.exportError') });
              } finally {
                setDebugExporting(false);
              }
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
  const t = useTranslations();
  const [paths, setPaths] = useState<PathConfig | null>(null);

  useEffect(() => {
    pathService.getConfig().then((response) => setPaths(response.paths)).catch(() => {});
  }, []);

  const selectFolder = async (title: string): Promise<string | null> => {
    try {
      const selected = await open({ directory: true, multiple: false, title });
      return selected as string | null;
    } catch { return null; }
  };

  const reloadConfig = async () => {
    const response = await pathService.getConfig();
    setPaths(response.paths);
  };

  const updatePath = async (type: 'game' | 'documents' | 'workshop') => {
    const titleKeys: Record<string, string> = {
      game: t('settings.paths.selectGameFolder'),
      documents: t('settings.paths.selectDocumentsFolder'),
      workshop: t('settings.paths.selectWorkshopFolder'),
    };
    const selected = await selectFolder(titleKeys[type]);
    if (!selected) return;
    try {
      switch (type) {
        case 'game': await pathService.setGameFolder(selected); break;
        case 'documents': await pathService.setDocumentsFolder(selected); break;
        case 'workshop': await pathService.setSteamWorkshopFolder(selected); break;
      }
      await reloadConfig();
    } catch { /* ignore */ }
  };

  const resetPath = async (type: 'game' | 'documents' | 'workshop') => {
    try {
      switch (type) {
        case 'game': await pathService.resetGameFolder(); break;
        case 'documents': await pathService.resetDocumentsFolder(); break;
        case 'workshop': await pathService.resetSteamWorkshopFolder(); break;
      }
      await reloadConfig();
    } catch { /* ignore */ }
  };

  const addCustomFolder = async (type: 'override' | 'hak') => {
    const title = type === 'override' ? t('settings.paths.selectOverrideFolder') : t('settings.paths.selectHakFolder');
    const selected = await selectFolder(title);
    if (!selected) return;
    try {
      if (type === 'override') {
        await pathService.addOverrideFolder(selected);
      } else {
        await pathService.addHakFolder(selected);
      }
      await reloadConfig();
    } catch { /* ignore */ }
  };

  const removeCustomFolder = async (type: 'override' | 'hak', path: string) => {
    try {
      if (type === 'override') {
        await pathService.removeOverrideFolder(path);
      } else {
        await pathService.removeHakFolder(path);
      }
      await reloadConfig();
    } catch { /* ignore */ }
  };

  if (!paths) {
    return <div style={{ color: T.textMuted, padding: 16 }}>{t('settings.debug.exporting')}</div>;
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <div>
        <SectionHeader title={t('settings.paths.mainPaths')} />
        <PathRow label={t('settings.paths.gameFolder')} path={paths.game_folder.path} exists={paths.game_folder.exists} autoDetected={paths.game_folder.source === 'auto'} onEdit={() => updatePath('game')} onReset={() => resetPath('game')} />
        <PathRow label={t('settings.paths.documentsFolder')} path={paths.documents_folder.path} exists={paths.documents_folder.exists} autoDetected={paths.documents_folder.source === 'auto'} onEdit={() => updatePath('documents')} onReset={() => resetPath('documents')} />
        <PathRow label={t('settings.paths.workshopFolder')} path={paths.steam_workshop_folder.path} exists={paths.steam_workshop_folder.exists} autoDetected={paths.steam_workshop_folder.source === 'auto'} onEdit={() => updatePath('workshop')} onReset={() => resetPath('workshop')} />
        <PathRow label={t('settings.paths.localvaultFolder')} path={paths.localvault_folder.path} exists={paths.localvault_folder.exists} autoDetected={paths.localvault_folder.source === 'derived'} />
      </div>

      <div>
        <SectionHeader title={t('settings.paths.customOverrideFolders')} />
        <CustomFolderList folders={paths.custom_override_folders} addLabel={t('settings.paths.addOverrideFolder')} onAdd={() => addCustomFolder('override')} onRemove={(p) => removeCustomFolder('override', p)} />
      </div>

      <div>
        <SectionHeader title={t('settings.paths.customHakFolders')} />
        <CustomFolderList folders={paths.custom_hak_folders} addLabel={t('settings.paths.addHakFolder')} onAdd={() => addCustomFolder('hak')} onRemove={(p) => removeCustomFolder('hak', p)} />
      </div>
    </div>
  );
}

function GameLaunchTab() {
  const t = useTranslations();
  const [showLaunchDialog, setShowLaunchDialog] = useState(true);
  const [autoCloseOnLaunch, setAutoCloseOnLaunch] = useState(false);

  useEffect(() => {
    invoke<{ show_launch_dialog: boolean; auto_close_on_launch: boolean }>('get_app_config')
      .then((config) => {
        setShowLaunchDialog(config.show_launch_dialog);
        setAutoCloseOnLaunch(config.auto_close_on_launch);
      })
      .catch(() => {});
  }, []);

  const updateSetting = (key: string, value: boolean) => {
    invoke('update_app_config', { updates: { [key]: value } }).catch(() => {});
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <div>
        <SectionHeader title={t('settings.launch.launchBehavior')} />
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <Switch
            checked={showLaunchDialog}
            onChange={() => {
              const next = !showLaunchDialog;
              setShowLaunchDialog(next);
              updateSetting('show_launch_dialog', next);
            }}
            label={t('settings.launch.showLaunchDialog')}
            style={{ marginBottom: 0 }}
          />
          <Switch
            checked={autoCloseOnLaunch}
            onChange={() => {
              const next = !autoCloseOnLaunch;
              setAutoCloseOnLaunch(next);
              updateSetting('auto_close_on_launch', next);
            }}
            label={t('settings.launch.autoClose')}
            style={{ marginBottom: 0 }}
          />
        </div>
      </div>
    </div>
  );
}

function SettingsContent() {
  const t = useTranslations();
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
          <Tab id="general" title={t('settings.tabs.general')} />
          <Tab id="paths" title={t('settings.tabs.paths')} />
          <Tab id="launch" title={t('settings.tabs.launch')} />
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
  const t = useTranslations();
  return (
    <ParchmentDialog
      isOpen={isOpen}
      onClose={onClose}
      title={t('settings.title')}
      width={680}
      minHeight={480}
      footerActions={
        <Button intent="primary" text={t('settings.done')} onClick={onClose} />
      }
    >
      <div style={{ margin: -16 }}>
        <SettingsContent />
      </div>
    </ParchmentDialog>
  );
}
