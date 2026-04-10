import { useEffect } from 'react';
import { Icon, type IconName } from '@blueprintjs/core';
import { T } from '../theme';
import { useTranslations } from '@/hooks/useTranslations';
import { useIcon, fetchIcon } from '@/hooks/useIcon';

const NAV_ITEMS: { id: string; icon: IconName; gameIcon: string | null; labelKey: string }[] = [
  { id: 'overview', icon: 'person', gameIcon: 'ia_character', labelKey: 'navigation.overview' },
  { id: 'appearance', icon: 'style', gameIcon: 'ia_appear', labelKey: 'navigation.appearance' },
  { id: 'abilities', icon: 'properties', gameIcon: 'ife_toughness', labelKey: 'navigation.abilityScores' },
  { id: 'classes', icon: 'layers', gameIcon: 'ic_b_fighter', labelKey: 'navigation.classes' },
  { id: 'skills', icon: 'build', gameIcon: 'isk_lore', labelKey: 'navigation.skills' },
  { id: 'feats', icon: 'star', gameIcon: 'ife_dodge', labelKey: 'navigation.feats' },
  { id: 'spells', icon: 'flash', gameIcon: 'b_spellbook', labelKey: 'navigation.spells' },
  { id: 'inventory', icon: 'box', gameIcon: 'ia_inventory', labelKey: 'navigation.inventory' },
  { id: 'gamestate', icon: 'globe', gameIcon: 'b_journal', labelKey: 'navigation.gameState' },
  { id: 'models', icon: 'cube', gameIcon: 'is_trueseeing', labelKey: 'navigation.models' },
];

function NavIcon({ gameIcon, fallback, size }: { gameIcon: string | null; fallback: IconName; size: number }) {
  const iconUrl = useIcon(gameIcon);
  if (iconUrl) {
    return <img src={iconUrl} alt="" width={size} height={size} style={{ borderRadius: 2, flexShrink: 0 }} />;
  }
  return <Icon icon={fallback} size={size} />;
}

interface SidebarProps {
  activeTab: string;
  onTabChange: (id: string) => void;
}

export function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  const t = useTranslations();

  useEffect(() => {
    NAV_ITEMS.forEach(item => {
      if (item.gameIcon) fetchIcon(item.gameIcon);
    });
  }, []);
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
            <NavIcon gameIcon={item.gameIcon} fallback={item.icon} size={24} />
            {t(item.labelKey)}
          </button>
        );
      })}
    </div>
  );
}
