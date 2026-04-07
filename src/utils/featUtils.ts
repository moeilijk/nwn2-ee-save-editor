import type { FeatInfo } from '@/blueprint/Feats/types';
import { FEAT_TYPES } from '@/blueprint/Feats/types';

export const FEAT_TYPE_LABELS: Record<number, string> = {
  [FEAT_TYPES.GENERAL]: 'General',
  [FEAT_TYPES.PROFICIENCY]: 'Proficiency',
  [FEAT_TYPES.SKILL_SAVE]: 'Skill/Save',
  [FEAT_TYPES.METAMAGIC]: 'Metamagic',
  [FEAT_TYPES.DIVINE]: 'Divine',
  [FEAT_TYPES.EPIC]: 'Epic',
  [FEAT_TYPES.CLASS]: 'Class',
  [FEAT_TYPES.BACKGROUND]: 'Background',
  [FEAT_TYPES.SPELLCASTING]: 'Spellcasting',
  [FEAT_TYPES.HISTORY]: 'History',
  [FEAT_TYPES.HERITAGE]: 'Heritage',
  [FEAT_TYPES.ITEM_CREATION]: 'Item Creation',
  [FEAT_TYPES.RACIAL]: 'Racial',
  [FEAT_TYPES.DOMAIN]: 'Domain',
};

export function getFeatTypeLabel(type: number): string {
  const match = Object.entries(FEAT_TYPE_LABELS).find(([bit]) => (type & Number(bit)) !== 0);
  return match ? match[1] : 'General';
}

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
