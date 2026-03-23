import { useState } from 'react';
import { T } from '../theme';

export interface ListSection<T> {
  key: string;
  title: string;
  items: T[];
}

interface GroupedListProps<T extends { id: number }> {
  sections: ListSection<T>[];
  selectedId: number | null;
  onSelect: (item: T) => void;
  renderItem: (item: T, selected: boolean) => React.ReactNode;
}

export function GroupedList<T extends { id: number }>({ sections, selectedId, onSelect, renderItem }: GroupedListProps<T>) {
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());

  const toggleSection = (key: string) => {
    setCollapsed(prev => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key); else next.add(key);
      return next;
    });
  };

  return (
    <div style={{ paddingTop: 4 }}>
      {sections.map(section => {
        const isCollapsed = collapsed.has(section.key);

        return (
          <div key={section.key}>
            <div
              onClick={() => toggleSection(section.key)}
              style={{
                display: 'flex', alignItems: 'center', gap: 6,
                padding: '6px 12px',
                cursor: 'pointer',
                userSelect: 'none',
                background: T.sectionBg,
                borderBottom: `1px solid ${T.sectionBorder}`,
              }}
            >
              <span style={{ color: T.accent, width: 10 }}>
                {isCollapsed ? '\u25B6' : '\u25BC'}
              </span>
              <span style={{ fontWeight: 700, color: T.accent, flex: 1 }}>
                {section.title}
              </span>
              <span style={{ color: T.textMuted }}>{section.items.length}</span>
            </div>

            {!isCollapsed && section.items.map(item => {
              const selected = selectedId === item.id;

              return (
                <div
                  key={item.id}
                  onClick={() => onSelect(item)}
                  style={{
                    padding: '5px 12px 5px 28px',
                    cursor: 'pointer',
                    borderBottom: `1px solid ${T.borderLight}`,
                    background: selected ? `${T.accent}12` : 'transparent',
                    borderLeft: selected ? `2px solid ${T.accent}` : '2px solid transparent',
                    transition: 'background 0.1s',
                  }}
                >
                  {renderItem(item, selected)}
                </div>
              );
            })}
          </div>
        );
      })}
    </div>
  );
}
