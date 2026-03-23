import { Card, Elevation } from '@blueprintjs/core';
import { T } from '../theme';

interface SplitPaneProps {
  toolbar: React.ReactNode;
  list: React.ReactNode;
  detail: React.ReactNode;
  listWidth?: number;
}

export function SplitPane({ toolbar, list, detail, listWidth = 320 }: SplitPaneProps) {
  return (
    <div style={{ padding: 16, height: '100%' }}>
      <Card elevation={Elevation.ONE} style={{ padding: 0, background: T.surface, overflow: 'hidden', display: 'flex', flexDirection: 'column', height: '100%' }}>
        <div style={{ padding: '6px 16px', borderBottom: `1px solid ${T.borderLight}`, display: 'flex', alignItems: 'center', gap: 8, flexShrink: 0 }}>
          {toolbar}
        </div>
        <div style={{ display: 'flex', flex: 1, minHeight: 0 }}>
          <div style={{ width: listWidth, borderRight: `1px solid ${T.borderLight}`, overflow: 'auto' }}>
            {list}
          </div>
          <div style={{ flex: 1, overflow: 'auto' }}>
            {detail}
          </div>
        </div>
      </Card>
    </div>
  );
}
