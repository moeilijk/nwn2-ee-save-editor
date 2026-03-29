import { useState } from 'react';
import {
  Button, Navbar as BPNavbar, NavbarGroup, NavbarHeading, NavbarDivider,
} from '@blueprintjs/core';
import { T } from '../theme';
import { CHARACTER } from '../dummy-data';
import { SettingsDialog } from '../Settings/SettingsPanel';

export function Navbar() {
  const [showSettings, setShowSettings] = useState(false);
  return (
    <>
      <BPNavbar className="bp5-dark" style={{ background: T.navbar, paddingLeft: 12, paddingRight: 12, boxShadow: '0 1px 4px rgba(0,0,0,0.15)', position: 'relative', zIndex: 10 }}>
        <NavbarGroup align="left">
          <NavbarHeading style={{ fontSize: 14, fontWeight: 700, marginRight: 8, color: T.sidebarAccent }}>NWN2 Save Editor</NavbarHeading>
          <NavbarDivider />
          <span style={{ fontSize: 13, color: '#e8e4dc' }}>{CHARACTER.name}</span>
          <span style={{ fontSize: 12, marginLeft: 8, color: T.sidebarText }}>
            Lvl {CHARACTER.level} {CHARACTER.classes.map(c => c.name).join('/')}
          </span>
        </NavbarGroup>
        <NavbarGroup align="right">
          <Button icon="floppy-disk" text="Save" small minimal style={{ color: T.sidebarText }} />
          <Button icon="export" text="Export" small minimal style={{ color: T.sidebarText }} />
          <Button icon="arrow-left" text="Back" small minimal style={{ color: T.sidebarText }} />
          <NavbarDivider />
          <Button icon="cog" small minimal style={{ color: T.sidebarText }} onClick={() => setShowSettings(true)} />
        </NavbarGroup>
      </BPNavbar>
      {showSettings && <SettingsDialog isOpen onClose={() => setShowSettings(false)} />}
    </>
  );
}
