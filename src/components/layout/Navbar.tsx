import { useState, useCallback, useEffect } from 'react';
import {
  Button, Navbar as BPNavbar, NavbarGroup, NavbarDivider,
} from '@blueprintjs/core';
import { invoke } from '@tauri-apps/api/core';
import { T } from '../theme';
import { SettingsDialog } from '../Settings/SettingsPanel';
import { GameLaunchDialog } from '../shared';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { useTranslations } from '@/hooks/useTranslations';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import { useToast } from '@/contexts/ToastContext';
import { TauriAPI } from '@/lib/tauri-api';

interface NavbarProps {
  onBack: () => void;
}

export function Navbar({ onBack }: NavbarProps) {
  const t = useTranslations();
  const { handleError } = useErrorHandler();
  const { showToast } = useToast();
  const { character } = useCharacterContext();
  const [showSettings, setShowSettings] = useState(false);
  const [showGameLaunch, setShowGameLaunch] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isExporting, setIsExporting] = useState(false);

  const handleSave = useCallback(async () => {
    setIsSaving(true);
    try {
      await invoke('save_character', { filePath: null });
      showToast(t('actions.saveSuccess'), 'success');
      const config = await invoke<{ show_launch_dialog: boolean }>('get_app_config');
      if (config.show_launch_dialog) {
        setShowGameLaunch(true);
      }
    } catch (err) {
      handleError(err);
    } finally {
      setIsSaving(false);
    }
  }, [showToast, t, handleError]);

  const handleExport = async () => {
    setIsExporting(true);
    try {
      await invoke('export_to_localvault');
      showToast(t('actions.exportSuccess'), 'success');
    } catch (err) {
      handleError(err);
    } finally {
      setIsExporting(false);
    }
  };

  const handleLaunchGame = async (closeEditor: boolean) => {
    const config = await invoke<{ auto_close_on_launch: boolean }>('get_app_config');
    await TauriAPI.launchNWN2Game();
    if (closeEditor || config.auto_close_on_launch) {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().close();
    }
  };

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 's') {
        e.preventDefault();
        handleSave();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleSave]);

  return (
    <>
      <BPNavbar className="bp5-dark" style={{ background: T.navbar, paddingLeft: 12, paddingRight: 12, boxShadow: '0 1px 4px rgba(0,0,0,0.15)', position: 'relative', zIndex: 10 }}>
        <NavbarGroup align="left">
          <Button icon="cog" text={t('common.settings')} small minimal style={{ color: T.sidebarText }} onClick={() => setShowSettings(true)} />
          <Button icon="arrow-left" text={t('common.back')} small minimal style={{ color: T.sidebarText }} onClick={onBack} />
        </NavbarGroup>
        <NavbarGroup align="right">
          <Button icon="export" text={t('actions.exportCharacter')} small minimal loading={isExporting} style={{ color: T.sidebarText }} onClick={handleExport} />
          <Button icon="floppy-disk" text={isSaving ? t('actions.saving') : t('actions.save')} small minimal loading={isSaving} style={{ color: T.sidebarText }} onClick={handleSave} />
        </NavbarGroup>
      </BPNavbar>
      {showSettings && <SettingsDialog isOpen onClose={() => setShowSettings(false)} />}
      <GameLaunchDialog
        isOpen={showGameLaunch}
        onClose={() => setShowGameLaunch(false)}
        onLaunch={handleLaunchGame}
        saveName={character?.name}
      />
    </>
  );
}
