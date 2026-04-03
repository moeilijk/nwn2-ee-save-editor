import { useState, useEffect } from 'react';
import { Card, Elevation, Icon, ProgressBar } from '@blueprintjs/core';
import { invoke } from '@tauri-apps/api/core';
import { TauriAPI } from '@/lib/tauri-api';
import { CharacterProvider, useCharacterContext } from '@/contexts/CharacterContext';
import { T, PATTERN_BG } from '../theme';
import '../blueprint.css';
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
import { LevelHelper } from '../shared';

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

function ShellContent() {
  const [activeTab, setActiveTab] = useState('overview');
  const { character } = useCharacterContext();
  const Panel = PANELS[activeTab];

  if (!character) {
    return <DashboardPanel />;
  }

  return (
    <div className="bp-app" style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: T.bg }}>
      <Navbar />
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
            {Panel ? <Panel /> : (
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
                <div style={{ textAlign: 'center' }}>
                  <Icon icon="build" size={40} style={{ color: T.border }} />
                  <p style={{ marginTop: 12, fontSize: 14, color: T.textMuted }}>Coming soon</p>
                </div>
              </div>
            )}
          </Card>
        </div>
      </div>
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
        display: 'flex', alignItems: 'center', justifyContent: 'center',
        height: '100vh', background: T.bg,
        backgroundImage: PATTERN_BG, backgroundSize: '200px 200px',
      }}>
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
    );
  }

  return (
    <CharacterProvider>
      <ShellContent />
    </CharacterProvider>
  );
}
