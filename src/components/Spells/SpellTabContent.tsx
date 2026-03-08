
import React, { memo, useState, useEffect, useRef } from 'react';
import { ScrollArea } from '@/components/ui/ScrollArea';
import { Button } from '@/components/ui/Button';
import { SpellCard } from './SpellCard';
import type { SpellInfo } from './types';
import type { SpellTab } from './SpellNavBar';
import { cn } from '@/lib/utils';

export interface SpellTabContentProps {
  activeTab: SpellTab;

  mySpells: SpellInfo[];
  preparedSpells: SpellInfo[];
  allSpells: SpellInfo[];

  ownedSpellIds: Set<number>;

  onAddSpell: (spellId: number, classIndex: number, spellLevel: number) => void;
  onRemoveSpell: (spellId: number, classIndex: number, spellLevel: number) => void;
  onLoadSpellDetails?: (spell: SpellInfo) => Promise<SpellInfo | null>;

  currentPage: number;
  totalPages: number;
  hasNext: boolean;
  hasPrevious: boolean;
  onPageChange: (page: number) => void;

  casterClasses: Array<{index: number; name: string; class_id: number; can_edit_spells: boolean}>;

  removingSpellKey?: string | null;
  addingSpellKey?: string | null;
  addedSpellKey?: string | null;
}

function AnimatedSpellCard({
  spell,
  spellKey,
  removingSpellKey,
  addedSpellKey,
  ...cardProps
}: {
  spell: SpellInfo;
  spellKey: string;
  removingSpellKey?: string | null;
  addedSpellKey?: string | null;
  isOwned: boolean;
  onRemove?: (spellId: number, classIndex: number, spellLevel: number) => void;
  onLoadDetails?: (spell: SpellInfo) => Promise<SpellInfo | null>;
  casterClasses: Array<{index: number; name: string; class_id: number; can_edit_spells: boolean}>;
}) {
  const isRemoving = removingSpellKey === spellKey;
  const isAdding = addedSpellKey === spellKey;
  const [mounted, setMounted] = useState(!isAdding);

  useEffect(() => {
    if (isAdding && !mounted) {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          setMounted(true);
        });
      });
    }
  }, [isAdding, mounted]);

  return (
    <div
      className={cn(
        'grid transition-all duration-[180ms]',
        isRemoving
          ? 'grid-rows-[0fr] opacity-0 -translate-x-10 mb-0 ease-in'
          : (isAdding && !mounted)
            ? 'grid-rows-[0fr] opacity-0 translate-x-10 mb-0 ease-in'
            : 'grid-rows-[1fr] opacity-100 translate-x-0 mb-3 ease-out'
      )}
    >
      <div className="overflow-hidden">
        <SpellCard spell={spell} {...cardProps} />
      </div>
    </div>
  );
}

function MySpellsTabComponent({
  spells,
  onRemove,
  onLoadDetails,
  casterClasses,
  removingSpellKey,
  addedSpellKey,
}: {
  spells: SpellInfo[];
  onRemove: (spellId: number, classIndex: number, spellLevel: number) => void;
  onLoadDetails?: (spell: SpellInfo) => Promise<SpellInfo | null>;
  casterClasses: Array<{index: number; name: string; class_id: number; can_edit_spells: boolean}>;
  removingSpellKey?: string | null;
  addedSpellKey?: string | null;
}) {
  if (spells.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-[rgb(var(--color-text-muted))]">
          No spells found. Try adjusting your filters.
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1">
      {spells.map((spell, index) => {
        const spellKey = `${spell.id}-${spell.level}`;
        return (
          <AnimatedSpellCard
            key={`${spell.id}-${spell.level}-${index}`}
            spell={spell}
            spellKey={spellKey}
            removingSpellKey={removingSpellKey}
            addedSpellKey={addedSpellKey}
            isOwned={true}
            onRemove={onRemove}
            onLoadDetails={onLoadDetails}
            casterClasses={casterClasses}
          />
        );
      })}
    </div>
  );
}

const MySpellsTab = memo(MySpellsTabComponent);

function AvailableSpellCardWrapper({
  spellKey,
  addingSpellKey,
  children,
}: {
  spellKey: string;
  addingSpellKey?: string | null;
  children: React.ReactNode;
}) {
  const isLeaving = addingSpellKey === spellKey;

  return (
    <div
      className={cn(
        'grid transition-all duration-[180ms]',
        isLeaving
          ? 'grid-rows-[0fr] opacity-0 translate-x-10 mb-0 ease-in'
          : 'grid-rows-[1fr] opacity-100 translate-x-0 mb-3 ease-out'
      )}
    >
      <div className="overflow-hidden">
        {children}
      </div>
    </div>
  );
}

