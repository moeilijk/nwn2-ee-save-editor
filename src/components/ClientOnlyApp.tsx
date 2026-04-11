import { useState, useEffect, useCallback } from 'react';
import { useTranslations } from '@/hooks/useTranslations';
import { useTauri } from '@/providers/TauriProvider';
import { TauriAPI } from '@/lib/tauri-api';

import CustomTitleBar from '@/components/ui/CustomTitleBar';
import Sidebar from '@/components/ui/Sidebar';
import { GameLaunchDialog } from '@/components/GameLaunchDialog';
import { getCurrentWindow } from '@tauri-apps/api/window';
import AbilityScoresEditor from '@/components/AbilityScores/AbilityScoresEditor';
import AppearanceEditor from '@/components/Appearance/AppearanceEditor';
import ClassAndLevelsEditor from '@/components/ClassesLevel/ClassAndLevelsEditor';
import InventoryEditor from '@/components/Inventory/InventoryEditor';
import SkillsEditor from '@/components/Skills/SkillsEditor';
import FeatsEditor from '@/components/Feats/FeatsEditor';
import SpellsEditor from '@/components/Spells/SpellsEditor';
import CharacterOverview from '@/components/Overview/CharacterOverview';
import CompanionsView from '@/components/Companions/CompanionsView';
import CharacterBuilder from '@/components/CharacterBuilder';
import GameStateEditor from '@/components/GameState/GameStateEditor';
import SettingsPage from '@/pages/SettingsPage';

import TauriInitializer from '@/components/TauriInitializer';
import { CharacterProvider, useCharacterContext } from '@/contexts/CharacterContext';
import { IconCacheProvider } from '@/contexts/IconCacheContext';
import Dashboard from '@/components/Dashboard';
import EditorHeader from '@/components/EditorHeader';
import { CharacterAPI } from '@/services/characterApi';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { pathService, type PathConfig } from '@/lib/api/paths';

import LevelHelperModal from '@/components/ClassesLevel/LevelHelperModal';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import FileBrowserModal from '@/components/FileBrowser/FileBrowserModal';
import { invoke } from '@tauri-apps/api/core';

