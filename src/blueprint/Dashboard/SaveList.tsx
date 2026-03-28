import { Button, Icon } from '@blueprintjs/core';
import { useTranslations } from '@/hooks/useTranslations';
import { SectionBar } from '../shared';
import { T } from '../theme';
import { SaveEntry } from './SaveEntry';
import type { SaveEntryData } from './SaveEntry';

interface SaveListProps {
  saves: SaveEntryData[];
  selectedIndex: number | null;
  onSelect: (index: number) => void;
}

export function SaveList({ saves, selectedIndex, onSelect }: SaveListProps) {
  const t = useTranslations();

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <SectionBar title={t('dashboard.saveGames')} />

      <div style={{ flex: 1, overflowY: 'auto' }}>
        {saves.length > 0 ? (
          saves.map((save, i) => (
            <SaveEntry
              key={save.folderName}
              save={save}
              isSelected={selectedIndex === i}
              onClick={() => onSelect(i)}
            />
          ))
        ) : (
          <div style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            padding: '48px 16px',
            gap: 12,
          }}>
            <Icon icon="search" size={32} color={T.border} />
            <span style={{ fontSize: 13, color: T.textMuted }}>
              {t('dashboard.noSaves')}
            </span>
          </div>
        )}

        {/* Browse row at the end of the list */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: '14px 16px',
            borderBottom: `1px solid ${T.borderLight}`,
            cursor: 'pointer',
          }}
        >
          <Button minimal icon="folder-open" intent="primary">
            {t('dashboard.browse')}
          </Button>
        </div>
      </div>
    </div>
  );
}
