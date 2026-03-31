import { useState, useEffect, useCallback } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { useTauri } from '@/providers/TauriProvider';
import { SaveFile } from '@/lib/tauri-api';
import { SaveThumbnail } from './SaveThumbnail';
import { useCharacterContext } from '@/contexts/CharacterContext';
import { useSettings } from '@/contexts/SettingsContext';
import { GameLaunchDialog } from '../GameLaunchDialog';
import { CharacterAPI, type SaveCharacterOption } from '@/services/characterApi';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import FileBrowserModal from '@/components/FileBrowser/FileBrowserModal';

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

  const [saveMode, setSaveMode] = useState<'sp' | 'mp'>('sp');
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
  const [showCharacterPicker, setShowCharacterPicker] = useState(false);
  const [currentPath, setCurrentPath] = useState<string>('');
  const [backupPath, setBackupPath] = useState<string>('');
  const [backupRefreshKey, setBackupRefreshKey] = useState(0);
  const [pendingSaveFile, setPendingSaveFile] = useState<SaveFile | null>(null);
  const [saveCharacters, setSaveCharacters] = useState<SaveCharacterOption[]>([]);
  const [selectedPlayerIndex, setSelectedPlayerIndex] = useState<number | null>(null);
  const [resolvingSaveCharacters, setResolvingSaveCharacters] = useState(false);

  const isBusy = importing || characterLoading || resolvingSaveCharacters;

  const loadAvailableSaves = useCallback(async (mode: 'sp' | 'mp' = saveMode) => {
    // Rust-only implementation
    if (!api) return;

    setLoading(true);
    setError(null);
    try {
      const saves = await api.findNWN2Saves(mode);
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
  }, [api, saveMode]);

  const importSaveFile = useCallback(async (saveFile: SaveFile, playerIndex?: number) => {
    setImporting(true);
    setError(null);

    try {
      await importCharacter(saveFile.path, playerIndex);
      setSelectedFile(saveFile);
      setSelectedPlayerIndex(playerIndex ?? 0);
      setPendingSaveFile(null);
      setSaveCharacters([]);
      setShowCharacterPicker(false);
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

  const confirmCharacterSwitch = useCallback(async (
    saveFile: SaveFile,
    nextPlayer?: SaveCharacterOption,
  ) => {
    if (!character || !api) {
      return true;
    }

    const switchingSave = selectedFile?.path !== saveFile.path;
    const nextPlayerIndex = nextPlayer?.player_index ?? 0;
    const switchingPlayer = selectedPlayerIndex !== null && selectedPlayerIndex !== nextPlayerIndex;

    if (!switchingSave && !switchingPlayer) {
      return true;
    }

    return api.confirmSaveSwitch(
      character.name || selectedFile?.name || 'Current Character',
      nextPlayer?.name || saveFile.name,
    );
  }, [api, character, selectedFile, selectedPlayerIndex]);

  const beginImportFlow = useCallback(async (saveFile: SaveFile) => {
    setResolvingSaveCharacters(true);
    setError(null);

    try {
      const players = await CharacterAPI.listSaveCharacters(saveFile.path);
      if (players.length <= 1) {
        const selectedPlayer = players[0];
        const targetPlayerIndex = selectedPlayer?.player_index ?? 0;

        if (selectedFile?.path === saveFile.path && selectedPlayerIndex === targetPlayerIndex && character) {
          return;
        }

        const confirmed = await confirmCharacterSwitch(saveFile, selectedPlayer);
        if (!confirmed) {
          return;
        }

        await importSaveFile(saveFile, targetPlayerIndex);
        return;
      }

      setPendingSaveFile(saveFile);
      setSaveCharacters(players);
      setShowCharacterPicker(true);
    } catch (err) {
      console.error('Failed to inspect save characters:', err);
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError('Failed to inspect save characters.');
      }
    } finally {
      setResolvingSaveCharacters(false);
    }
  }, [character, confirmCharacterSwitch, importSaveFile, selectedFile?.path, selectedPlayerIndex]);

  const handleSelectSaveCharacter = useCallback(async (player: SaveCharacterOption) => {
    if (!pendingSaveFile) {
      return;
    }

    if (selectedFile?.path === pendingSaveFile.path && selectedPlayerIndex === player.player_index && character) {
      setShowCharacterPicker(false);
      setPendingSaveFile(null);
      setSaveCharacters([]);
      return;
    }

    const confirmed = await confirmCharacterSwitch(pendingSaveFile, player);
    if (!confirmed) {
      return;
    }

    await importSaveFile(pendingSaveFile, player.player_index);
  }, [character, confirmCharacterSwitch, importSaveFile, pendingSaveFile, selectedFile?.path, selectedPlayerIndex]);

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
    setSuccessMessage(null);
    setError(null);

    let backupsPath = '';
    if (selectedFile) {
      const pathParts = selectedFile.path.replace(/\\/g, '/').split('/');
      const saveName = pathParts[pathParts.length - 1] || pathParts[pathParts.length - 2];
      const savesDir = pathParts.slice(0, -1).join('/');
      backupsPath = `${savesDir}/backups/${saveName}`;
    } else {
      try {
        const defaultSavesPath = await invoke<string>('get_default_saves_path', { saveMode });
        backupsPath = defaultSavesPath.replace(/\\/g, '/') + '/backups';
      } catch {
        backupsPath = '';
      }
    }

    setBackupPath(backupsPath);
    setShowBackupBrowser(true);
  }, [saveMode, selectedFile]);

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

  const handleModeChange = (mode: 'sp' | 'mp') => {
    setSaveMode(mode);
    setSaves([]);
    setSelectedFile(null);
    setCurrentPath('');
    setAutoScanComplete(false);
    loadAvailableSaves(mode).finally(() => setAutoScanComplete(true));
  };

  useEffect(() => {
    if (isAvailable && api && !autoScanComplete) {
      loadAvailableSaves(saveMode).finally(() => {
        setAutoScanComplete(true);
      });
    }
  }, [isAvailable, api, autoScanComplete, loadAvailableSaves, saveMode]);

  useEffect(() => {
    (window as Window & { __openBackups?: () => void }).__openBackups = handleOpenBackupsFolder;
    return () => {
      delete (window as Window & { __openBackups?: () => void }).__openBackups;
    };
  }, [handleOpenBackupsFolder]);

  useEffect(() => {
    if (!character) {
      setSelectedFile(null);
      setSelectedPlayerIndex(null);
    }
  }, [character]);

  const handleSelectFile = async () => {
    if (!currentPath) {
      try {
        const defaultPath = await invoke<string>('get_default_saves_path', { saveMode });
        setCurrentPath(defaultPath);
      } catch {
        // ignore, FileBrowserModal will use its own default
      }
    }
    setShowFileBrowser(true);
  };

  const handleFileBrowserSelect = async (file: { path: string; name: string }) => {
    setShowFileBrowser(false);
    if (file.path) {
      const saveFile: SaveFile = {
        path: file.path,
        name: file.name,
      };
      await beginImportFlow(saveFile);
    }
  };

  const handleImportSelectedSave = async (save: SaveFile) => {
    await beginImportFlow(save);
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

      <div className="flex gap-1 mt-4 mb-3 rounded-lg overflow-hidden border border-[rgb(var(--color-surface-border))]">
        <button
          className={`flex-1 py-1.5 text-sm font-medium transition-colors ${saveMode === 'sp' ? 'bg-[rgb(var(--color-primary))] text-white' : 'bg-[rgb(var(--color-surface-1))] text-[rgb(var(--color-text-secondary))] hover:bg-[rgb(var(--color-surface-2))]'}`}
          onClick={() => handleModeChange('sp')}
          disabled={isBusy}
        >
          Single Player
        </button>
        <button
          className={`flex-1 py-1.5 text-sm font-medium transition-colors ${saveMode === 'mp' ? 'bg-[rgb(var(--color-primary))] text-white' : 'bg-[rgb(var(--color-surface-1))] text-[rgb(var(--color-text-secondary))] hover:bg-[rgb(var(--color-surface-2))]'}`}
          onClick={() => handleModeChange('mp')}
          disabled={isBusy}
        >
          Multiplayer
        </button>
      </div>

      <div className="flex gap-2 mb-6">
        <Button
          variant="outline"
          size="md"
          className="flex-1 text-sm h-10"
          onClick={handleSelectFile}
          disabled={isBusy}
        >
          {isBusy ? 'Loading...' : 'Browse...'}
        </Button>
      </div>

      {loading && !autoScanComplete ? (
        <div className="text-xs text-text-muted">Scanning for saves...</div>
      ) : saves.length > 0 ? (
        <Card variant="container">
          <div className="recent-saves-header">
            Last 3 {saveMode === 'mp' ? 'Multiplayer' : 'Single Player'} Saves
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
                    {save.character_name ? (
                        <div className="flex flex-col">
                            <span className="font-semibold text-[rgb(var(--color-primary))]">{save.character_name}</span>
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
      
      <FileBrowserModal
        isOpen={showFileBrowser}
        onClose={() => setShowFileBrowser(false)}
        mode="load-saves"
        onSelectFile={handleFileBrowserSelect}
        currentPath={currentPath}
        onPathChange={setCurrentPath}
      />

      <FileBrowserModal
        isOpen={showBackupBrowser}
        onClose={() => setShowBackupBrowser(false)}
        mode="manage-backups"
        currentPath={backupPath}
        onPathChange={setBackupPath}
        onDeleteBackup={handleDeleteBackup}
        canRestore={true}
        refreshKey={backupRefreshKey}
        onSelectFile={handleBackupSelect}
      />

      {showCharacterPicker && pendingSaveFile && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4">
          <div className="w-full max-w-2xl rounded-xl border border-[rgb(var(--color-surface-border))] bg-[rgb(var(--color-surface-1))] shadow-2xl">
            <div className="flex items-center justify-between border-b border-[rgb(var(--color-surface-border)/0.6)] px-5 py-4">
              <div>
                <div className="text-lg font-semibold text-[rgb(var(--color-text-primary))]">
                  Choose Character
                </div>
                <div className="text-sm text-[rgb(var(--color-text-secondary))]">
                  {pendingSaveFile.name} contains multiple player characters. Pick the one to edit.
                </div>
              </div>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => {
                  if (!importing) {
                    setShowCharacterPicker(false);
                    setPendingSaveFile(null);
                    setSaveCharacters([]);
                  }
                }}
                disabled={importing}
              >
                Close
              </Button>
            </div>

            <div className="max-h-[60vh] space-y-3 overflow-y-auto px-5 py-4">
              {saveCharacters.map(player => {
                const classSummary = player.classes.length > 0
                  ? player.classes.map(classEntry => `${classEntry.name} ${classEntry.level}`).join(' / ')
                  : `Level ${player.total_level}`;
                const isCurrentSelection =
                  selectedFile?.path === pendingSaveFile.path &&
                  selectedPlayerIndex === player.player_index;

                return (
                  <button
                    key={`${pendingSaveFile.path}-${player.player_index}`}
                    type="button"
                    onClick={() => handleSelectSaveCharacter(player)}
                    disabled={importing}
                    className={`w-full rounded-lg border px-4 py-3 text-left transition-colors ${
                      isCurrentSelection
                        ? 'border-[rgb(var(--color-primary))] bg-[rgb(var(--color-primary)/0.12)]'
                        : 'border-[rgb(var(--color-surface-border)/0.7)] bg-[rgb(var(--color-surface-2)/0.6)] hover:border-[rgb(var(--color-primary)/0.45)]'
                    }`}
                  >
                    <div className="flex items-start justify-between gap-4">
                      <div className="min-w-0">
                        <div className="font-medium text-[rgb(var(--color-text-primary))]">
                          {player.name}
                        </div>
                        <div className="mt-1 text-sm text-[rgb(var(--color-text-secondary))]">
                          {player.race}
                        </div>
                        <div className="mt-2 text-sm text-[rgb(var(--color-text-secondary))]">
                          {classSummary}
                        </div>
                      </div>
                      <div className="shrink-0 text-xs text-[rgb(var(--color-text-muted))]">
                        Slot {player.player_index + 1}
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
