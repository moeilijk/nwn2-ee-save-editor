import {
  Button, Icon, Navbar as BPNavbar, NavbarGroup, NavbarHeading, NavbarDivider, Tag,
} from '@blueprintjs/core';
import { T } from '../theme';
import { CHARACTER } from '../dummy-data';

export function Navbar() {
  return (
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
        <Tag minimal round style={{ background: 'rgba(90,122,90,0.2)', color: '#7aad7a', border: '1px solid rgba(90,122,90,0.3)' }}>
          <Icon icon="tick-circle" size={12} style={{ marginRight: 4 }} />Saved
        </Tag>
        <NavbarDivider />
        <Button icon="floppy-disk" text="Save" small style={{ background: T.accent, color: '#fff', border: 'none' }} />
        <Button icon="export" text="Export" small minimal style={{ color: T.sidebarText }} />
        <Button icon="arrow-left" text="Back" small minimal style={{ color: T.sidebarText }} />
      </NavbarGroup>
    </BPNavbar>
  );
}
