
import React, { memo, useState, useEffect, useRef } from 'react';
import { ScrollArea } from '@/components/ui/ScrollArea';
import { Button } from '@/components/ui/Button';
import { FeatCard } from './FeatCard';
import type { FeatInfo } from './types';
import type { FeatTab } from './FeatNavBar';
import { cn } from '@/lib/utils';

export interface FeatTabContentProps {
  activeTab: FeatTab;

  myFeats: FeatInfo[];
  allFeats: FeatInfo[];

  ownedFeatIds: Set<number>;
  protectedFeatIds: Set<number>;

  onAddFeat: (featId: number) => void;
  onRemoveFeat: (featId: number) => void;
  onLoadFeatDetails: (feat: FeatInfo) => Promise<FeatInfo | null>;

  currentPage: number;
  totalPages: number;
  hasNext: boolean;
  hasPrevious: boolean;
  onPageChange: (page: number) => void;

  removingFeatId?: number | null;
  addingFeatId?: number | null;
  addedFeatId?: number | null;
}

function AnimatedFeatCard({
  feat,
  removingFeatId,
  addedFeatId,
  ...cardProps
}: {
  feat: FeatInfo;
  removingFeatId?: number | null;
  addedFeatId?: number | null;
  isOwned: boolean;
  isProtected?: boolean;
  onRemove?: (featId: number) => void;
  onAdd?: (featId: number) => void;
  onLoadDetails: (feat: FeatInfo) => Promise<FeatInfo | null>;
}) {
  const isRemoving = removingFeatId === feat.id;
  const isAdding = addedFeatId === feat.id;
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
        <FeatCard feat={feat} {...cardProps} />
      </div>
    </div>
  );
}

function AvailableFeatCardWrapper({
  featId,
  addingFeatId,
  children,
}: {
  featId: number;
  addingFeatId?: number | null;
  children: React.ReactNode;
}) {
  const isLeaving = addingFeatId === featId;

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

function MyFeatsTabComponent({
  feats,
  protectedFeatIds,
  onRemove,
  onLoadDetails,
  removingFeatId,
  addedFeatId,
}: {
  feats: FeatInfo[];
  protectedFeatIds: Set<number>;
  onRemove: (featId: number) => void;
  onLoadDetails: (feat: FeatInfo) => Promise<FeatInfo | null>;
  removingFeatId?: number | null;
  addedFeatId?: number | null;
}) {
  if (feats.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-[rgb(var(--color-text-muted))]">
          No feats found. Try adjusting your filters.
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1">
      {feats.map((feat) => (
        <AnimatedFeatCard
          key={feat.id}
          feat={feat}
          removingFeatId={removingFeatId}
          addedFeatId={addedFeatId}
          isOwned={true}
          isProtected={protectedFeatIds.has(feat.id)}
          onRemove={onRemove}
          onLoadDetails={onLoadDetails}
        />
      ))}
    </div>
  );
}

const MyFeatsTab = memo(MyFeatsTabComponent);

function AvailableFeatsTabComponent({
  feats,
  ownedFeatIds,
  onAdd,
  onLoadDetails,
  addingFeatId,
}: {
  feats: FeatInfo[];
  ownedFeatIds: Set<number>;
  onAdd: (featId: number) => void;
  onLoadDetails: (feat: FeatInfo) => Promise<FeatInfo | null>;
  addingFeatId?: number | null;
}) {
  if (feats.length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <p className="text-[rgb(var(--color-text-muted))]">
          No available feats found. Try adjusting your filters.
        </p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1">
      {feats.map((feat) => (
        <AvailableFeatCardWrapper
          key={feat.id}
          featId={feat.id}
          addingFeatId={addingFeatId}
        >
          <FeatCard
            feat={feat}
            isOwned={ownedFeatIds.has(feat.id)}
            onAdd={onAdd}
            onLoadDetails={onLoadDetails}
          />
        </AvailableFeatCardWrapper>
      ))}
    </div>
  );
}

const AvailableFeatsTab = memo(AvailableFeatsTabComponent);

function FeatTabContentComponent({
  activeTab,
  myFeats,
  allFeats,
  ownedFeatIds,
  protectedFeatIds,
  onAddFeat,
  onRemoveFeat,
  onLoadFeatDetails,
  currentPage,
  totalPages,
  hasNext,
  hasPrevious,
  onPageChange,
  removingFeatId,
  addingFeatId,
  addedFeatId,
}: FeatTabContentProps) {
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
          <div className={cn('transition-opacity duration-200', activeTab === 'my-feats' ? 'block' : 'hidden')}>
            <MyFeatsTab
              feats={myFeats}
              protectedFeatIds={protectedFeatIds}
              onRemove={onRemoveFeat}
              onLoadDetails={onLoadFeatDetails}
              removingFeatId={removingFeatId}
              addedFeatId={addedFeatId}
            />
          </div>

          <div className={cn('transition-opacity duration-200', activeTab === 'all-feats' ? 'block' : 'hidden')}>
            <AvailableFeatsTab
              feats={allFeats}
              ownedFeatIds={ownedFeatIds}
              onAdd={onAddFeat}
              onLoadDetails={onLoadFeatDetails}
              addingFeatId={addingFeatId}
            />

          </div>


        </div>
      </ScrollArea>

      {activeTab === 'all-feats' && totalPages > 1 && (
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

export const FeatTabContent = memo(FeatTabContentComponent);
