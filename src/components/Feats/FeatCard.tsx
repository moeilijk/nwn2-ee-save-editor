
import React, { memo, useState } from 'react';
import { ChevronDown, ChevronUp, Shield, Swords, Sparkles, Sun, Zap, Check, X, AlertCircle, Trash2 } from 'lucide-react';
import { Card } from '@/components/ui/Card';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import NWN2Icon from '@/components/ui/NWN2Icon';
import type { FeatInfo, Prerequisite, DetailedPrerequisites, BackendFeatPrerequisites } from './types';
import { cn } from '@/lib/utils';

export interface FeatCardProps {
  feat: FeatInfo;
  isOwned: boolean;
  onAdd?: (featId: number) => void;
  onRemove?: (featId: number) => void;
  onLoadDetails?: (feat: FeatInfo) => Promise<FeatInfo | null>;
  isProtected?: boolean;
}

interface CategoryBadgeInfo {
  label: string;
  icon: React.ReactNode;
  colorClass: string;
}

const CATEGORY_DEFS: Record<number, CategoryBadgeInfo> = {
  1:    { label: 'General',       icon: <Shield className="w-3 h-3" />,   colorClass: 'bg-blue-500' },
  2:    { label: 'Proficiency',   icon: <Swords className="w-3 h-3" />,   colorClass: 'bg-gray-500' },
  4:    { label: 'Skill/Save',    icon: <Shield className="w-3 h-3" />,   colorClass: 'bg-cyan-500' },
  8:    { label: 'Metamagic',     icon: <Sparkles className="w-3 h-3" />, colorClass: 'bg-purple-500' },
  16:   { label: 'Divine',        icon: <Sun className="w-3 h-3" />,      colorClass: 'bg-yellow-500' },
  32:   { label: 'Epic',          icon: <Zap className="w-3 h-3" />,      colorClass: 'bg-orange-500' },
  64:   { label: 'Class',         icon: <Shield className="w-3 h-3" />,   colorClass: 'bg-green-500' },
  128:  { label: 'Background',    icon: <Shield className="w-3 h-3" />,   colorClass: 'bg-teal-500' },
  256:  { label: 'Spellcasting',  icon: <Sparkles className="w-3 h-3" />, colorClass: 'bg-indigo-500' },
  512:  { label: 'History',       icon: <Shield className="w-3 h-3" />,   colorClass: 'bg-amber-500' },
  1024: { label: 'Heritage',      icon: <Shield className="w-3 h-3" />,   colorClass: 'bg-rose-500' },
  2048: { label: 'Item Creation', icon: <Sparkles className="w-3 h-3" />, colorClass: 'bg-lime-500' },
  4096: { label: 'Racial',        icon: <Shield className="w-3 h-3" />,   colorClass: 'bg-pink-500' },
  8192: { label: 'Domain',        icon: <Sun className="w-3 h-3" />,      colorClass: 'bg-yellow-600' },
};

const CATEGORY_BITS = Object.keys(CATEGORY_DEFS).map(Number).sort((a, b) => a - b);
const DEFAULT_BADGE: CategoryBadgeInfo = { label: 'General', icon: <Shield className="w-3 h-3" />, colorClass: 'bg-blue-500' };

function getFeatCategories(featType: number): CategoryBadgeInfo[] {
  const categories: CategoryBadgeInfo[] = [];
  for (const bit of CATEGORY_BITS) {
    if ((featType & bit) !== 0) {
      categories.push(CATEGORY_DEFS[bit]);
    }
  }
  return categories.length > 0 ? categories : [DEFAULT_BADGE];
}

interface ParsedDescription {
  type?: string;
  prerequisite?: string;
  requiredFor?: string;
  specifics?: string;
  use?: string;
  normal?: string;
  special?: string;
}

function stripHtmlTags(text: string): string {
  return text.replace(/<\/?[^>]+(>|$)/g, '');
}