function AvailableSpellsTabComponent({
  spells,
  ownedSpellIds,
  onAdd,
  onLoadDetails,
  casterClasses,
  addingSpellKey,
}: {
  spells: SpellInfo[];
  ownedSpellIds: Set<number>;
  onAdd: (spellId: number, classIndex: number, spellLevel: number) => void;
  onLoadDetails?: (spell: SpellInfo) => Promise<SpellInfo | null>;
  casterClasses: Array<{index: number; name: string; class_id: number; can_edit_spells: boolean}>;
  addingSpellKey?: string | null;
}) {
  if (spells.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-[rgb(var(--color-text-muted))]">
          No available spells found. Try adjusting your filters.
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1">
      {spells.map((spell, index) => {
        const spellKey = `${spell.id}-${spell.level}`;
        return (
          <AvailableSpellCardWrapper
            key={`${spell.id}-${spell.level}-${index}`}
            spellKey={spellKey}
            addingSpellKey={addingSpellKey}
          >
            <SpellCard
              spell={spell}
              isOwned={ownedSpellIds.has(spell.id)}
              onAdd={onAdd}
              onLoadDetails={onLoadDetails}
              casterClasses={casterClasses}
            />
          </AvailableSpellCardWrapper>
        );
      })}
    </div>
  );
}

const AvailableSpellsTab = memo(AvailableSpellsTabComponent);

function SpellTabContentComponent({
  activeTab,
  mySpells,
  preparedSpells,
  allSpells,
  ownedSpellIds,
  onAddSpell,
  onRemoveSpell,
  onLoadSpellDetails,
  currentPage,
  totalPages,
  hasNext,
  hasPrevious,
  onPageChange,
  casterClasses,
  removingSpellKey,
  addingSpellKey,
  addedSpellKey,
}: SpellTabContentProps) {
  const topAnchorRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let el = topAnchorRef.current?.parentElement;
    while (el) {
      el.scrollTop = 0;
      el = el.parentElement;
    }
  }, [currentPage]);

  return (
    <div className="flex flex-col flex-1 bg-[rgb(var(--color-surface-1))] border border-[rgb(var(--color-surface-border))] rounded-lg overflow-hidden">
      <ScrollArea className="flex-1">
        <div ref={topAnchorRef} />
        <div className="p-4">
          <div className={cn('transition-opacity duration-200', activeTab === 'my-spells' ? 'block' : 'hidden')}>
            <MySpellsTab
              spells={mySpells}
              onRemove={onRemoveSpell}
              onLoadDetails={onLoadSpellDetails}
              casterClasses={casterClasses}
              removingSpellKey={removingSpellKey}
              addedSpellKey={addedSpellKey}
            />
          </div>

          <div className={cn('transition-opacity duration-200', activeTab === 'prepared' ? 'block' : 'hidden')}>
            <MySpellsTab
              spells={preparedSpells}
              onRemove={onRemoveSpell}
              onLoadDetails={onLoadSpellDetails}
              casterClasses={casterClasses}
              removingSpellKey={removingSpellKey}
              addedSpellKey={addedSpellKey}
            />
          </div>

          <div className={cn('transition-opacity duration-200', activeTab === 'all-spells' ? 'block' : 'hidden')}>
            <AvailableSpellsTab
              spells={allSpells}
              ownedSpellIds={ownedSpellIds}
              onAdd={onAddSpell}
              onLoadDetails={onLoadSpellDetails}
              casterClasses={casterClasses}
              addingSpellKey={addingSpellKey}
            />
          </div>
        </div>
      </ScrollArea>

      {activeTab === 'all-spells' && totalPages > 1 && (
        <div className="border-t border-[rgb(var(--color-surface-border))] bg-[rgb(var(--color-surface-1))] px-4 py-3">
          <div className="flex items-center justify-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => onPageChange(currentPage - 1)}
              disabled={!hasPrevious}
            >
              Previous
            </Button>
            <span className="text-sm text-[rgb(var(--color-text-primary))] px-2">
              Page {currentPage} of {totalPages}
            </span>
            <Button
              variant="outline"
              size="sm"
              onClick={() => onPageChange(currentPage + 1)}
              disabled={!hasNext}
            >
              Next
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}

export const SpellTabContent = memo(SpellTabContentComponent);
