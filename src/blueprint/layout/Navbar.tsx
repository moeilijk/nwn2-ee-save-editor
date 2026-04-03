import { useState } from 'react';
import {
  Button, Navbar as BPNavbar, NavbarGroup, NavbarHeading, NavbarDivider,
} from '@blueprintjs/core';
import { invoke } from '@tauri-apps/api/core';
import { T } from '../theme';
import { SettingsDialog } from '../Settings/SettingsPanel';
import { useCharacterContext, useSubsystem } from '@/contexts/CharacterContext';
import { useTranslations } from '@/hooks/useTranslations';
import { useErrorHandler } from '@/hooks/useErrorHandler';

export function Navbar() {
  const t = useTranslations();
  const { handleError } = useErrorHandler();
  const { character, clearCharacter } = useCharacterContext();
  const classesSubsystem = useSubsystem('classes');
  const [showSettings, setShowSettings] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isExporting, setIsExporting] = useState(false);

  const classes = classesSubsystem.data?.entries ?? [];
  const classLabel = classes.map(c => c.name).join('/');

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await invoke('save_character', { filePath: null });
    } catch (err) {
      handleError(err);
    } finally {
      setIsSaving(false);
    }
  };

  const handleExport = async () => {
    setIsExporting(true);
    try {
      await invoke('export_to_localvault');
    } catch (err) {
      handleError(err);
    } finally {
      setIsExporting(false);
    }
  };

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
          <Button icon="arrow-left" text={t('common.back')} small minimal style={{ color: T.sidebarText }} onClick={clearCharacter} />
          <NavbarDivider />
          <Button icon="cog" small minimal style={{ color: T.sidebarText }} onClick={() => setShowSettings(true)} />
        </NavbarGroup>
      </BPNavbar>
      {showSettings && <SettingsDialog isOpen onClose={() => setShowSettings(false)} />}
    </>
  );
}