function parseAndCleanDescription(rawDescription: string): ParsedDescription {
  const stripped = stripHtmlTags(rawDescription);

  const sections: ParsedDescription = {};
  const lines = stripped.split('\n').map(l => l.trim()).filter(l => l);

  for (const line of lines) {
    if (line.startsWith('Type of Feat:')) {
      sections.type = line.replace('Type of Feat:', '').trim();
    } else if (line.startsWith('Prerequisite:')) {
      sections.prerequisite = line.replace('Prerequisite:', '').trim();
    } else if (line.startsWith('Required for:')) {
      sections.requiredFor = line.replace('Required for:', '').trim();
    } else if (line.startsWith('Specifics:')) {
      sections.specifics = line.replace('Specifics:', '').trim();
    } else if (line.startsWith('Use:')) {
      sections.use = line.replace('Use:', '').trim();
    } else if (line.startsWith('Normal:')) {
      sections.normal = line.replace('Normal:', '').trim();
    } else if (line.startsWith('Special:')) {
      sections.special = line.replace('Special:', '').trim();
    }
  }

  return sections;
}

function PrerequisiteItem({ prereq }: { prereq: Prerequisite }) {
  const Icon = prereq.met ? Check : X;
  const colorClass = prereq.met ? 'text-[rgb(var(--color-success))]' : 'text-[rgb(var(--color-error))]';

  return (
    <div className="flex items-start gap-2 text-sm">
      <Icon className={cn('w-4 h-4 mt-0.5 flex-shrink-0', colorClass)} />
      <div className="flex-1">
        <span className={prereq.met ? 'text-[rgb(var(--color-text-secondary))]' : 'text-[rgb(var(--color-text-primary))]'}>
          {prereq.description}
        </span>
        {prereq.required_value !== undefined && prereq.current_value !== undefined && (
          <div className="mt-1">
            <div className="flex items-center gap-2">
              <div className="flex-1 bg-[rgb(var(--color-surface-3))] rounded-full h-1.5 overflow-hidden">
                <div
                  className={cn(
                    'h-full transition-all',
                    prereq.met ? 'bg-[rgb(var(--color-success))]' : 'bg-[rgb(var(--color-primary))]'
                  )}
                  style={{ width: `${Math.min((prereq.current_value / prereq.required_value) * 100, 100)}%` }}
                />
              </div>
              <span className="text-xs text-[rgb(var(--color-text-muted))]">
                {prereq.current_value}/{prereq.required_value}
              </span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function transformPrerequisites(prereqs: BackendFeatPrerequisites): DetailedPrerequisites {
  const requirements: Prerequisite[] = [];
  const met: string[] = [];
  const unmet: string[] = [];

  for (const f of prereqs.feats) {
    const desc = `Requires: ${f.name}`;
    requirements.push({ type: 'feat', description: desc, feat_id: f.id, met: f.met });
    (f.met ? met : unmet).push(desc);
  }

  for (const a of prereqs.abilities) {
    const desc = `${a.ability} ${a.required}+`;
    requirements.push({ type: 'ability', description: desc, required_value: a.required, current_value: a.current, met: a.met });
    (a.met ? met : unmet).push(desc);
  }

  if (prereqs.bab) {
    const desc = `Base Attack Bonus +${prereqs.bab.required}`;
    requirements.push({ type: 'bab', description: desc, required_value: prereqs.bab.required, current_value: prereqs.bab.current, met: prereqs.bab.met });
    (prereqs.bab.met ? met : unmet).push(desc);
  }

  if (prereqs.level) {
    const desc = `Character Level ${prereqs.level.required}`;
    requirements.push({ type: 'level', description: desc, required_value: prereqs.level.required, current_value: prereqs.level.current, met: prereqs.level.met });
    (prereqs.level.met ? met : unmet).push(desc);
  }

  if (prereqs.caster_level) {
    const desc = `Caster Level ${prereqs.caster_level.required}`;
    requirements.push({ type: 'level', description: desc, required_value: prereqs.caster_level.required, current_value: prereqs.caster_level.current, met: prereqs.caster_level.met });
    (prereqs.caster_level.met ? met : unmet).push(desc);
  }

  if (prereqs.spell_level) {
    const desc = `Spell Level ${prereqs.spell_level.required}`;
    requirements.push({ type: 'spell_level', description: desc, required_value: prereqs.spell_level.required, current_value: prereqs.spell_level.current, met: prereqs.spell_level.met });
    (prereqs.spell_level.met ? met : unmet).push(desc);
  }

  for (const s of prereqs.skills) {
    const desc = `${s.skill} ${s.required}+`;
    requirements.push({ type: 'ability', description: desc, required_value: s.required, current_value: s.current, met: s.met });
    (s.met ? met : unmet).push(desc);
  }

  return { requirements, met, unmet };
}

function FeatCardComponent({ feat, isOwned, onAdd, onRemove, onLoadDetails, isProtected = false }: FeatCardProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [detailedFeat, setDetailedFeat] = useState<FeatInfo | null>(null);
  const [isLoadingDetails, setIsLoadingDetails] = useState(false);

  const categories = getFeatCategories(feat.type ?? 1);

  const handleToggleExpand = async () => {
    if (!isExpanded && onLoadDetails && !detailedFeat) {
      setIsLoadingDetails(true);
      const details = await onLoadDetails(feat);
      setDetailedFeat(details);
      setIsLoadingDetails(false);
    }
    setIsExpanded(!isExpanded);
  };

  const displayFeat = detailedFeat || feat;

  const computedPrerequisites = React.useMemo(() => {
    if (displayFeat.detailed_prerequisites?.requirements?.length) {
      return displayFeat.detailed_prerequisites;
    }
    if (displayFeat.prerequisites) {
      const prereqs = displayFeat.prerequisites as BackendFeatPrerequisites;
      const hasAny = prereqs.feats?.length > 0
        || prereqs.abilities?.length > 0
        || prereqs.bab != null
        || prereqs.level != null
        || prereqs.caster_level != null
        || prereqs.spell_level != null
        || prereqs.skills?.length > 0;
      if (hasAny) return transformPrerequisites(prereqs);
    }
    return null;
  }, [displayFeat.detailed_prerequisites, displayFeat.prerequisites]);

  const hasPrerequisites = computedPrerequisites != null && computedPrerequisites.requirements.length > 0;

  return (
    <Card
      variant="interactive"
      className={cn(
        'transition-all duration-200',
        isOwned && 'border-[rgb(var(--color-primary)/0.3)]'
      )}
    >
      <div
        className="group/card flex items-start gap-3 cursor-pointer"
        onClick={handleToggleExpand}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleToggleExpand(); } }}
      >
        <div className="flex-shrink-0">
          <NWN2Icon icon={`ife_${feat.label?.toLowerCase() || ''}`} size="lg" />
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-start justify-between gap-2 mb-2">
            <div className="flex-1 min-w-0">
              <h3 className="text-base font-semibold text-[rgb(var(--color-text-primary))] truncate">
                {feat.name}
              </h3>
              <div className="flex items-center gap-2 mt-1 flex-wrap">
                {categories.map((cat) => (
                  <Badge key={cat.label} className={cn("w-[7.5rem] gap-1 text-white", cat.colorClass)}>
                    {cat.icon}
                    {cat.label}
                  </Badge>
                ))}
                {isProtected && (
                  <Badge variant="outline" className="w-[5.5rem] text-[rgb(var(--color-warning))]">
                    Protected
                  </Badge>
                )}
                {feat.custom && (
                  <Badge variant="secondary" className="w-[3.75rem]">Custom</Badge>
                )}
              </div>
            </div>

            <div className="flex items-center gap-2 flex-shrink-0">
              {!isOwned && onAdd && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={(e) => {
                    e.stopPropagation();
                    onAdd(feat.id);
                  }}
                  disabled={feat.can_take === false}
                >
                  Add
                </Button>
              )}
              {isOwned && onRemove && !isProtected && (
                <Button
                  variant="outline"
                  size="icon"
                  className="flex-shrink-0 w-8 h-8 border-red-500/50 text-red-400 hover:bg-red-500/10 hover:border-red-500/30 hover:text-red-500"
                  onClick={(e) => {
                    e.stopPropagation();
                    onRemove(feat.id);
                  }}
                >
                  <Trash2 className="w-4 h-4" />
                </Button>
              )}
              <div className="ml-2 pl-3 border-l border-[rgb(var(--color-surface-border))]">
                {isExpanded
                  ? <ChevronUp className="w-4 h-4 text-[rgb(var(--color-text-muted))] group-hover/card:text-[rgb(var(--color-primary))] transition-colors" />
                  : <ChevronDown className="w-4 h-4 text-[rgb(var(--color-text-muted))] group-hover/card:text-[rgb(var(--color-primary))] transition-colors" />
                }
              </div>
            </div>
          </div>

          {feat.can_take === false && feat.missing_requirements && feat.missing_requirements.length > 0 && !isExpanded && (
            <div className="mt-2 flex items-start gap-2 text-xs text-[rgb(var(--color-warning))]">
              <AlertCircle className="w-3 h-3 mt-0.5 flex-shrink-0" />
              <span>{feat.missing_requirements[0]}</span>
            </div>
          )}
        </div>
      </div>

      {isExpanded && (
        <div className="mt-4 pt-4 border-t border-[rgb(var(--color-surface-border))] space-y-4">
          {isLoadingDetails && (
            <div className="flex items-center justify-center py-4">
              <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-[rgb(var(--color-primary))]"></div>
            </div>
          )}

          {!isLoadingDetails && (
            <div className="px-4">
              {displayFeat.description && (() => {
                const parsed = parseAndCleanDescription(displayFeat.description);
                const hasStructuredSections = !!(
                  parsed.specifics ||
                  parsed.requiredFor ||
                  parsed.use ||
                  parsed.normal ||
                  parsed.special
                );

                if (hasStructuredSections) {
                  return (
                    <div className="space-y-3">
                      {parsed.specifics && (
                        <div>
                          <h4 className="text-sm font-semibold text-[rgb(var(--color-text-primary))] mb-1">
                            Effect
                          </h4>
                          <p className="text-sm text-[rgb(var(--color-text-secondary))] leading-relaxed">
                            {parsed.specifics}
                          </p>
                        </div>
                      )}

                      {parsed.requiredFor && (
                        <div>
                          <h4 className="text-sm font-semibold text-[rgb(var(--color-text-primary))] mb-1">
                            Required For
                          </h4>
                          <p className="text-sm text-[rgb(var(--color-text-secondary))]">
                            {parsed.requiredFor}
                          </p>
                        </div>
                      )}

                      {parsed.use && (
                        <div>
                          <h4 className="text-sm font-semibold text-[rgb(var(--color-text-primary))] mb-1">
                            Usage
                          </h4>
                          <p className="text-sm text-[rgb(var(--color-text-secondary))]">
                            {parsed.use}
                          </p>
                        </div>
                      )}

                      {parsed.normal && (
                        <div>
                          <h4 className="text-sm font-semibold text-[rgb(var(--color-text-primary))] mb-1">
                            Normal
                          </h4>
                          <p className="text-sm text-[rgb(var(--color-text-secondary))]">
                            {parsed.normal}
                          </p>
                        </div>
                      )}

                      {parsed.special && (
                        <div>
                          <h4 className="text-sm font-semibold text-[rgb(var(--color-text-primary))] mb-1">
                            Special
                          </h4>
                          <p className="text-sm text-[rgb(var(--color-text-secondary))]">
                            {parsed.special}
                          </p>
                        </div>
                      )}
                    </div>
                  );
                } else {
                  return (
                    <div>
                      <h4 className="text-sm font-semibold text-[rgb(var(--color-text-primary))] mb-2">
                        Description
                      </h4>
                      <p className="text-sm text-[rgb(var(--color-text-secondary))] leading-relaxed whitespace-pre-wrap">
                        {stripHtmlTags(displayFeat.description)}
                      </p>
                    </div>
                  );
                }
              })()}

              {hasPrerequisites && computedPrerequisites && (
                <div>
                  <h4 className="text-sm font-semibold text-[rgb(var(--color-text-primary))] mb-2">
                    Prerequisites
                  </h4>
                  <div className="space-y-2">
                    {computedPrerequisites.requirements.map((prereq, idx) => (
                      <PrerequisiteItem key={idx} prereq={prereq} />
                    ))}
                  </div>
                </div>
              )}

              {displayFeat.can_take === false && displayFeat.missing_requirements && displayFeat.missing_requirements.length > 0 && (
                <div>
                  <h4 className="text-sm font-semibold text-[rgb(var(--color-error))] mb-2">
                    Missing Requirements
                  </h4>
                  <ul className="space-y-1">
                    {displayFeat.missing_requirements.map((req, idx) => (
                      <li key={idx} className="text-sm text-[rgb(var(--color-text-secondary))] flex items-start gap-2">
                        <X className="w-4 h-4 text-[rgb(var(--color-error))] flex-shrink-0 mt-0.5" />
                        {req}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </Card>
  );
}

export const FeatCard = memo(FeatCardComponent);
