import { Icon } from '@blueprintjs/core';
import { T } from '../theme';

export function ClassesPanel() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
      <div style={{ textAlign: 'center' }}>
        <Icon icon="layers" size={40} style={{ color: T.border }} />
        <p style={{ marginTop: 12, fontSize: 14, color: T.textMuted }}>Classes - stub</p>
      </div>
    </div>
  );
}
