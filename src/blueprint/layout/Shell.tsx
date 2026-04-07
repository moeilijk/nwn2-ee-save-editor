import { useState, useEffect } from 'react';
import { Button, Card, Elevation, Icon, Overlay2, ProgressBar, Spinner } from '@blueprintjs/core';
import { invoke } from '@tauri-apps/api/core';
import { TauriAPI } from '@/lib/tauri-api';
import { CharacterProvider, useCharacterContext } from '@/contexts/CharacterContext';
import { useTranslations } from '@/hooks/useTranslations';
import { T, PATTERN_BG } from '../theme';
import '../blueprint.css';
import { TitleBar } from './TitleBar';
import { Navbar } from './Navbar';
import { Sidebar } from './Sidebar';
import { OverviewPanel } from '../Overview/OverviewPanel';
import { AbilitiesPanel } from '../AbilityScores/AbilitiesPanel';
import { ClassesPanel } from '../ClassesLevel/ClassesPanel';
import { SkillsPanel } from '../Skills/SkillsPanel';
import { FeatsPanel } from '../Feats/FeatsPanel';
import { SpellsPanel } from '../Spells/SpellsPanel';
import { InventoryPanel } from '../Inventory/InventoryPanel';
import { GameStatePanel } from '../GameState/GameStatePanel';
import DashboardPanel from '../Dashboard/DashboardPanel';
import { LevelHelper, ErrorBoundary, AboutDialog } from '../shared';

const PANELS: Record<string, React.ComponentType> = {
  overview: OverviewPanel,
  abilities: AbilitiesPanel,
  classes: ClassesPanel,
  skills: SkillsPanel,
  feats: FeatsPanel,
  spells: SpellsPanel,
  inventory: InventoryPanel,
  gamestate: GameStatePanel,
};

function LoadingOverlay({ isOpen, message }: { isOpen: boolean; message: string }) {
  return (
    <Overlay2 isOpen={isOpen} canEscapeKeyClose={false} canOutsideClickClose={false}>
      <div style={{
        position: 'fixed', inset: 0,
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        background: 'rgba(0,0,0,0.5)',
      }}>
        <div style={{ textAlign: 'center' }}>
          <Spinner size={40} intent="primary" />
          <div style={{ marginTop: 12, color: '#e8e4dc', fontSize: 14 }}>{message}</div>
        </div>
      </div>
    </Overlay2>
  );
}

function ShellContent() {
  const [activeTab, setActiveTab] = useState('overview');
  const [showAbout, setShowAbout] = useState(false);
  const [viewMode, setViewMode] = useState<'dashboard' | 'editor'>('dashboard');
  const { character, isLoading, clearCharacter } = useCharacterContext();
  const t = useTranslations();
  const Panel = PANELS[activeTab];

  useEffect(() => {
    if (character && !isLoading && viewMode === 'dashboard') {
      setViewMode('editor');
    }
    if (!character) {
      setViewMode('dashboard');
    }
  }, [character, isLoading]);

  const handleBackToDashboard = () => setViewMode('dashboard');

  const handleCloseSession = () => {
    clearCharacter();
    setViewMode('dashboard');
  };

  if (viewMode === 'dashboard' || !character) {
    return (
      <div className="bp-app" style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: T.bg }}>
        <TitleBar onAboutClick={() => setShowAbout(true)} />
        {character && (
          <div style={{
            display: 'flex', alignItems: 'center', justifyContent: 'space-between',
            padding: '8px 16px', background: T.sectionBg, borderBottom: `1px solid ${T.sectionBorder}`,
          }}>
            <span style={{ fontSize: 13, color: T.text }}>
              <Icon icon="person" size={12} style={{ marginRight: 6 }} />
              <strong>{t('dashboard.sessionActive', { name: character.name })}</strong>
            </span>
            <div style={{ display: 'flex', gap: 8 }}>
              <Button small intent="primary" text={t('dashboard.continueEditing')} onClick={() => setViewMode('editor')} />
              <Button small text={t('dashboard.closeSession')} onClick={handleCloseSession} />
            </div>
          </div>
        )}
        <DashboardPanel />
        <AboutDialog isOpen={showAbout} onClose={() => setShowAbout(false)} />
        <LoadingOverlay isOpen={isLoading} message={t('common.loading')} />
      </div>
    );
  }

  return (
    <div className="bp-app" style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: T.bg }}>
      <TitleBar onAboutClick={() => setShowAbout(true)} />
      <Navbar onBack={handleBackToDashboard} />
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
        <div style={{ flex: 1, overflow: 'hidden' }}>
          <Card elevation={Elevation.ONE} style={{
            margin: 0, padding: 0, height: '100%', borderRadius: 0, overflow: 'auto',
            background: T.surface,
            backgroundImage: PATTERN_BG,
            backgroundSize: '200px 200px',
          }}>
            <LevelHelper onNavigate={setActiveTab} />
            <ErrorBoundary key={activeTab}>
              {Panel ? <Panel /> : (
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
                  <div style={{ textAlign: 'center' }}>
                    <Icon icon="build" size={40} style={{ color: T.border }} />
                    <p style={{ marginTop: 12, fontSize: 14, color: T.textMuted }}>Coming soon</p>
                  </div>
                </div>
              )}
            </ErrorBoundary>
          </Card>
        </div>
      </div>
      <AboutDialog isOpen={showAbout} onClose={() => setShowAbout(false)} />
      <LoadingOverlay isOpen={isLoading} message={t('common.loading')} />
    </div>
  );
}

export default function Shell() {
  const [initReady, setInitReady] = useState(false);
  const [initProgress, setInitProgress] = useState({ step: 'initializing', progress: 0, message: 'Starting up...' });

  useEffect(() => {
    invoke('show_main_window').catch(() => {});
    TauriAPI.initializeGameData().catch(err => {
      console.error('Failed to initialize game data:', err);
    });

    let cancelled = false;
    const poll = async () => {
      try {
        const status = await TauriAPI.getInitializationStatus();
        if (cancelled) return;
        setInitProgress(status);
        if (status.step === 'ready') {
          setInitReady(true);
          return;
        }
      } catch {
        // backend not ready yet
      }
      if (!cancelled) setTimeout(poll, 500);
    };
    poll();

    return () => { cancelled = true; };
  }, []);

  if (!initReady) {
    return (
      <div className="bp-app" style={{
        display: 'flex', flexDirection: 'column',
        height: '100vh', background: T.bg,
        backgroundImage: PATTERN_BG, backgroundSize: '200px 200px',
      }}>
        <TitleBar onAboutClick={() => {}} />
        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <div style={{ width: 320, textAlign: 'center' }}>
            <div style={{ fontSize: 18, fontWeight: 700, color: T.accent, marginBottom: 16 }}>
              NWN2 Save Editor
            </div>
            <ProgressBar
              value={initProgress.progress / 100}
              intent="primary"
              animate={initProgress.step !== 'ready'}
              stripes={false}
              style={{ marginBottom: 8 }}
            />
            <div style={{ fontSize: 12, color: T.textMuted }}>{initProgress.message}</div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <CharacterProvider>
      <ShellContent />
    </CharacterProvider>
  );
}
