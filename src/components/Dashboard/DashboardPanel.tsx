import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button, Spinner } from '@blueprintjs/core';
import { useTranslations } from '@/hooks/useTranslations';
import { useCharacterContext } from '@/contexts/CharacterContext';
import { useErrorHandler } from '@/hooks/useErrorHandler';
import { useToast } from '@/contexts/ToastContext';
import { TauriAPI, type SaveFile } from '@/lib/tauri-api';
import { T, PATTERN_BG } from '../theme';
import '../blueprint.css';
import { SaveList } from './SaveList';
import type { SaveEntryData } from './SaveEntry';
import { FileBrowserDialog } from './FileBrowserDialog';
import type { FileInfo } from './FileBrowserDialog';
import { SettingsDialog } from '../Settings/SettingsPanel';
import { BackupAPI } from '@/services/backupApi';

export default function DashboardPanel() {
  const t = useTranslations();
  const { importCharacter, refreshAll } = useCharacterContext();
  const { handleError } = useErrorHandler();
  const { showToast } = useToast();

  const [saves, setSaves] = useState<SaveEntryData[]>([]);
  const [savePaths, setSavePaths] = useState<string[]>([]);
  const [isLoadingSaves, setIsLoadingSaves] = useState(true);
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [isImporting, setIsImporting] = useState(false);

  const [showVaultBrowser, setShowVaultBrowser] = useState(false);
  const [showBackupBrowser, setShowBackupBrowser] = useState(false);
  const [backupPath, setBackupPath] = useState('');
  const [backupRefreshKey, setBackupRefreshKey] = useState(0);
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    let cancelled = false;

    async function loadSaves() {
      setIsLoadingSaves(true);
      try {
        const result: SaveFile[] = await TauriAPI.findNWN2Saves();
        if (cancelled) return;

        const paths: string[] = [];
        const entries: SaveEntryData[] = result.map(save => {
          paths.push(save.path);
          return {
            characterName: save.character_name || save.name,
            folderName: save.name,
            date: save.modified
              ? new Date(save.modified * 1000).toLocaleString()
              : '',
            thumbnail: null,
            isActive: false,
          };
        });

        setSaves(entries);
        setSavePaths(paths);

        // Load thumbnails in background
        result.forEach((save, i) => {
          if (save.thumbnail) {
            TauriAPI.getSaveThumbnail(save.thumbnail).then(base64 => {
              if (!cancelled) {
                setSaves(prev => prev.map((s, j) =>
                  j === i ? { ...s, thumbnail: base64 } : s
                ));
              }
            }).catch(() => { /* thumbnail failed, keep placeholder */ });
          }
        });
      } catch (err) {
        if (!cancelled) handleError(err);
      } finally {
        if (!cancelled) setIsLoadingSaves(false);
      }
    }

    loadSaves();
    return () => { cancelled = true; };
  }, [handleError]);

  const handleOpenSave = async () => {
    if (selectedIndex === null || isImporting) return;
    const path = savePaths[selectedIndex];
    if (!path) return;

    setIsImporting(true);
    try {
      await importCharacter(path);
    } catch (err) {
      handleError(err);
    } finally {
      setIsImporting(false);
    }
  };

  const handleSelectAndOpen = async (index: number) => {
    setSelectedIndex(index);
    const path = savePaths[index];
    if (!path) return;

    setIsImporting(true);
    try {
      await importCharacter(path);
    } catch (err) {
      handleError(err);
    } finally {
      setIsImporting(false);
    }
  };

  const handleImportVaultFile = async (file: FileInfo) => {
    setShowVaultBrowser(false);
    setIsImporting(true);
    try {
      await importCharacter(file.path);
    } catch (err) {
      handleError(err);
    } finally {
      setIsImporting(false);
    }
  };

  const handleBrowseFile = async (file: FileInfo) => {
    setIsImporting(true);
    try {
      await importCharacter(file.path);
    } catch (err) {
      handleError(err);
    } finally {
      setIsImporting(false);
    }
  };

  return (
    <div
      className="bp-app"
      style={{
        height: '100vh',
        background: T.sidebar,
        padding: 32,
        display: 'flex',
      }}
    >
      <div style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        borderRadius: 8,
        overflow: 'hidden',
        background: T.bg,
        backgroundImage: PATTERN_BG,
        backgroundSize: '200px 200px',
        boxShadow: '0 4px 24px rgba(0, 0, 0, 0.3)',
      }}>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'flex-end',
          padding: '10px 24px',
          borderBottom: `1px solid ${T.borderLight}`,
        }}>
          <div style={{ display: 'flex', gap: 4 }}>
            {selectedIndex !== null && (
              <Button
                small
                intent="primary"
                icon="folder-open"
                loading={isImporting}
                onClick={handleOpenSave}
              >
                {t('dashboard.openSave')}
              </Button>
            )}
            <Button minimal small icon="import" onClick={() => setShowVaultBrowser(true)}>
              {t('actions.importCharacter')}
            </Button>
            <Button
              minimal
              small
              icon="folder-open"
              onClick={async () => {
                try {
                  const config = await TauriAPI.getPathsConfig();
                  const docsPath = config?.documents_folder?.path;
                  if (docsPath) {
                    await TauriAPI.openFolderInExplorer(docsPath);
                  }
                } catch (err) { handleError(err); }
              }}
            >
              {t('actions.openDocumentsFolder')}
            </Button>
            <Button
              minimal
              small
              icon="history"
              onClick={async () => {
                try {
                  const path = await invoke<string>('get_default_backups_path');
                  if (path) {
                    setBackupPath(path);
                  }
                } catch { /* open dialog anyway */ }
                setShowBackupBrowser(true);
              }}
            >
              {t('actions.manageBackups')}
            </Button>
            <Button minimal small icon="cog" onClick={() => setShowSettings(true)}>
              {t('navigation.settings')}
            </Button>
          </div>
        </div>

        <div style={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
          {isLoadingSaves ? (
            <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
              <Spinner size={30} />
            </div>
          ) : (
            <SaveList
              saves={saves}
              selectedIndex={selectedIndex}
              onSelect={setSelectedIndex}
              onDoubleClick={handleSelectAndOpen}
              onBrowseFile={handleBrowseFile}
            />
          )}
        </div>
      </div>

      <FileBrowserDialog
        isOpen={showVaultBrowser}
        onClose={() => setShowVaultBrowser(false)}
        mode="load-vault"
        onSelectFile={handleImportVaultFile}
      />

      <FileBrowserDialog
        isOpen={showBackupBrowser}
        onClose={() => setShowBackupBrowser(false)}
        mode="manage-backups"
        currentPath={backupPath}
        onPathChange={setBackupPath}
        refreshKey={backupRefreshKey}
        canRestore
        onSelectFile={async (file) => {
          try {
            await BackupAPI.restoreFromBackup(0, {
              backup_path: file.path,
              confirm_restore: true,
              create_pre_restore_backup: true,
            });
            setShowBackupBrowser(false);
            showToast(t('fileBrowser.restoreSuccess'), 'success');
            await refreshAll();
          } catch (error) {
            handleError(error);
          }
        }}
        onDeleteBackup={async (file) => {
          try {
            await BackupAPI.deleteBackup(file.path);
            showToast(t('fileBrowser.backupDeleted'), 'success');
            setBackupRefreshKey(prev => prev + 1);
          } catch (error) {
            handleError(error);
          }
        }}
      />

      {showSettings && <SettingsDialog isOpen onClose={() => setShowSettings(false)} />}
    </div>
  );
}
