import { useState } from 'react';
import { Card, Elevation, Icon } from '@blueprintjs/core';
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

export default function Shell() {
  const [activeTab, setActiveTab] = useState('overview');
  const Panel = PANELS[activeTab];

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
