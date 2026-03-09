import { Icon, type IconName } from '@blueprintjs/core';
import { T } from '../theme';

const NAV_ITEMS: { id: string; icon: IconName; label: string }[] = [
  { id: 'overview', icon: 'person', label: 'Overview' },
  { id: 'abilities', icon: 'properties', label: 'Abilities' },
  { id: 'classes', icon: 'layers', label: 'Classes' },
  { id: 'skills', icon: 'build', label: 'Skills' },
  { id: 'feats', icon: 'star', label: 'Feats' },
  { id: 'spells', icon: 'flash', label: 'Spells' },
  { id: 'inventory', icon: 'box', label: 'Inventory' },
  { id: 'gamestate', icon: 'globe', label: 'Game State' },
];

interface SidebarProps {
  activeTab: string;
  onTabChange: (id: string) => void;
}

export function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  return (
    <div style={{
      width: 160, flexShrink: 0,
      background: T.sidebar,
      borderRight: `1px solid ${T.sidebar}`,
      display: 'flex', flexDirection: 'column',
      padding: '8px 0',
      position: 'relative', zIndex: 5,
    }}>
      {NAV_ITEMS.map(item => {
        const active = activeTab === item.id;
        return (
          <button
            key={item.id}
            onClick={() => onTabChange(item.id)}
            style={{
              display: 'flex', alignItems: 'center', gap: 8,
              width: '100%', padding: '8px 16px',
              border: 'none', cursor: 'pointer',
              background: active ? 'rgba(160,82,45,0.12)' : 'transparent',
              color: active ? T.sidebarAccent : T.sidebarText,
              fontSize: 13, textAlign: 'left',
              borderLeft: active ? `2px solid ${T.sidebarAccent}` : '2px solid transparent',
              transition: 'all 0.15s',
            }}
          >
            <Icon icon={item.icon} size={14} />
            {item.label}
          </button>
        );
      })}
    </div>
  );
}
