import { Icon, type IconName } from '@blueprintjs/core';
import { T } from '../theme';
import { useTranslations } from '@/hooks/useTranslations';

const NAV_ITEMS: { id: string; icon: IconName; labelKey: string }[] = [
  { id: 'overview', icon: 'person', labelKey: 'navigation.overview' },
  { id: 'appearance', icon: 'style', labelKey: 'navigation.appearance' },
  { id: 'abilities', icon: 'properties', labelKey: 'navigation.abilityScores' },
  { id: 'classes', icon: 'layers', labelKey: 'navigation.classes' },
  { id: 'skills', icon: 'build', labelKey: 'navigation.skills' },
  { id: 'feats', icon: 'star', labelKey: 'navigation.feats' },
  { id: 'spells', icon: 'flash', labelKey: 'navigation.spells' },
  { id: 'inventory', icon: 'box', labelKey: 'navigation.inventory' },
  { id: 'gamestate', icon: 'globe', labelKey: 'navigation.gameState' },
  { id: 'models', icon: 'cube', labelKey: 'navigation.models' },
];

interface SidebarProps {
  activeTab: string;
  onTabChange: (id: string) => void;
}

export function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  const t = useTranslations();
  return (
    <div style={{
      width: 200, flexShrink: 0,
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
            {t(item.labelKey)}
          </button>
        );
      })}
    </div>
  );
}
