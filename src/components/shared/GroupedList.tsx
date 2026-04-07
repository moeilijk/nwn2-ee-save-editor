import { useState, useMemo, useCallback, useRef, useEffect } from 'react';
import { VariableSizeList } from 'react-window';
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

type FlatRow<T> =
  | { type: 'header'; sectionKey: string; title: string; count: number }
  | { type: 'item'; item: T };

const HEADER_HEIGHT = 32;
const ITEM_HEIGHT = 31;

export function GroupedList<T extends { id: number }>({ sections, selectedId, onSelect, renderItem }: GroupedListProps<T>) {
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [containerHeight, setContainerHeight] = useState(400);
  const containerRef = useRef<HTMLDivElement>(null);
  const listRef = useRef<VariableSizeList>(null);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const observer = new ResizeObserver(entries => {
      for (const entry of entries) {
        setContainerHeight(entry.contentRect.height);
      }
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const toggleSection = useCallback((key: string) => {
    setCollapsed(prev => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key); else next.add(key);
      return next;
    });
  }, []);

  // Reset list cache when collapsed state or sections change
  useEffect(() => {
    listRef.current?.resetAfterIndex(0);
  }, [collapsed, sections]);

  const flatRows: FlatRow<T>[] = useMemo(() => {
    const rows: FlatRow<T>[] = [];
    for (const section of sections) {
      rows.push({ type: 'header', sectionKey: section.key, title: section.title, count: section.items.length });
      if (!collapsed.has(section.key)) {
        for (const item of section.items) {
          rows.push({ type: 'item', item });
        }
      }
    }
    return rows;
  }, [sections, collapsed]);

  const getItemSize = useCallback((index: number) => {
    return flatRows[index].type === 'header' ? HEADER_HEIGHT : ITEM_HEIGHT;
  }, [flatRows]);

  const Row = useCallback(({ index, style }: { index: number; style: React.CSSProperties }) => {
    const row = flatRows[index];
    if (row.type === 'header') {
      const isCollapsed = collapsed.has(row.sectionKey);
      return (
        <div
          style={{ ...style, display: 'flex', alignItems: 'center', gap: 6, padding: '0 12px', cursor: 'pointer', userSelect: 'none', background: T.sectionBg, borderBottom: `1px solid ${T.sectionBorder}` }}
          onClick={() => toggleSection(row.sectionKey)}
        >
          <span style={{ color: T.accent, width: 10 }}>
            {isCollapsed ? '\u25B6' : '\u25BC'}
          </span>
          <span style={{ fontWeight: 700, color: T.accent, flex: 1 }}>
            {row.title}
          </span>
          <span style={{ color: T.textMuted }}>{row.count}</span>
        </div>
      );
    }

    const selected = selectedId === row.item.id;
    return (
      <div
        style={{
          ...style,
          padding: '0 12px 0 28px',
          display: 'flex',
          alignItems: 'center',
          cursor: 'pointer',
          borderBottom: `1px solid ${T.borderLight}`,
          background: selected ? `${T.accent}12` : 'transparent',
          borderLeft: selected ? `2px solid ${T.accent}` : '2px solid transparent',
        }}
        onClick={() => onSelect(row.item)}
      >
        {renderItem(row.item, selected)}
      </div>
    );
  }, [flatRows, collapsed, selectedId, onSelect, renderItem, toggleSection]);

  return (
    <div ref={containerRef} style={{ height: '100%' }}>
      <VariableSizeList
        ref={listRef}
        height={containerHeight}
        width="100%"
        itemCount={flatRows.length}
        itemSize={getItemSize}
        overscanCount={20}
      >
        {Row}
      </VariableSizeList>
    </div>
  );
}
