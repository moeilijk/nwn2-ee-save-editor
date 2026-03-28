import { useState } from 'react';
import { Button } from '@blueprintjs/core';
import { useTranslations } from '@/hooks/useTranslations';
import { T, PATTERN_BG } from '../theme';
import '../blueprint.css';
import { SaveList } from './SaveList';
import type { SaveEntryData } from './SaveEntry';
import { FileBrowserDialog } from './FileBrowserDialog';
import type { FileInfo } from './FileBrowserDialog';

const now = Date.now() / 1000;

const DEMO_SAVES: FileInfo[] = [
  { name: '000000 - Chapter Act I', path: 'C:/NWN2/saves/000000', size: 4_812_000, modified: String(now - 3600), is_directory: true, save_name: 'Chapter Act I', character_name: 'Khelgar Ironfist' },
  { name: '000001 - Crossroad Keep', path: 'C:/NWN2/saves/000001', size: 5_230_000, modified: String(now - 86400), is_directory: true, save_name: 'Crossroad Keep', character_name: 'Ammon Jerro' },
  { name: '000002 - Highcliff', path: 'C:/NWN2/saves/000002', size: 3_145_000, modified: String(now - 172800), is_directory: true, save_name: 'Highcliff', character_name: 'Neeshka' },
  { name: '000003 - Old Owl Well', path: 'C:/NWN2/saves/000003', size: 6_780_000, modified: String(now - 259200), is_directory: true, save_name: 'Old Owl Well', character_name: 'Khelgar Ironfist' },
  { name: '000004 - Neverwinter Docks', path: 'C:/NWN2/saves/000004', size: 4_100_000, modified: String(now - 345600), is_directory: true, save_name: 'Neverwinter Docks', character_name: 'Elanee' },
  { name: '000005 - West Harbor', path: 'C:/NWN2/saves/000005', size: 2_890_000, modified: String(now - 432000), is_directory: true, save_name: 'West Harbor', character_name: 'Casavir' },
  { name: '000006 - Githyanki Caves', path: 'C:/NWN2/saves/000006', size: 7_200_000, modified: String(now - 518400), is_directory: true, save_name: 'Githyanki Caves', character_name: 'Ammon Jerro' },
  { name: '000007 - Merdelain', path: 'C:/NWN2/saves/000007', size: 8_410_000, modified: String(now - 604800), is_directory: true, save_name: 'Merdelain', character_name: 'Zhjaeve' },
];

const DEMO_BACKUPS: FileInfo[] = [
  { name: 'backup_2026-03-27_18-45', path: 'C:/NWN2/saves/backups/backup_001', size: 4_812_000, modified: String(now - 7200), is_directory: true, save_name: 'Backup of Chapter Act I', character_name: 'Khelgar Ironfist' },
  { name: 'backup_2026-03-26_14-20', path: 'C:/NWN2/saves/backups/backup_002', size: 5_230_000, modified: String(now - 93600), is_directory: true, save_name: 'Backup of Crossroad Keep', character_name: 'Ammon Jerro' },
  { name: 'pre_restore_2026-03-25', path: 'C:/NWN2/saves/backups/backup_003', size: 3_145_000, modified: String(now - 180000), is_directory: true, save_name: 'Pre-restore backup', character_name: 'Neeshka' },
  { name: 'backup_2026-03-24_09-10', path: 'C:/NWN2/saves/backups/backup_004', size: 6_100_000, modified: String(now - 266400), is_directory: true, save_name: 'Backup of Old Owl Well', character_name: 'Khelgar Ironfist' },
];

const DEMO_VAULT: FileInfo[] = [
  { name: 'Khelgar Ironfist', path: 'C:/NWN2/localvault/khelgar.bic', size: 128_000, modified: String(now - 3600), is_directory: false, display_name: 'Khelgar Ironfist' },
  { name: 'Ammon Jerro', path: 'C:/NWN2/localvault/ammon.bic', size: 156_000, modified: String(now - 86400), is_directory: false, display_name: 'Ammon Jerro' },
  { name: 'Neeshka', path: 'C:/NWN2/localvault/neeshka.bic', size: 142_000, modified: String(now - 172800), is_directory: false, display_name: 'Neeshka' },
  { name: 'Elanee', path: 'C:/NWN2/localvault/elanee.bic', size: 134_000, modified: String(now - 259200), is_directory: false, display_name: 'Elanee' },
  { name: 'Casavir', path: 'C:/NWN2/localvault/casavir.bic', size: 118_000, modified: String(now - 345600), is_directory: false, display_name: 'Casavir' },
];

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

  const [showVaultBrowser, setShowVaultBrowser] = useState(false);
  const [showBackupBrowser, setShowBackupBrowser] = useState(false);
  const [backupPath, setBackupPath] = useState('');
  const [backupRefreshKey, setBackupRefreshKey] = useState(0);

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
            <Button minimal small icon="import" onClick={() => setShowVaultBrowser(true)}>{t('actions.importCharacter')}</Button>
            <Button minimal small icon="folder-open">{t('actions.openDocumentsFolder')}</Button>
            <Button minimal small icon="history" onClick={() => setShowBackupBrowser(true)}>{t('actions.manageBackups')}</Button>
            <Button minimal small icon="cog">{t('navigation.settings')}</Button>
          </div>
        </div>

        {/* Save list */}
        <div style={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
          <SaveList
            saves={DUMMY_SAVES}
            selectedIndex={selectedIndex}
            onSelect={setSelectedIndex}
            demoFiles={DEMO_SAVES}
          />
        </div>
      </div>

      <FileBrowserDialog
        isOpen={showVaultBrowser}
        onClose={() => setShowVaultBrowser(false)}
        mode="load-vault"
        currentPath="C:/Users/Player/Documents/Neverwinter Nights 2/localvault"
        demoFiles={DEMO_VAULT}
        onSelectFile={(file) => {
          console.log('Import vault character:', file.path);
          setShowVaultBrowser(false);
        }}
      />

      <FileBrowserDialog
        isOpen={showBackupBrowser}
        onClose={() => setShowBackupBrowser(false)}
        mode="manage-backups"
        currentPath={backupPath || 'C:/Users/Player/Documents/Neverwinter Nights 2/saves/backups'}
        onPathChange={setBackupPath}
        refreshKey={backupRefreshKey}
        canRestore
        demoFiles={DEMO_BACKUPS}
        onSelectFile={(file) => {
          console.log('Restore backup:', file.path);
          setShowBackupBrowser(false);
        }}
        onDeleteBackup={async (file) => {
          console.log('Delete backup:', file.path);
          setBackupRefreshKey(prev => prev + 1);
        }}
      />
    </div>
  );
}
