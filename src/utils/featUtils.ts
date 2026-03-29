import type { FeatInfo } from '@/components/Feats/types';

interface FeatsSummary {
  protected?: FeatInfo[];
  class_feats?: FeatInfo[];
  general_feats?: FeatInfo[];
  custom_feats?: FeatInfo[];
  background_feats?: FeatInfo[];
  domain_feats?: FeatInfo[];
}

export function aggregateFeats(summary: FeatsSummary | null | undefined): FeatInfo[] {
  if (!summary) return [];

  const allFeats = [
    ...(summary.protected || []),
    ...(summary.class_feats || []),
    ...(summary.general_feats || []),
    ...(summary.custom_feats || []),
    ...(summary.background_feats || []),
    ...(summary.domain_feats || []),
  ];

  const unique = new Map<number, FeatInfo>();
  for (const feat of allFeats) {
    unique.set(feat.id, feat);
  }
  return Array.from(unique.values());
}

export function filterFeatsByType(feats: FeatInfo[], selectedTypes: Set<number>): FeatInfo[] {
  if (selectedTypes.size === 0) return feats;
  return feats.filter(feat =>
    Array.from(selectedTypes).some(type => (feat.type & type) !== 0)
  );
}

export function sortFeats(feats: FeatInfo[], sortBy: string): FeatInfo[] {
  return [...feats].sort((a, b) => {
    switch (sortBy) {
      case 'name': return a.name.localeCompare(b.name);
      case 'type': return a.type - b.type;
      default: return 0;
    }
  });
}

export function filterOwnedFeats(feats: FeatInfo[], ownedIds: Set<number>): FeatInfo[] {
  return feats.filter(feat => !ownedIds.has(feat.id));
}
