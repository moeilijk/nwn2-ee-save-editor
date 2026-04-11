import { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/Select';
import { Label } from '@/components/ui/Label';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/Tabs';
import { pathService, PathConfig } from '@/lib/api/paths';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { FolderIcon, CheckCircleIcon, XCircleIcon, PlusIcon, TrashIcon, CogIcon, LanguageIcon, PaintBrushIcon, BugAntIcon } from '@heroicons/react/24/outline';
import ThemeCustomizer from '@/components/Settings/ThemeCustomizer';
import { useLocale } from '@/providers/LocaleProvider';
import { useTranslations } from '@/hooks/useTranslations';

interface AppSettings {
  theme: 'light' | 'dark';
  fontSize: 'small' | 'medium' | 'large';
}

interface SettingsPageProps {
  initialTab?: 'general' | 'theme' | 'paths';
  onPathsUpdated?: (paths: PathConfig) => void;
}

export default function SettingsPage({
  initialTab = 'general',
  onPathsUpdated
}: SettingsPageProps) {
  const { locale, setLocale } = useLocale();
  const t = useTranslations();
  const [paths, setPaths] = useState<PathConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [appSettings, setAppSettings] = useState<AppSettings>({
    theme: 'dark',
    fontSize: 'medium'
  });
  const [debugExporting, setDebugExporting] = useState(false);
  const [debugExportResult, setDebugExportResult] = useState<{ success: boolean; message: string } | null>(null);

  const applyPaths = useCallback((nextPaths: PathConfig) => {
    setPaths(nextPaths);
    onPathsUpdated?.(nextPaths);
  }, [onPathsUpdated]);

  const loadPaths = useCallback(async () => {
    try {
      setLoading(true);
      const response = await pathService.getConfig();
      applyPaths(response.paths);
    } catch (err) {
      setError('Failed to load path configuration');
      console.error('Error loading paths:', err);
    } finally {
      setLoading(false);
    }
  }, [applyPaths]);

  useEffect(() => {
    loadPaths();
    loadAppSettings();
  }, [loadPaths]);

  const loadAppSettings = () => {
    try {
      const saved = localStorage.getItem('nwn2ee-app-settings');
      if (saved) {
        setAppSettings(JSON.parse(saved));
      }
    } catch (err) {
      console.error('Error loading app settings:', err);
    }
  };

  const saveAppSettings = (newSettings: Partial<AppSettings>) => {
    const updated = { ...appSettings, ...newSettings };
    setAppSettings(updated);
    localStorage.setItem('nwn2ee-app-settings', JSON.stringify(updated));
    if (newSettings.theme) {
      applyTheme(newSettings.theme);
    }
    if (newSettings.fontSize) {
      applyFontSize(newSettings.fontSize);
    }
  };

  const applyTheme = (theme: 'light' | 'dark') => {
    const root = document.documentElement;
    if (theme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  };

  const applyFontSize = (fontSize: 'small' | 'medium' | 'large') => {
    const root = document.documentElement;
    root.classList.remove('text-sm', 'text-base', 'text-lg');
    
    switch (fontSize) {
      case 'small':
        root.classList.add('text-sm');
        break;
      case 'large':
        root.classList.add('text-lg');
        break;
      default:
        root.classList.add('text-base');
    }
  };

  const selectFolder = async (title: string): Promise<string | null> => {
    const selected = await open({
      directory: true,
      multiple: false,
      title
    });
    return selected as string | null;
  };

  const updatePath = async (
    type: 'game' | 'documents' | 'workshop'
  ) => {
    const title = type === 'game' ? 'Select NWN2 Game Folder' :
                  type === 'documents' ? 'Select NWN2 Documents Folder' :
                  'Select Steam Workshop Folder';
    
    const selected = await selectFolder(title);
    if (!selected) return;

    try {
      setSaving(true);
      let response;
      switch (type) {
        case 'game':
          response = await pathService.setGameFolder(selected);
          break;
        case 'documents':
          response = await pathService.setDocumentsFolder(selected);
          break;
        case 'workshop':
          response = await pathService.setSteamWorkshopFolder(selected);
          break;
      }
      applyPaths(response.paths);
      setError(null);
    } catch (err) {
      setError(`Failed to update ${type} folder`);
      console.error('Error updating path:', err);
    } finally {
      setSaving(false);
    }
  };

  const resetPath = async (
    type: 'game' | 'documents' | 'workshop'
  ) => {
    try {
      setSaving(true);
      let response;
      switch (type) {
        case 'game':
          response = await pathService.resetGameFolder();
          break;
        case 'documents':
          response = await pathService.resetDocumentsFolder();
          break;
        case 'workshop':
          response = await pathService.resetSteamWorkshopFolder();
          break;
      }
      applyPaths(response.paths);
      setError(null);
    } catch (err) {
      setError(`Failed to reset ${type} folder`);
      console.error('Error resetting path:', err);
    } finally {
      setSaving(false);
    }
  };

  const addCustomFolder = async (type: 'override' | 'hak') => {
    const title = `Select Custom ${type.charAt(0).toUpperCase() + type.slice(1)} Folder`;
    const selected = await selectFolder(title);
    if (!selected) return;

    try {
      setSaving(true);
      let response;
      switch (type) {
        case 'override':
          response = await pathService.addOverrideFolder(selected);
          break;
        case 'hak':
          response = await pathService.addHakFolder(selected);
          break;
      }
      applyPaths(response.paths);
      setError(null);
    } catch (err) {
      setError(`Failed to add custom ${type} folder`);
      console.error('Error adding custom folder:', err);
    } finally {
      setSaving(false);
    }
  };

  const removeCustomFolder = async (type: 'override' | 'hak', path: string) => {
    try {
      setSaving(true);
      let response;
      switch (type) {
        case 'override':
          response = await pathService.removeOverrideFolder(path);
          break;
        case 'hak':
          response = await pathService.removeHakFolder(path);
          break;
      }
      applyPaths(response.paths);
      setError(null);
    } catch (err) {
      setError(`Failed to remove custom ${type} folder`);
      console.error('Error removing custom folder:', err);
    } finally {
      setSaving(false);
    }
  };

  const exportDebugLog = async () => {
    try {
      setDebugExporting(true);
      setDebugExportResult(null);
      const filePath = await invoke<string>('export_debug_log');
      setDebugExportResult({
        success: true,
        message: `${t('settings.debug.exportSuccess')} ${filePath}`
      });
    } catch (err) {
      console.error('Error exporting debug log:', err);
      setDebugExportResult({
        success: false,
        message: t('settings.debug.exportError')
      });
    } finally {
      setDebugExporting(false);
    }
  };

  const handleAutoDiscover = async () => {
    try {
      setSaving(true);
      setError(null);
      const response = await pathService.autoDetect();
      applyPaths(response.current_paths);

      const missing: string[] = [];
      if (!response.current_paths.game_folder.exists) {
        missing.push('game installation folder');
      }
      if (!response.current_paths.documents_folder.exists) {
        missing.push('documents folder');
      }
      if (missing.length > 0) {
        setError(`Auto-discovery did not resolve all required paths: ${missing.join(', ')}.`);
      }
    } catch (err) {
      setError('Failed to auto-discover required NWN2 folders');
      console.error('Error auto-detecting paths:', err);
    } finally {
      setSaving(false);
    }
  };

  const handleSetupModeChange = async (mode: 'auto' | 'manual') => {
    try {
      setSaving(true);
      setError(null);
      const response = await pathService.setSetupMode(mode);
      applyPaths(response.paths);
    } catch (err) {
      setError(`Failed to switch startup mode to ${mode}`);
      console.error('Error updating setup mode:', err);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto"></div>
          <p className="mt-4 text-sm text-muted-foreground">Loading path configuration...</p>
        </div>
      </div>
    );
  }

  if (!paths) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center text-destructive">
          <p>Failed to load path configuration</p>
          <Button onClick={loadPaths} className="mt-4">
            Retry
          </Button>
        </div>
      </div>
    );
  }

  const PathRow = ({ 
    label, 
    path, 
    exists, 
    autoDetected,
    onEdit,
    onReset
  }: { 
    label: string;
    path: string | null;
    exists: boolean;
    autoDetected: boolean;
    onEdit?: () => void;
    onReset?: () => void;
  }) => (
    <div className="flex items-center justify-between py-3 border-b last:border-0">
      <div className="flex-1">
        <div className="flex items-center gap-2">
          <FolderIcon className="w-5 h-5 text-muted-foreground" />
          <span className="font-medium">{label}</span>
          {autoDetected ? (
            <span className="text-xs bg-blue-100 text-blue-800 px-2 py-0.5 rounded">Auto-detected</span>
          ) : (
            <span className="text-xs bg-amber-100 text-amber-800 px-2 py-0.5 rounded">Manually Set</span>
          )}
        </div>
        <div className="mt-1 flex items-center gap-2">
          <span className="text-sm text-muted-foreground font-mono">
            {path || '(Not configured)'}
          </span>
          {path && (
            exists ? 
              <CheckCircleIcon className="w-4 h-4 text-green-600" /> :
              <XCircleIcon className="w-4 h-4 text-red-600" />
          )}
        </div>
      </div>
      <div className="flex items-center gap-2">
        {!autoDetected && (
          <Button 
            onClick={onReset} 
            variant="ghost" 
            size="sm"
            disabled={saving}
            className="text-muted-foreground hover:text-destructive transition-colors"
          >
            Reset
          </Button>
        )}
        <Button 
          onClick={onEdit} 
          variant="outline" 
          size="sm"
          disabled={saving}
        >
          {path ? 'Change' : 'Set'}
        </Button>
      </div>
    </div>
  );

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-semibold text-[rgb(var(--color-text-primary))]">Settings</h2>
      
      <Tabs defaultValue={initialTab} className="w-full">
        <TabsList className="w-full flex bg-transparent p-0 gap-2 mb-6">
          <TabsTrigger 
            value="general" 
            className="flex-1 h-10 rounded-md border border-[rgb(var(--color-primary))] text-[rgb(var(--color-primary))] bg-transparent data-[state=active]:!bg-[rgb(var(--color-primary))] data-[state=active]:!text-white transition-colors hover:bg-[rgb(var(--color-primary))/10] flex items-center justify-center gap-2"
          >
            <CogIcon className="w-4 h-4" />
            General
          </TabsTrigger>
          <TabsTrigger 
            value="theme" 
            className="flex-1 h-10 rounded-md border border-[rgb(var(--color-primary))] text-[rgb(var(--color-primary))] bg-transparent data-[state=active]:!bg-[rgb(var(--color-primary))] data-[state=active]:!text-white transition-colors hover:bg-[rgb(var(--color-primary))/10] flex items-center justify-center gap-2"
          >
            <PaintBrushIcon className="w-4 h-4" />
            Theme
          </TabsTrigger>
          <TabsTrigger 
            value="paths" 
            className="flex-1 h-10 rounded-md border border-[rgb(var(--color-primary))] text-[rgb(var(--color-primary))] bg-transparent data-[state=active]:!bg-[rgb(var(--color-primary))] data-[state=active]:!text-white transition-colors hover:bg-[rgb(var(--color-primary))/10] flex items-center justify-center gap-2"
          >
            <FolderIcon className="w-4 h-4" />
            Game Paths
          </TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="space-y-6">

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <LanguageIcon className="w-5 h-5" />
                Language & Region
              </CardTitle>
              <CardDescription>Language and localization settings</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="language">Language</Label>
                <Select 
                  value={locale} 
                  onValueChange={(value: string) => setLocale(value)}
                >
                  <SelectTrigger id="language">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="en">English</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <CogIcon className="w-5 h-5" />
                Display
              </CardTitle>
              <CardDescription>Adjust display settings</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="fontSize">Font Size</Label>
                <Select 
                  value={appSettings.fontSize} 
                  onValueChange={(value: 'small' | 'medium' | 'large') => 
                    saveAppSettings({ fontSize: value })
                  }
                >
                  <SelectTrigger id="fontSize">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="small">Small</SelectItem>
                    <SelectItem value="medium">Medium</SelectItem>
                    <SelectItem value="large">Large</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <BugAntIcon className="w-5 h-5" />
                {t('settings.debug.title')}
              </CardTitle>
              <CardDescription>Export diagnostic information for bug reports</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center gap-4">
                <Button
                  onClick={exportDebugLog}
                  variant="outline"
                  disabled={debugExporting}
                >
                  {debugExporting ? t('settings.debug.exporting') : t('settings.debug.exportButton')}
                </Button>
              </div>
              {debugExportResult && (
                <div className={`text-sm p-3 rounded-md ${
                  debugExportResult.success
                    ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
                    : 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400'
                }`}>
                  {debugExportResult.message}
                </div>
              )}
            </CardContent>
          </Card>

        </TabsContent>

        <TabsContent value="theme" className="space-y-6">
          <ThemeCustomizer />
        </TabsContent>

        <TabsContent value="paths" className="space-y-6">
          <h2 className="text-xl font-semibold">Game Paths Configuration</h2>

      {error && (
        <Card className="border-destructive">
          <CardContent className="pt-6">
            <p className="text-sm text-destructive">{error}</p>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle>Startup Path Mode</CardTitle>
          <CardDescription>
            Choose whether the app should auto-discover missing NWN2 folders on startup or wait for manual paths. Game and documents paths are required.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center gap-2 text-sm">
            <span className="text-muted-foreground">Current mode:</span>
            <span className="rounded px-2 py-0.5 text-xs font-medium bg-[rgb(var(--color-surface-1))] border border-[rgb(var(--color-surface-border))]">
              {paths.setup_mode === 'auto'
                ? 'Auto-Discover'
                : paths.setup_mode === 'manual'
                  ? 'Manual'
                  : 'Not chosen'}
            </span>
          </div>
          <div className="flex flex-wrap gap-3">
            <Button
              onClick={handleAutoDiscover}
              variant={paths.setup_mode === 'auto' ? 'primary' : 'outline'}
              disabled={saving}
            >
              {paths.setup_mode === 'auto' ? 'Re-run Auto-Discover' : 'Use Auto-Discover'}
            </Button>
            <Button
              onClick={() => handleSetupModeChange('manual')}
              variant={paths.setup_mode === 'manual' ? 'secondary' : 'outline'}
              disabled={saving}
            >
              Use Manual Setup
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Main Paths</CardTitle>
          <CardDescription>Configure the main NWN2 installation and user directories</CardDescription>
        </CardHeader>
        <CardContent>
          <PathRow
            label="Game Installation Folder"
            path={paths.game_folder.path}
            exists={paths.game_folder.exists}
            autoDetected={paths.game_folder.source === 'auto'}
            onEdit={() => updatePath('game')}
            onReset={() => resetPath('game')}
          />
          <PathRow
            label="Documents Folder"
            path={paths.documents_folder.path}
            exists={paths.documents_folder.exists}
            autoDetected={paths.documents_folder.source === 'auto'}
            onEdit={() => updatePath('documents')}
            onReset={() => resetPath('documents')}
          />
          <PathRow
            label="Steam Workshop Folder"
            path={paths.steam_workshop_folder.path}
            exists={paths.steam_workshop_folder.exists}
            autoDetected={paths.steam_workshop_folder.source === 'auto'}
            onEdit={() => updatePath('workshop')}
            onReset={() => resetPath('workshop')}
          />
          <PathRow
            label="Character Vault (LocalVault)"
            path={paths.localvault_folder.path}
            exists={paths.localvault_folder.exists}
            autoDetected={paths.localvault_folder.source === 'derived'}
          />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Custom Override Folders</CardTitle>
          <CardDescription>Additional directories to search for override files</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {paths.custom_override_folders.map((folder, i) => (
              <div key={i} className="flex items-center justify-between p-3 bg-muted rounded-md">
                <div className="flex items-center gap-2">
                  <FolderIcon className="w-5 h-5 text-muted-foreground" />
                  <span className="text-sm font-mono">{folder.path}</span>
                  {folder.exists ? 
                    <CheckCircleIcon className="w-4 h-4 text-green-600" /> :
                    <XCircleIcon className="w-4 h-4 text-red-600" />
                  }
                </div>
                <Button
                  onClick={() => removeCustomFolder('override', folder.path)}
                  variant="ghost"
                  size="sm"
                  disabled={saving}
                >
                  <TrashIcon className="w-4 h-4" />
                </Button>
              </div>
            ))}
            <Button
              onClick={() => addCustomFolder('override')}
              variant="outline"
              size="sm"
              className="w-full"
              disabled={saving}
            >
              <PlusIcon className="w-4 h-4 mr-2" />
              Add Override Folder
            </Button>
          </div>
        </CardContent>
      </Card>


      <Card>
        <CardHeader>
          <CardTitle>Custom HAK Folders</CardTitle>
          <CardDescription>Additional directories to search for HAK files</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {paths.custom_hak_folders.map((folder, i) => (
              <div key={i} className="flex items-center justify-between p-3 bg-muted rounded-md">
                <div className="flex items-center gap-2">
                  <FolderIcon className="w-5 h-5 text-muted-foreground" />
                  <span className="text-sm font-mono">{folder.path}</span>
                  {folder.exists ? 
                    <CheckCircleIcon className="w-4 h-4 text-green-600" /> :
                    <XCircleIcon className="w-4 h-4 text-red-600" />
                  }
                </div>
                <Button
                  onClick={() => removeCustomFolder('hak', folder.path)}
                  variant="ghost"
                  size="sm"
                  disabled={saving}
                >
                  <TrashIcon className="w-4 h-4" />
                </Button>
              </div>
            ))}
            <Button
              onClick={() => addCustomFolder('hak')}
              variant="outline"
              size="sm"
              className="w-full"
              disabled={saving}
            >
              <PlusIcon className="w-4 h-4 mr-2" />
              Add HAK Folder
            </Button>
          </div>
        </CardContent>
      </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