function AppContent() {
  const t = useTranslations();
  const { isAvailable, isLoading, api } = useTauri();
  const { character, isLoading: characterLoading, loadSubsystem } = useCharacterContext();
  const { clearCharacter, characterId, importCharacter, loadCharacter } = useCharacterContext();

  const [activeTab, setActiveTab] = useState('overview');
  const [viewMode, setViewMode] = useState<'dashboard' | 'editor'>('dashboard');

  const [currentCompanion, setCurrentCompanion] = useState<{
    name: string;
    portrait?: string;
    isCompanion: boolean;
  } | null>(null);

  // Level Helper State - just controls visibility, data is fetched dynamically by the modal
  const [showLevelHelper, setShowLevelHelper] = useState(false);

  // When a level is gained, show the helper modal
  // The modal itself will query subsystems for current available points
  const handleLevelGains = useCallback(() => {
    setShowLevelHelper(true);
  }, []);
  const [appReady, setAppReady] = useState(false);
  const [initProgress, setInitProgress] = useState({
    step: 'initializing',
    progress: 0,
    message: 'Starting up...'
  });
  const [showSettings, setShowSettings] = useState(false);
  const [startupPhase, setStartupPhase] = useState<'bootstrap' | 'choose-path-mode' | 'manual-setup' | 'initializing' | 'ready'>('bootstrap');
  const [startupPaths, setStartupPaths] = useState<PathConfig | null>(null);
  const [startupError, setStartupError] = useState<string | null>(null);
  const [startupBusy, setStartupBusy] = useState(false);

  const [showLoadingOverlay, setShowLoadingOverlay] = useState(false);
  const [showLaunchDialog, setShowLaunchDialog] = useState(false);
  const [showVaultBrowser, setShowVaultBrowser] = useState(false);
  const [vaultPath, setVaultPath] = useState('');

  // Sync viewMode when character ID changes (new load)
  useEffect(() => {
    console.log('[App] Character ID changed:', character?.id);
    if (character?.id) {
      console.log('[App] Switching to editor view');
      setViewMode('editor');
    }
  }, [character?.id]);

  const [feedback, setFeedback] = useState<{type: 'success' | 'error', message: string} | null>(null);

  useEffect(() => {
    setShowLoadingOverlay(characterLoading);
  }, [characterLoading]);

  // Create the current character object for the sidebar
  const currentCharacter = currentCompanion || (character ? {
    name: character.name || 'Unknown Character',
    portrait: character.portrait,
    customPortrait: character.customPortrait,
    isCompanion: false
  } : null);

  const characterBuilderKey = character
    ? [
        characterId ?? 'no-id',
        character.name,
        character.race,
        character.subrace ?? '',
        character.background?.name ?? '',
        character.level,
        character.classes.map(cls => `${cls.name}:${cls.level}`).join('|'),
      ].join('::')
    : 'no-character';

  // Mock function to simulate loading a companion
  const handleLoadCompanion = (companionName: string) => {
    setCurrentCompanion({
      name: companionName,
      isCompanion: true
    });
  };

  const handleBackToMain = () => {
    setCurrentCompanion(null);
  };

  const handleEditorBack = () => {
    // Minimize to dashboard instead of clearing
    setViewMode('dashboard');
    setActiveTab('overview'); // Reset tab optionally, or keep it
  };

  const handleContinueEditing = () => {
    setViewMode('editor');
  };

  const handleCloseSession = () => {
    clearCharacter();
    setViewMode('dashboard');
    setActiveTab('overview');
  };
  
  const handleCloseSettings = () => {
    setShowSettings(false);
  };

  const beginBackendInitialization = useCallback(() => {
    setStartupPhase('initializing');
    setStartupError(null);

    TauriAPI.initializeGameData().catch(err => {
      console.error('Failed to initialize backend:', err);
      setStartupError('Failed to initialize game data. Check your configured NWN2 paths.');
      setStartupPhase('manual-setup');
    });

    const checkStatus = async () => {
      try {
        const status = await TauriAPI.getInitializationStatus();

        setInitProgress({
          step: status.step,
          progress: status.progress,
          message: status.message
        });

        if (status.step === 'ready') {
          setAppReady(true);
          setStartupPhase('ready');
          return;
        }

        setTimeout(checkStatus, 500);
      } catch (err) {
        console.error('Failed to read initialization status:', err);
        setStartupError('Failed to read initialization progress.');
        setStartupPhase('manual-setup');
      }
    };

    checkStatus();
  }, []);

  const refreshStartupPaths = useCallback(async () => {
    const response = await pathService.getConfig();
    setStartupPaths(response.paths);
    return response.paths;
  }, []);

  const hasGameInstallPath = useCallback((paths: PathConfig) => {
    return Boolean(paths.game_folder.path && paths.game_folder.exists);
  }, []);

  const hasDocumentsPath = useCallback((paths: PathConfig) => {
    return Boolean(paths.documents_folder.path && paths.documents_folder.exists);
  }, []);

  const hasLocalVaultPath = useCallback((paths: PathConfig) => {
    return Boolean(paths.localvault_folder.path && paths.localvault_folder.exists);
  }, []);

  const missingRequiredPaths = useCallback((paths: PathConfig) => {
    const missing: string[] = [];
    if (!hasGameInstallPath(paths)) {
      missing.push('game installation folder');
    }
    if (!hasDocumentsPath(paths)) {
      missing.push('documents folder');
    }
    return missing;
  }, [hasDocumentsPath, hasGameInstallPath]);

  const formatMissingPathError = useCallback((paths: PathConfig, prefix: string) => {
    const missing = missingRequiredPaths(paths);
    if (missing.length === 0) {
      return null;
    }
    return `${prefix} Missing required path(s): ${missing.join(', ')}.`;
  }, [missingRequiredPaths]);

  const autoDiscoverMissingStartupPaths = useCallback(async (paths: PathConfig) => {
    if (paths.setup_mode !== 'auto') {
      return paths;
    }

    if (missingRequiredPaths(paths).length === 0) {
      return paths;
    }

    const response = await pathService.autoDetect();
    setStartupPaths(response.current_paths);
    return response.current_paths;
  }, [missingRequiredPaths]);

  const handleStartAutoSetup = useCallback(async () => {
    try {
      setStartupBusy(true);
      setStartupError(null);
      const response = await pathService.autoDetect();
      setStartupPaths(response.current_paths);

      const missingError = formatMissingPathError(
        response.current_paths,
        'Auto-discovery did not resolve all required NWN2 folders.'
      );

      if (!missingError) {
        beginBackendInitialization();
      } else {
        setStartupError(missingError);
        setStartupPhase('manual-setup');
      }
    } catch (err) {
      console.error('Failed to auto-discover NWN2 paths:', err);
      setStartupError('Auto-discovery failed. Set the game and documents paths manually.');
      setStartupPhase('manual-setup');
    } finally {
      setStartupBusy(false);
    }
  }, [beginBackendInitialization, formatMissingPathError]);

  const handleStartManualSetup = useCallback(async () => {
    try {
      setStartupBusy(true);
      setStartupError(null);
      const response = await pathService.setSetupMode('manual');
      setStartupPaths(response.paths);
      setStartupPhase('manual-setup');
    } catch (err) {
      console.error('Failed to switch to manual path setup:', err);
      setStartupError('Failed to switch to manual setup.');
    } finally {
      setStartupBusy(false);
    }
  }, []);

  const handleContinueAfterManualSetup = useCallback(async () => {
    try {
      setStartupBusy(true);
      const paths = await refreshStartupPaths();
      const resolvedPaths = await autoDiscoverMissingStartupPaths(paths);
      const missingError = formatMissingPathError(
        resolvedPaths,
        'Set all required NWN2 folders before continuing.'
      );
      if (!missingError) {
        beginBackendInitialization();
        return;
      }

      setStartupError(missingError);
    } catch (err) {
      console.error('Failed to refresh NWN2 paths:', err);
      setStartupError('Failed to validate your configured paths.');
    } finally {
      setStartupBusy(false);
    }
  }, [autoDiscoverMissingStartupPaths, beginBackendInitialization, formatMissingPathError, refreshStartupPaths]);

  const handleSaveCharacter = async () => {
    if (!characterId) return;
    try {
      await CharacterAPI.saveCharacter(characterId);
      // Refresh to clear dirty state
      await loadCharacter(characterId);
      
      setShowLaunchDialog(true);
    } catch (error) {
       console.error('Failed to save character', error);
       setFeedback({ type: 'error', message: error instanceof Error ? error.message : 'Failed to save character' });
       setTimeout(() => setFeedback(null), 5000);
    }
  };

  const handleLaunchGame = async (closeEditor: boolean) => {
    try {
      await api?.launchNWN2Game();
      if (closeEditor) {
        try {
           await getCurrentWindow().close();
        } catch (e) {
           console.error("Failed to close window", e);
        }
      }
      setShowLaunchDialog(false);
    } catch (error) {
      console.error("Failed to launch game", error);
      setFeedback({ type: 'error', message: "Failed to launch game" });
    }
  };

  const handleOpenBackups = () => {
    (window as Window & { __openBackups?: () => void }).__openBackups?.();
  };

  const handleOpenFolder = async () => {
      try {
        const config = await api?.getPathsConfig();
        // Prefer documents folder where saves usually are
        const savePath = config?.documents_folder?.path;
        
        if (savePath) {
           await api?.openFolderInExplorer(savePath);
        } else {
             console.warn("No documents folder configured.");
             // Fallback to trying to find saves if config fails
             const saves = await api?.findNWN2Saves();
             if (saves && saves.length > 0) {
               const firstSavePath = saves[0].path;
               const separator = firstSavePath.includes('\\') ? '\\' : '/';
               const lastSeparatorIndex = firstSavePath.lastIndexOf(separator);
               const folderPath = firstSavePath.substring(0, lastSeparatorIndex);
               await api?.openFolderInExplorer(folderPath);
             }
        }
      } catch (err) {
        console.error("Failed to open saves folder:", err);
      }
  };

  const handleImportCharacter = async () => {
    setShowVaultBrowser(true);
  };

  const handleVaultSelect = async (file: { path: string; name: string }) => {
    try {
      setShowVaultBrowser(false);
      await importCharacter(file.path);
    } catch (error) {
       console.error("Failed to import character from vault:", error);
    }
  };

  const handleExportToVault = async () => {
    try {
      const exportedPath = await invoke<string>('export_to_localvault');
      setFeedback({ type: 'success', message: `Character exported to: ${exportedPath}` });
      setTimeout(() => setFeedback(null), 5000);
    } catch (error) {
       console.error("Failed to export to vault:", error);
       setFeedback({ type: 'error', message: error instanceof Error ? error.message : 'Failed to export character' });
       setTimeout(() => setFeedback(null), 5000);
    }
  };

  const handleSettings = () => {
      setShowSettings(true);
  };

  const handleTabChange = async (tabId: string) => {
    setActiveTab(tabId);
    
    // Fetch fresh data for subsystem-related tabs
    if (character?.id) {
      try {
        switch (tabId) {
          case 'skills':
            await loadSubsystem('skills');
            break;
          case 'classes':
            await loadSubsystem('classes');
            break;
          case 'abilityScores':
            await loadSubsystem('abilityScores');
            break;
          case 'feats':
            await loadSubsystem('feats');
            break;
          case 'combat':
            await loadSubsystem('combat');
            break;
          case 'saves':
            await loadSubsystem('saves');
            break;
          case 'spells':
            await loadSubsystem('spells');
            break;
          case 'inventory':
            await loadSubsystem('inventory');
            break;
          case 'gameState':
            break;
          case 'overview':
            await loadSubsystem('abilityScores');
            await loadSubsystem('combat');
            await loadSubsystem('skills');
            await loadSubsystem('feats');
            await loadSubsystem('saves');
            await loadSubsystem('classes');
            break;
          default:
            break;
        }
      } catch (err) {
        console.error(`Failed to fetch data for ${tabId}:`, err);
      }
    }
  };


  useEffect(() => {
    const bootstrap = async () => {
      try {
        invoke('show_main_window').catch(() => {});

        const initialPaths = await refreshStartupPaths();
        if (initialPaths.needs_initial_setup) {
          setStartupPhase('choose-path-mode');
          return;
        }

        const resolvedPaths = await autoDiscoverMissingStartupPaths(initialPaths);

        const missingError = formatMissingPathError(
          resolvedPaths,
          'Missing required NWN2 folders.'
        );

        if (!missingError) {
          beginBackendInitialization();
          return;
        }

        setStartupError(missingError);
        setStartupPhase('manual-setup');
      } catch (err) {
        console.error('Failed to bootstrap startup path setup:', err);
        setStartupError('Failed to load the NWN2 path configuration.');
        setStartupPhase('manual-setup');
      }
    };

    bootstrap();
  }, [autoDiscoverMissingStartupPaths, beginBackendInitialization, formatMissingPathError, refreshStartupPaths]);

  if (startupPhase === 'choose-path-mode') {
    return (
      <div className="h-screen flex flex-col overflow-hidden bg-[rgb(var(--color-background))]">
        <CustomTitleBar />
        <div className="flex-1 flex items-center justify-center p-6">
          <Card className="w-full max-w-3xl">
            <CardHeader className="space-y-3">
              <CardTitle className="text-3xl">Choose How To Configure NWN2 Folders</CardTitle>
              <CardDescription>
                On first start, decide whether the editor should keep auto-discovering missing NWN2 folders or wait for you to set them manually. Game and documents paths are required.
              </CardDescription>
            </CardHeader>
            <CardContent className="grid gap-4 md:grid-cols-2">
              <Card variant="container" className="flex flex-col justify-between gap-4">
                <div className="space-y-2">
                  <h3 className="text-xl font-semibold text-[rgb(var(--color-text-primary))]">Auto-Discover</h3>
                  <p className="text-sm text-[rgb(var(--color-text-secondary))]">
                    Detect required game and documents paths now, and retry discovery later if one goes missing. You can still override individual paths in Settings.
                  </p>
                </div>
                <Button onClick={handleStartAutoSetup} loading={startupBusy}>
                  Start With Auto-Discover
                </Button>
              </Card>

              <Card variant="container" className="flex flex-col justify-between gap-4">
                <div className="space-y-2">
                  <h3 className="text-xl font-semibold text-[rgb(var(--color-text-primary))]">Manual Setup</h3>
                  <p className="text-sm text-[rgb(var(--color-text-secondary))]">
                    Skip discovery and choose the NWN2 folders yourself before loading game data.
                  </p>
                </div>
                <Button onClick={handleStartManualSetup} variant="outline" loading={startupBusy}>
                  Configure Manually
                </Button>
              </Card>
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

  if (startupPhase === 'manual-setup') {
    return (
      <div className="h-screen flex flex-col overflow-hidden bg-[rgb(var(--color-background))]">
        <CustomTitleBar />
        <div className="flex-1 overflow-y-auto">
          <div className="max-w-5xl mx-auto w-full p-6 md:p-8 space-y-6">
            <Card>
              <CardHeader className="space-y-3">
                <CardTitle className="text-3xl">Manual NWN2 Path Setup</CardTitle>
                <CardDescription>
                    Set the required NWN2 folders to continue: game installation and documents. Localvault status is shown but does not block startup. You can also switch back to auto-discover from the path settings below.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                {startupError && (
                  <div className="rounded-md border border-[rgb(var(--color-error)/0.3)] bg-[rgb(var(--color-error)/0.1)] px-4 py-3 text-sm text-[rgb(var(--color-error))]">
                    {startupError}
                  </div>
                )}
                {startupPaths && (
                  <div className="rounded-md border border-[rgb(var(--color-surface-border))] bg-[rgb(var(--color-surface-1))] px-4 py-3 text-sm text-[rgb(var(--color-text-secondary))] space-y-1">
                    <div>Game installation: {hasGameInstallPath(startupPaths) ? 'OK' : 'Missing'}</div>
                    <div>Documents folder: {hasDocumentsPath(startupPaths) ? 'OK' : 'Missing'}</div>
                    <div>Localvault folder: {hasLocalVaultPath(startupPaths) ? 'OK' : 'Missing'}</div>
                  </div>
                )}
                <div className="flex flex-wrap items-center gap-3">
                  <Button
                    onClick={handleContinueAfterManualSetup}
                    disabled={!startupPaths || missingRequiredPaths(startupPaths).length > 0}
                    loading={startupBusy}
                  >
                    Continue To App
                  </Button>
                  <Button onClick={handleStartAutoSetup} variant="outline" disabled={startupBusy}>
                    Try Auto-Discover Instead
                  </Button>
                </div>
              </CardContent>
            </Card>

            <SettingsPage
              initialTab="paths"
              onPathsUpdated={setStartupPaths}
            />
          </div>
        </div>
      </div>
    );
  }

  if (!appReady || isLoading || !isAvailable || !api) {
    return (
      <div className="h-screen w-screen flex items-center justify-center bg-[rgb(var(--color-background))]">
        <div className="max-w-md w-full mx-4">
          <div className="bg-[rgb(var(--color-surface-1))] rounded-lg shadow-lg border border-[rgb(var(--color-surface-border))] p-8 text-center">
              <h1 className="text-3xl font-bold font-sans text-[rgb(var(--color-text-primary))] mb-6">Initializing...</h1>


            <div className="w-full bg-[rgb(var(--color-surface-2))] rounded-full h-5 mb-4 relative overflow-hidden">
               <div 
                 className="bg-[rgb(var(--color-primary))] h-full rounded-full transition-all duration-500 ease-out"
                 style={{ width: `${initProgress.progress}%` }}
               ></div>
               <div className="absolute inset-0 flex items-center justify-center text-sm font-bold text-[rgb(var(--color-text-primary))] drop-shadow-md">
                 {Math.round(initProgress.progress)}%
               </div>
            </div>
            
            

          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col overflow-hidden">
      <TauriInitializer />

      <div className="flex-shrink-0">
        <CustomTitleBar />
      </div>

      <div className="flex-1 flex overflow-hidden">
        {(() => {
          if (viewMode === 'editor' && character) {
             return (
              <div className="flex flex-col w-full h-full overflow-hidden">
                 <EditorHeader
                    characterName={character.name}
                    saveName="Save File"
                    onBack={handleEditorBack}
                    onImport={() => {}}
                    onExport={handleExportToVault}
                    onSave={handleSaveCharacter}
                    isModified={character.has_unsaved_changes}
                  />

                 <div className="flex-1 flex overflow-hidden">
                    <Sidebar 
                      activeTab={activeTab === 'dashboard_minimized' ? 'overview' : activeTab} 
                      onTabChange={handleTabChange}
                      currentCharacter={currentCharacter}
                      onBackToMain={handleBackToMain}
                      isLoading={characterLoading}
                    />
                    
                    <div className="flex-1 flex flex-col overflow-hidden">
                      <main className="flex-1 bg-[rgb(var(--color-background))] overflow-y-auto">
                        <ErrorBoundary
                          key={activeTab}
                          fallbackTitle={t('errors.boundary.title')}
                          fallbackMessage={t('errors.boundary.message')}
                          fallbackRetry={t('errors.boundary.retry')}
                        >
                        <div className="p-6">
                          {activeTab === 'overview' && (
                            <div className="space-y-6">
                              <h2 className="text-2xl font-semibold text-[rgb(var(--color-text-primary))]">{t('navigation.overview')}</h2>
                              <CharacterOverview onNavigate={setActiveTab} />
                            </div>
                          )}
                          
                          {activeTab === 'character-builder' && (
                            <div className="space-y-6">
                              <h2 className="text-2xl font-semibold text-[rgb(var(--color-text-primary))]">{t('navigation.characterBuilder')}</h2>
                              <CharacterBuilder key={characterBuilderKey} />
                            </div>
                          )}
                          
                          {activeTab === 'abilityScores' && (
                            <div className="space-y-6">
                              <h2 className="text-2xl font-semibold text-[rgb(var(--color-text-primary))]">{t('navigation.abilityScores')}</h2>
                              <AbilityScoresEditor />
                            </div>
                          )}
                          
                          {activeTab === 'appearance' && (
                            <div className="space-y-6">
                              <h2 className="text-2xl font-semibold text-[rgb(var(--color-text-primary))]">{t('navigation.appearance')}</h2>
                              <AppearanceEditor />
                            </div>
                          )}
                          
                          {activeTab === 'classes' && (
                            <div className="space-y-6">
                              <h2 className="text-2xl font-semibold text-[rgb(var(--color-text-primary))]">{t('navigation.classes')}</h2>
                              <ClassAndLevelsEditor 
                                onNavigate={(path) => {
                                  // Simple mapping: remove leading slash
                                  const tabId = path.replace(/^\//, '');
                                  setActiveTab(tabId);
                                }} 
                                onLevelGains={handleLevelGains}
                              />
                            </div>
                          )}
                          
                          {activeTab === 'skills' && (
                            <div className="space-y-6">
                              <h2 className="text-2xl font-semibold text-[rgb(var(--color-text-primary))]">{t('navigation.skills')}</h2>
                              <SkillsEditor />
                            </div>
                          )}
                          
                          {activeTab === 'feats' && (
                            <div className="space-y-6">
                              <h2 className="text-2xl font-semibold text-[rgb(var(--color-text-primary))]">{t('navigation.feats')}</h2>
                              <FeatsEditor />
                            </div>
                          )}
                          
                          {activeTab === 'spells' && (
                            <div className="space-y-6">
                              <h2 className="text-2xl font-semibold text-[rgb(var(--color-text-primary))]">{t('navigation.spells')}</h2>
                              <SpellsEditor />
                            </div>
                          )}
                          
                          {activeTab === 'inventory' && (
                            <div className="space-y-6">
                              <h2 className="text-2xl font-semibold text-[rgb(var(--color-text-primary))]">{t('navigation.inventory')}</h2>
                              <InventoryEditor />
                            </div>
                          )}
                          
                          {activeTab === 'companions' && <CompanionsView onLoadCompanion={handleLoadCompanion} currentCharacterName={currentCharacter?.name} />}
    
                          {activeTab === 'gameState' && (
                            <div className="space-y-6">
                              <h2 className="text-2xl font-semibold text-[rgb(var(--color-text-primary))]">{t('navigation.gameState')}</h2>
                              <GameStateEditor />
                            </div>
                          )}

                          {activeTab === 'settings' && (
                            <div className="space-y-6">

                               <SettingsPage />
                            </div>
                          )}
                        </div>
                        </ErrorBoundary>
                      </main>
                    </div>
                 </div>
              </div>
             );
          }

          if (showSettings) {
            return (
              <div className="flex flex-col h-full w-full bg-[rgb(var(--color-background))]">
                <div className="h-14 flex items-center px-4 bg-[rgb(var(--color-surface-2))] border-b border-[rgb(var(--color-surface-border))] shadow-sm flex-shrink-0">
                  <div className="flex items-center w-full">
                    <Button 
                      variant="ghost" 
                      onClick={handleCloseSettings}
                      className="text-[rgb(var(--color-text-secondary))] hover:text-[rgb(var(--color-text-primary))]"
                    >
                      <span className="mr-2">←</span> Back to Dashboard
                    </Button>
                    <div className="h-6 w-px bg-[rgb(var(--color-surface-border))] mx-4"></div>
                    <span className="text-sm font-medium text-[rgb(var(--color-text-secondary))]">Application Settings</span>
                  </div>
                </div>

                <div className="flex-1 overflow-y-auto p-6 md:p-8 max-w-5xl mx-auto w-full">
                   <SettingsPage />
                </div>
              </div>
            );
          }

          return (
            <Dashboard 
              onOpenBackups={handleOpenBackups}
              onOpenFolder={handleOpenFolder}
              onSettings={handleSettings}
              onImportCharacter={handleImportCharacter}
              activeCharacter={character ? { name: character.name } : undefined}
              onContinueEditing={handleContinueEditing}
              onCloseSession={handleCloseSession}
            />
          );
        })()}
      </div>

      {showLoadingOverlay && (
        <div className="fixed inset-0 z-[9999] flex flex-col items-center justify-center bg-[rgb(var(--color-background))] animate-in fade-in duration-200">
          <div className="flex flex-col items-center gap-4">
            <div className="w-12 h-12 rounded-full border-2 border-[rgb(var(--color-primary)/0.2)] border-t-[rgb(var(--color-primary))] animate-spin"></div>
            <div className="text-[rgb(var(--color-text-primary))] font-medium animate-pulse">
              Loading save
            </div>
          </div>
        </div>
      )}

      {feedback && (
        <div className="fixed top-20 right-8 z-[10000] animate-in fade-in slide-in-from-right-8 duration-300">
          <div className={`rounded-lg shadow-lg border p-4 flex items-center gap-3 ${
            feedback.type === 'success' 
              ? 'bg-green-900/90 border-green-500 text-green-100' 
              : 'bg-red-900/90 border-red-500 text-red-100'
          }`}>
             <div className={`p-1 rounded-full ${feedback.type === 'success' ? 'bg-green-500/20' : 'bg-red-500/20'}`}>
                {feedback.type === 'success' ? (
                  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" /></svg>
                ) : (
                  <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                )}
             </div>
             <div>
               <h4 className="font-semibold">{feedback.type === 'success' ? 'Success' : 'Error'}</h4>
               <p className="text-sm opacity-90">{feedback.message}</p>
             </div>
             <button onClick={() => setFeedback(null)} className="ml-4 hover:opacity-75">
               <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" /></svg>
             </button>
          </div>
        </div>
      )}

      <LevelHelperModal
        isOpen={showLevelHelper}
        onClose={() => setShowLevelHelper(false)}
        className=""
        onNavigate={(path) => {
           const tabId = path.replace(/^\//, '');
           setActiveTab(tabId);
        }}
      />

      <GameLaunchDialog
        isOpen={showLaunchDialog}
        onClose={() => setShowLaunchDialog(false)}
        onLaunch={handleLaunchGame}
        saveName={character?.name}
      />

      <FileBrowserModal
        isOpen={showVaultBrowser}
        onClose={() => setShowVaultBrowser(false)}
        mode="load-vault"
        onSelectFile={handleVaultSelect}
        currentPath={vaultPath}
        onPathChange={setVaultPath}
      />
    </div>
  );
}

export default function ClientOnlyApp() {
  return (
    <IconCacheProvider>
      <CharacterProvider>
        <AppContent />
      </CharacterProvider>
    </IconCacheProvider>
  );
}
