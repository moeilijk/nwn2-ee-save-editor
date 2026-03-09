import { Icon } from '@blueprintjs/core';
import { T } from '../theme';

export function SpellsPanel() {
  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
      <div style={{ textAlign: 'center' }}>
        <Icon icon="flash" size={40} style={{ color: T.border }} />
        <p style={{ marginTop: 12, fontSize: 14, color: T.textMuted }}>Spells - stub</p>
      </div>
    </div>
  );
}
