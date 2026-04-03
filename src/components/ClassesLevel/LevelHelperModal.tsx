import { useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { X, AlertCircle, AlertTriangle, ArrowRight, Zap, Award, Sparkles, ChevronDown, ChevronUp } from 'lucide-react';
import { useSubsystem } from '@/contexts/CharacterContext';
import { useTranslations } from '@/hooks/useTranslations';
import { cn } from '@/lib/utils';

interface LevelHelperModalProps {
  isOpen: boolean;
  onClose: () => void;
  className: string;
  onNavigate?: (path: string) => void;
}

export default function LevelHelperModal({ isOpen, onClose, className, onNavigate }: LevelHelperModalProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isVisible, setIsVisible] = useState(false);
  const [spellsExpanded, setSpellsExpanded] = useState(false);
  const t = useTranslations();

  // Get live data from subsystems
  const skillsSubsystem = useSubsystem('skills');
  const abilityScoresSubsystem = useSubsystem('abilityScores');
  const featsSubsystem = useSubsystem('feats');
  const spellsSubsystem = useSubsystem('spells');

  // Calculate available points from subsystem data
  const skillPoints = (() => {
    const data = skillsSubsystem.data;
    if (!data) return 0;
    return Math.max(0, data.available_points ?? 0);
  })();

  const abilityPoints = (() => {
    const data = abilityScoresSubsystem.data as { point_summary?: { available: number } } | null;
    if (!data?.point_summary) return 0;
    return data.point_summary.available;
  })();

  // Get feat slots from feats subsystem
  const featSlots = (() => {
    const data = featsSubsystem.data as { feat_slots?: { open_slots?: number } } | null;
    if (!data?.feat_slots) return 0;
    return data.feat_slots.open_slots ?? 0;
  })();

  // Get pending spells to learn - only for spontaneous casters and wizards
  const spellData = (() => {
    interface PendingSpellLearning {
      class_id: { inner: number };
      class_name: string;
      caster_type: string;
      by_level: Record<string, number>;
      total: number;
    }
    interface SpellSubsystemData {
      pending_spell_learning?: PendingSpellLearning[];
    }
    const data = spellsSubsystem.data as SpellSubsystemData | null;
    if (!data?.pending_spell_learning?.length) return null;

    const classes: Array<{
      name: string;
      casterType: string;
      byLevel: Record<number, number>;
      total: number;
    }> = [];
    let grandTotal = 0;

    data.pending_spell_learning.forEach((cls: PendingSpellLearning) => {
      if (cls.total > 0) {
        const byLevel: Record<number, number> = {};
        if (cls.by_level) {
          Object.entries(cls.by_level).forEach(([level, count]) => {
            byLevel[parseInt(level)] = count;
          });
        }
        classes.push({
          name: cls.class_name,
          casterType: cls.caster_type,
          byLevel,
          total: cls.total,
        });
        grandTotal += cls.total;
      }
    });

    return classes.length > 0 ? { classes, total: grandTotal } : null;
  })();

  const hasPendingSpells = spellData !== null && spellData.total > 0;
  const hasPendingGains = skillPoints > 0 || abilityPoints > 0 || featSlots > 0 || hasPendingSpells;

  useEffect(() => {
    if (isOpen) {
      setIsVisible(true);
      setIsExpanded(false);
    } else {
      setIsVisible(false);
    }
  }, [isOpen]);

  if (!isOpen && !isVisible) return null;

  const handleNavigate = (path: string) => {
    if (onNavigate) {
      onNavigate(path);
    }
  };

  const portalRoot = document.getElementById('portal-root') || document.body;

  return createPortal(
    <div className={cn(
      "fixed bottom-6 right-6 z-50 flex flex-col items-end gap-3",
      className
    )}>

      {/* Expanded Content Card */}
      <div className={cn(
        "bg-[rgb(var(--color-surface-1))] rounded-lg overflow-hidden transition-[height,opacity,transform,margin] duration-300 origin-bottom-right",
        isExpanded
          ? "opacity-100 scale-100 translate-y-0 w-80 mb-2 border border-[rgb(var(--color-border))] shadow-2xl"
          : "opacity-0 scale-95 translate-y-4 w-80 h-0 p-0 overflow-hidden pointer-events-none border-0 shadow-none"
      )}>
         {/* Internal Card Header */}
         <div className="bg-[rgb(var(--color-surface-2))] p-3 border-b border-[rgb(var(--color-border))] flex items-center justify-between">
           <div className="flex items-center gap-2">
             <span className="font-bold text-sm text-[rgb(var(--color-text-primary))]">
               {t('levelHelper.pendingAllocations')}
             </span>
           </div>
           <button
             onClick={onClose}
             className="text-[rgb(var(--color-text-muted))] hover:text-[rgb(var(--color-text-primary))]"
             title={t('common.dismiss')}
           >
             <X className="w-4 h-4" />
           </button>
         </div>

         {/* Content */}
         <div className="p-4 space-y-3">
            {hasPendingGains ? (
              <p className="text-xs text-[rgb(var(--color-text-muted))]">
                {t('levelHelper.pendingGainsMessage')}
              </p>
            ) : (
              <p className="text-xs text-[rgb(var(--color-text-muted))]">
                {t('levelHelper.noPendingAllocations')}
              </p>
            )}

            {/* Skills Row */}
            {skillPoints > 0 && (
              <div className="flex items-center justify-between p-2 bg-[rgb(var(--color-surface-2))] rounded hover:bg-[rgb(var(--color-surface-3))] transition-colors cursor-pointer group" onClick={() => handleNavigate('/skills')}>
                <div className="flex items-center gap-2">
                   <div className="p-1.5 bg-green-500/20 text-green-500 rounded-md">
                     <Zap className="w-4 h-4" />
                   </div>
                   <span className="text-sm font-medium">{t('levelHelper.skillPoints')}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-xs font-bold text-green-500">{skillPoints} {t('levelHelper.available')}</span>
                  <ArrowRight className="w-3 h-3 text-[rgb(var(--color-text-muted))] group-hover:translate-x-0.5 transition-transform" />
                </div>
              </div>
            )}

            {/* Feat Slots Row */}
            {featSlots > 0 && (
              <div className="flex items-center justify-between p-2 bg-[rgb(var(--color-surface-2))] rounded hover:bg-[rgb(var(--color-surface-3))] transition-colors cursor-pointer group" onClick={() => handleNavigate('/feats')}>
                <div className="flex items-center gap-2">
                   <div className="p-1.5 bg-purple-500/20 text-purple-500 rounded-md">
                     <Award className="w-4 h-4" />
                   </div>
                   <span className="text-sm font-medium">{t('levelHelper.featSlots')}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-xs font-bold text-purple-500">{featSlots} {t('levelHelper.available')}</span>
                  <ArrowRight className="w-3 h-3 text-[rgb(var(--color-text-muted))] group-hover:translate-x-0.5 transition-transform" />
                </div>
              </div>
            )}

            {/* Ability Score Row */}
            {abilityPoints > 0 && (
              <div className="flex items-center justify-between p-2 bg-[rgb(var(--color-surface-2))] rounded hover:bg-[rgb(var(--color-surface-3))] transition-colors cursor-pointer group" onClick={() => handleNavigate('/abilityScores')}>
                 <div className="flex items-center gap-2">
                    <div className="p-1.5 bg-yellow-500/20 text-yellow-500 rounded-md">
                      <AlertCircle className="w-4 h-4" />
                    </div>
                    <span className="text-sm font-medium">{t('levelHelper.abilityScoreIncrease')}</span>
                 </div>
                 <div className="flex items-center gap-2">
                   <span className="text-xs font-bold text-yellow-500">{abilityPoints} {t('levelHelper.available')}</span>
                   <ArrowRight className="w-3 h-3 text-[rgb(var(--color-text-muted))] group-hover:translate-x-0.5 transition-transform" />
                 </div>
              </div>
            )}

            {/* Spells to Learn Row */}
            {hasPendingSpells && spellData && (
              <div className="space-y-2">
                <div className="flex items-center justify-between p-2 bg-[rgb(var(--color-surface-2))] rounded hover:bg-[rgb(var(--color-surface-3))] transition-colors cursor-pointer group">
                  <div className="flex items-center gap-2 flex-1" onClick={() => handleNavigate('/spells')}>
                     <div className="p-1.5 bg-blue-500/20 text-blue-500 rounded-md">
                       <Sparkles className="w-4 h-4" />
                     </div>
                     <span className="text-sm font-medium">{t('levelHelper.spellsToLearn')}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-bold text-blue-500">{spellData.total} {t('levelHelper.available')}</span>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setSpellsExpanded(!spellsExpanded);
                      }}
                      className="p-0.5 hover:bg-[rgb(var(--color-surface-3))] rounded transition-colors"
                    >
                      {spellsExpanded ? (
                        <ChevronUp className="w-4 h-4 text-[rgb(var(--color-text-muted))]" />
                      ) : (
                        <ChevronDown className="w-4 h-4 text-[rgb(var(--color-text-muted))]" />
                      )}
                    </button>
                  </div>
                </div>

                {/* Expandable Spell Level Breakdown - Per Class */}
                {spellsExpanded && (
                  <div className="ml-4 pl-4 border-l-2 border-[rgb(var(--color-border))] space-y-3">
                    {spellData.classes.map((casterClass, classIdx) => (
                      <div key={classIdx} className="space-y-1">
                        <div className="text-xs font-medium text-[rgb(var(--color-text-primary))]">
                          {casterClass.name}
                          {casterClass.casterType === 'spellbook' && (
                            <span className="text-[rgb(var(--color-text-muted))] ml-1">
                              ({t('levelHelper.spellbook')})
                            </span>
                          )}
                        </div>
                        {casterClass.casterType === 'spellbook' ? (
                          <div className="flex items-center justify-between text-xs text-[rgb(var(--color-text-muted))] pl-2">
                            <span>{t('levelHelper.freeSpells')}</span>
                            <span className="font-medium text-blue-400">{casterClass.total}</span>
                          </div>
                        ) : (
                          Object.entries(casterClass.byLevel)
                            .sort(([a], [b]) => parseInt(a) - parseInt(b))
                            .map(([level, count]) => (
                              <div key={level} className="flex items-center justify-between text-xs text-[rgb(var(--color-text-muted))] pl-2">
                                <span>
                                  {parseInt(level) === 0
                                    ? t('levelHelper.cantrips')
                                    : t('levelHelper.spellLevel', { level })}
                                </span>
                                <span className="font-medium text-blue-400">{count}</span>
                              </div>
                            ))
                        )}
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
         </div>
      </div>

      {/* Trigger Button (Floating Action Button style) */}
      {hasPendingGains && (
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className={cn(
            "relative flex items-center justify-center w-12 h-12 rounded-full shadow-lg transition-all duration-300 hover:scale-105 active:scale-95 group z-50",
            isExpanded
               ? "bg-[rgb(var(--color-surface-3))] text-[rgb(var(--color-text-primary))]"
               : "bg-blue-600 text-white animate-bounce-subtle"
          )}
          title={isExpanded ? t('levelHelper.closeHelper') : t('levelHelper.pendingAllocations')}
        >
           {isExpanded ? (
             <X className="w-6 h-6" />
           ) : (
             <>
               <AlertTriangle className="w-6 h-6" />

               {/* Pulse effect ring */}
               <span className="absolute inset-0 rounded-full border-2 border-blue-400 opacity-75 animate-ping-slow"></span>
             </>
           )}
        </button>
      )}

    </div>,
    portalRoot
  );
}
