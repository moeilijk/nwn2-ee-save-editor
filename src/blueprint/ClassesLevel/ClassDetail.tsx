import { Tag, Icon } from '@blueprintjs/core';
import { T } from '../theme';
import { display } from '@/utils/dataHelpers';
import type { ClassInfo } from '@/hooks/useClassesLevel';
import { useTranslations } from '@/hooks/useTranslations';

interface ClassDetailProps {
  cls: ClassInfo | null;
  canSelect: boolean;
  selectReason?: string;
}

export function ClassDetail({ cls, canSelect, selectReason }: ClassDetailProps) {
  const t = useTranslations();

  if (!cls) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: T.textMuted }}>
        {t('classes.selectClassToView')}
      </div>
    );
  }

  return (
    <div style={{ padding: '16px 20px', display: 'flex', flexDirection: 'column', gap: 12 }}>
      <div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
          <span style={{ fontWeight: 700, fontSize: 15, color: T.text }}>{display(cls.name)}</span>
          <Tag minimal round intent={cls.type === 'prestige' ? 'warning' : 'primary'} style={{ fontSize: 10 }}>
            {cls.type === 'prestige' ? 'Prestige' : 'Base'}
          </Tag>
        </div>
        {!canSelect && selectReason && (
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: T.negative }}>
            <Icon icon="warning-sign" size={12} />
            {selectReason}
          </div>
        )}
      </div>

      <div style={{ color: T.textMuted, fontSize: 12, fontStyle: 'italic', padding: '12px 0' }}>
        Class details coming soon.
      </div>
    </div>
  );
}
