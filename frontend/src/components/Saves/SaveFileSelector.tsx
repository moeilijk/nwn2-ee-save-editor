import { useState, useEffect, useCallback } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { useTauri } from '@/providers/TauriProvider';
import { SaveFile } from '@/lib/tauri-api';
import { SaveThumbnail } from './SaveThumbnail';
import { useCharacterContext } from '@/contexts/CharacterContext';
import { useSettings } from '@/contexts/SettingsContext';
import { GameLaunchDialog } from '../GameLaunchDialog';
import { CharacterAPI } from '@/services/characterApi';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import FileBrowserModal from '@/components/FileBrowser/FileBrowserModal';

interface BackupInfo {
  path: string;
  timestamp: string;
  size_bytes: number;
  created_at: number;
}

interface RestoreResult {
  success: boolean;
  pre_restore_backup: string | null;
  files_restored: number;
  message: string;
}


export function SaveFileSelector() {
  const { isAvailable, isLoading, api } = useTauri();
  const { importCharacter, character, isLoading: characterLoading } = useCharacterContext();
  const { gameSettings } = useSettings();
  

  
  interface ExtendedSaveFile extends SaveFile {
    character?: string;
  }

  const [selectedFile, setSelectedFile] = useState<SaveFile | null>(null);
  const [saves, setSaves] = useState<ExtendedSaveFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [importing, setImporting] = useState(false);
  const [, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [autoScanComplete, setAutoScanComplete] = useState(false);
  const [showLaunchDialog, setShowLaunchDialog] = useState(false);
  const [showFileBrowser, setShowFileBrowser] = useState(false);
  const [showBackupBrowser, setShowBackupBrowser] = useState(false);
  const [currentPath, setCurrentPath] = useState<string>('');
  const [backupPath, setBackupPath] = useState<string>('');
  const [backupRefreshKey, setBackupRefreshKey] = useState(0);

  const loadAvailableSaves = useCallback(async () => {
    // Rust-only implementation
    if (!api) return;
    
    setLoading(true);
    setError(null);
    try {
      const saves = await api.findNWN2Saves();
      if (saves) {
          const mapped: ExtendedSaveFile[] = saves.map(s => ({
              ...s,
              is_directory: true, // Assuming all returned by findNWN2Saves are valid saves
              character: undefined, // properties logic not available in simple scan
              thumbnail: s.thumbnail || ''
          }));
          setSaves(mapped);
      }
    } catch (err) {
      console.error('❌ SaveFileSelector: Failed to find saves:', err);
      const errorMessage = typeof err === 'string' ? err : 'An unknown error occurred while finding save files.';
      setError(`Failed to find save files. Please check if NWN2 save directory exists. Details: ${errorMessage}`);
    } finally {
      setLoading(false);
    }
  }, [api]);

  const importSaveFile = useCallback(async (saveFile: SaveFile) => {
    setImporting(true);
    setError(null);

    try {
      await importCharacter(saveFile.path);
      setError(null);
    } catch (err) {
      console.error('Failed to import save:', err);
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError('Failed to import save file. Please check the file and try again.');
      }
    } finally {
      setImporting(false);
    }
  }, [importCharacter]);

  const _saveCharacter = useCallback(async () => {
    if (!character?.id) {
      setError('No character loaded to save');
      return;
    }

    setSaving(true);
    setError(null);

    try {
      await CharacterAPI.saveCharacter(character.id, { sync_current_state: true });
      
      // Show launch dialog after successful save (if enabled in settings)
      if (gameSettings.show_launch_dialog) {
        setShowLaunchDialog(true);
      }
    } catch (err) {
      console.error('Failed to save character:', err);
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError('Failed to save character. Please try again.');
      }
    } finally {
      setSaving(false);
    }
  }, [character, gameSettings.show_launch_dialog]);

  const handleGameLaunch = useCallback(async (closeEditor: boolean) => {
    if (!api) {
      setError('Cannot launch game: Tauri API not available');
      return;
    }

    try {
      await api.launchNWN2Game(gameSettings.nwn2_installation_path);
      setShowLaunchDialog(false);
      if (closeEditor) {
        await getCurrentWindow().close();
      }
    } catch (err) {
      console.error('Failed to launch game:', err);
      setError(err instanceof Error ? err.message : 'Failed to launch NWN2:EE');
      setShowLaunchDialog(false);
    }
  }, [api, gameSettings.nwn2_installation_path]);

  const handleOpenBackupsFolder = useCallback(async () => {
    if (!api) return;
    try {
      const backups = await invoke<BackupInfo[]>('list_backups');
      if (backups.length > 0) {
        const backupDir = backups[0].path.split(/[/\\]/).slice(0, -1).join('/');
        await api.openFolderInExplorer(backupDir);
      } else {
        setError('No backups found');
      }
    } catch (err) {
      console.error('Failed to open backups folder:', err);
      setError(err instanceof Error ? err.message : 'Failed to open backups folder');
    }
  }, [api]);

  const handleBackupSelect = useCallback(async (file: { path: string; name: string }) => {
    try {
      const result = await invoke<RestoreResult>('restore_backup', {
        backupPath: file.path,
        createPreRestoreBackup: true
      });
      if (result.success) {
        setSuccessMessage(`Restored backup: ${result.message}`);
        setShowBackupBrowser(false);
        await loadAvailableSaves();
      } else {
        setError('Restore failed');
      }
    } catch (err) {
      console.error('Failed to restore backup:', err);
      setError(err instanceof Error ? err.message : 'Failed to restore backup');
    }
  }, [loadAvailableSaves]);

  const handleDeleteBackup = useCallback(async (file: { path: string; name: string }) => {
    try {
      await invoke<boolean>('delete_backup', { backupPath: file.path });
      setBackupRefreshKey(prev => prev + 1);
      setSuccessMessage('Backup deleted');
    } catch (err) {
      console.error('Failed to delete backup:', err);
      setError(err instanceof Error ? err.message : 'Failed to delete backup');
    }
  }, []);

  useEffect(() => {
    if (isAvailable && api && !autoScanComplete) {
      loadAvailableSaves().finally(() => {
        setAutoScanComplete(true);
      });
    }
  }, [isAvailable, api, autoScanComplete, loadAvailableSaves]);

  // Window backup hook disabled/removed
  // useEffect(() => {
  //   (window as Window & { __openBackups?: () => void }).__openBackups = handleOpenBackupsFolder;
  //   return () => {
  //     delete (window as Window & { __openBackups?: () => void }).__openBackups;
  //   };
  // }, [handleOpenBackupsFolder]);

  const handleSelectFile = async () => {
    if (api) {
       // Use selectSaveFile for folder selection (Standard NWN2 saves)
       // This returns a SaveFile object directly
       try {
         const saveFile = await api.selectSaveFile();
         if (saveFile) {
             await importSaveFile(saveFile);
         }
       } catch (err) {
         console.error('Failed to select save file:', err);
         // Error is already logged in rust, just show partial error if needed or rely on save selector UI state
       }
    }
  };

  const handleFileBrowserSelect = async (file: { path: string; name: string }) => {
      // Not using FileBrowserModal currently
  };

  const handleImportSelectedSave = async (save: SaveFile) => {
    if (selectedFile?.path === save.path && character) {
      return;
    }
    
    if (selectedFile && selectedFile.path !== save.path && character && api) {
      const confirmed = await api.confirmSaveSwitch(selectedFile.name, save.name);
      if (!confirmed) {
        return; 
      }
    }
    
    setSelectedFile(save);
    await importSaveFile(save);
  };

  if (isLoading) {
    return <div className="text-sm text-text-muted">Initializing...</div>;
  }

  if (!isAvailable) {
    return <div className="text-sm text-error">Desktop mode unavailable</div>;
  }

  return (
    <div className="space-y-3">
      {error && (
        <div className="p-2 bg-surface-1 text-error rounded text-sm">
          {error}
        </div>
      )}

      {successMessage && (
        <div className="p-2 bg-surface-1 text-success rounded text-sm">
          {successMessage}
        </div>
      )}

      {character && (
        <div className="p-2 bg-surface-1 text-success rounded text-sm">
          Loaded: {character.name}
        </div>
      )}

      <div className="flex gap-2 mt-4 mb-6">
        <Button
          variant="outline"
          size="md"
          className="flex-1 text-sm h-10"
          onClick={handleSelectFile}
          disabled={importing || characterLoading}
        >
          {importing ? 'Loading...' : 'Browse...'}
        </Button>
      </div>

      {loading && !autoScanComplete ? (
        <div className="text-xs text-text-muted">Scanning for saves...</div>
      ) : saves.length > 0 ? (
        <Card variant="container">
          <div className="recent-saves-header">
            Last 3 Save Games
          </div>
          <div className="space-y-2 max-h-[600px] overflow-y-auto">
            {saves.map((save, index) => (
              <Card
                key={index}
                variant="interactive"
                selected={selectedFile?.path === save.path}
                onClick={() => handleImportSelectedSave(save)}
                className="cursor-pointer flex items-center gap-4 p-3"
              >
                <SaveThumbnail 
                  thumbnailPath={save.thumbnail} 
                  size="lg" 
                  className="shrink-0"
                />
                <div className="flex flex-col flex-1 min-w-0">
                  <div className="recent-save-name truncate">
                    {save.character ? (
                        <div className="flex flex-col">
                            <span className="font-semibold text-[rgb(var(--color-primary))]">{save.character}</span>
                            <span className="text-xs text-[rgb(var(--color-text-muted))] opacity-75">{save.name}</span>
                        </div>
                    ) : save.name}
                  </div>
                  {save.modified && (
                    <div className="text-xs text-[rgb(var(--color-text-muted))] mt-0.5">
                       {new Date(save.modified).toLocaleString()}
                    </div>
                  )}
                  <div className="recent-save-action mt-1">
                    {selectedFile?.path === save.path ? 'Loaded' : 'Click to load'}
                  </div>
                </div>
              </Card>
            ))}
          </div>
        </Card>
      ) : autoScanComplete ? (
        <Card variant="container">
          <div className="recent-saves-header">Saved Games</div>
          <div className="text-xs text-text-muted text-center py-4">
            No saves found automatically. Use the Browse button above.
          </div>
        </Card>
      ) : null}

      <GameLaunchDialog
        isOpen={showLaunchDialog}
        onClose={() => setShowLaunchDialog(false)}
        onLaunch={handleGameLaunch}
        saveName={character?.name}
        gamePathDetected={!!gameSettings.nwn2_installation_path}
      />
      
      {/* FileBrowserModal/BackupModal disabled for now as they relied on legacy API */}
    </div>
  );
}
