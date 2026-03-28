import { useState } from 'react';
import { Button } from '@blueprintjs/core';
import { useTranslations } from '@/hooks/useTranslations';
import { T, PATTERN_BG } from '../theme';
import '../blueprint.css';
import { SaveList } from './SaveList';
import type { SaveEntryData } from './SaveEntry';

const DUMMY_SAVES: SaveEntryData[] = [
  {
    characterName: 'Khelgar Ironfist',
    folderName: '000000 - Chapter Act I',
    date: '2026-03-24 18:45',
    thumbnail: null,
    isActive: false,
  },
  {
    characterName: 'Ammon Jerro',
    folderName: '000001 - Crossroad Keep',
    date: '2026-03-23 14:20',
    thumbnail: null,
    isActive: true,
  },
  {
    characterName: 'Neeshka',
    folderName: '000002 - Highcliff',
    date: '2026-03-20 09:10',
    thumbnail: null,
    isActive: false,
  },
];

export default function DashboardPanel() {
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const t = useTranslations();

  return (
    <div
      className="bp-app"
      style={{
        height: '100vh',
        background: T.sidebar,
        padding: 32,
        display: 'flex',
      }}
    >
      <div style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        borderRadius: 8,
        overflow: 'hidden',
        background: T.bg,
        backgroundImage: PATTERN_BG,
        backgroundSize: '200px 200px',
        boxShadow: '0 4px 24px rgba(0, 0, 0, 0.3)',
      }}>
        {/* Top bar */}
        <div style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '16px 24px',
          borderBottom: `1px solid ${T.borderLight}`,
        }}>
          <span style={{ fontSize: 18, fontWeight: 700, color: T.accent }}>
            {t('dashboard.title')}
          </span>
          <div style={{ display: 'flex', gap: 4 }}>
            <Button minimal small icon="import">{t('actions.importCharacter')}</Button>
            <Button minimal small icon="folder-open">{t('actions.openDocumentsFolder')}</Button>
            <Button minimal small icon="history">{t('actions.manageBackups')}</Button>
            <Button minimal small icon="cog">{t('navigation.settings')}</Button>
          </div>
        </div>

        {/* Save list */}
        <div style={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
          <SaveList
            saves={DUMMY_SAVES}
            selectedIndex={selectedIndex}
            onSelect={setSelectedIndex}
          />
        </div>
      </div>
    </div>
  );
}
