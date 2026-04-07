import { useState, useCallback, useEffect } from 'react';
import {
  Button, Navbar as BPNavbar, NavbarGroup, NavbarHeading, NavbarDivider,
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
  const classesSubsystem = useSubsystem('classes');
  const [showSettings, setShowSettings] = useState(false);
  const [showGameLaunch, setShowGameLaunch] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isExporting, setIsExporting] = useState(false);

  const classes = classesSubsystem.data?.entries ?? [];
  const classLabel = classes.map(c => c.name).join('/');

  const handleSave = useCallback(async () => {
    setIsSaving(true);
    try {
      await invoke('save_character', { filePath: null });
      showToast(t('actions.saveSuccess'), 'success');
      setShowGameLaunch(true);
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
    await TauriAPI.launchNWN2Game();
    if (closeEditor) {
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
          <NavbarHeading style={{ fontSize: 14, fontWeight: 700, marginRight: 8, color: T.sidebarAccent }}>NWN2 Save Editor</NavbarHeading>
          <NavbarDivider />
          <span style={{ fontSize: 13, color: '#e8e4dc' }}>{character?.name ?? ''}</span>
          {classLabel && (
            <span style={{ fontSize: 12, marginLeft: 8, color: T.sidebarText }}>
              Lvl {character?.level} {classLabel}
            </span>
          )}
        </NavbarGroup>
        <NavbarGroup align="right">
          <Button icon="floppy-disk" text={isSaving ? t('actions.saving') : t('actions.save')} small minimal loading={isSaving} style={{ color: T.sidebarText }} onClick={handleSave} />
          <Button icon="export" text={t('actions.exportCharacter')} small minimal loading={isExporting} style={{ color: T.sidebarText }} onClick={handleExport} />
          <Button icon="arrow-left" text={t('common.back')} small minimal style={{ color: T.sidebarText }} onClick={onBack} />
          <NavbarDivider />
          <Button icon="cog" small minimal style={{ color: T.sidebarText }} onClick={() => setShowSettings(true)} />
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
