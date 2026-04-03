import React, { useEffect, useState } from 'react';
import { CharacterData } from '@/services/characterApi';
import {
  gameStateAPI,
  type CampaignSummaryResponse,
  type ModuleInfo,
} from '@/services/gameStateApi';
import { display, formatNumber } from '@/utils/dataHelpers';
import { Button } from '@/components/ui/Button';

interface CollapsibleSectionProps {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
  badge?: string | number;
}

function CollapsibleSection({ title, children, defaultOpen = false, badge }: CollapsibleSectionProps) {
  const [isOpen, setIsOpen] = useState(defaultOpen);
  
  return (
    <div className="group">
      <div className={`bg-gradient-to-r ${isOpen ? 'from-[rgb(var(--color-surface-2))] to-[rgb(var(--color-surface-1))]' : 'from-[rgb(var(--color-surface-1))] to-[rgb(var(--color-surface-1))]'} rounded-lg border border-[rgb(var(--color-surface-border)/0.5)] overflow-hidden transition-all duration-300 hover:border-[rgb(var(--color-primary)/0.3)]`}>
        <Button
          onClick={() => setIsOpen(!isOpen)}
          variant="ghost"
          className="w-full p-4 flex items-center justify-between h-auto"
        >
          <div className="flex items-center space-x-3">
            <h3 className="text-lg font-semibold text-[rgb(var(--color-text-primary))]">{title}</h3>
            {badge && (
              <span className="px-2.5 py-1 bg-gradient-to-r from-[rgb(var(--color-primary)/0.15)] to-[rgb(var(--color-primary)/0.1)] text-[rgb(var(--color-primary))] text-xs font-medium rounded-full">
                {badge}
              </span>
            )}
          </div>
          <svg 
            className={`w-5 h-5 text-[rgb(var(--color-text-muted))] transition-all duration-300 ${isOpen ? 'rotate-180' : ''}`}
            fill="none" 
            stroke="currentColor" 
            viewBox="0 0 24 24"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </Button>
        <div className={`transition-all duration-300 ease-out ${isOpen ? 'max-h-none opacity-100' : 'max-h-0 opacity-0 overflow-hidden'}`}>
          <div className={`px-6 md:px-8 pb-6 md:pb-8 ${isOpen ? 'border-t border-[rgb(var(--color-surface-border)/0.3)]' : 'border-t-0 border-transparent'}`}>
            <div className="pt-6 md:pt-8 grid-flow-row">
              {children}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

interface CampaignOverviewProps {
  character: CharacterData;
}

const CampaignOverview: React.FC<CampaignOverviewProps> = ({ character }) => {
  const [summary, setSummary] = useState<CampaignSummaryResponse | null>(null);
  const [moduleInfo, setModuleInfo] = useState<ModuleInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const loadCampaignData = async () => {
      if (!character.id) {
        if (!cancelled) {
          setSummary(null);
          setModuleInfo(null);
          setIsLoading(false);
        }
        return;
      }

      setIsLoading(true);
      setError(null);

      try {
        const [nextSummary, nextModuleInfo] = await Promise.all([
          gameStateAPI.getCampaignSummary(character.id),
          gameStateAPI.getModuleInfo(character.id),
        ]);

        if (!cancelled) {
          setSummary(nextSummary);
          setModuleInfo(nextModuleInfo);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : 'Failed to load campaign overview');
          setSummary(null);
          setModuleInfo(null);
        }
      } finally {
        if (!cancelled) {
          setIsLoading(false);
        }
      }
    };

    loadCampaignData();

    return () => {
      cancelled = true;
    };
  }, [character.id]);

  const formatTimestamp = (timestamp?: string | null): string => {
    if (!timestamp) return 'Unknown';
    try {
      const date = new Date(timestamp);
      return Number.isNaN(date.getTime()) ? 'Invalid Date' : date.toLocaleString();
    } catch {
      return 'Invalid Date';
    }
  };

  const formatRecruitmentLabel = (value: string): string => {
    if (!value) {
      return 'Unknown';
    }

    return value.replace(/_/g, ' ');
  };

  const companions = Object.entries(summary?.companion_status ?? {}).sort(([, left], [, right]) =>
    left.name.localeCompare(right.name)
  );
  const recruitedCompanions = companions.filter(([, companion]) => companion.recruitment === 'recruited');
  const metCompanions = companions.filter(([, companion]) => companion.recruitment === 'met');
  const questOverview = summary?.quest_overview;
  const gameAct = summary?.general_info?.game_act;
  const lastSaved = summary?.general_info?.last_saved;
  const playerName = summary?.general_info?.player_name;

  return (
    <CollapsibleSection 
      title="Campaign Overview" 
      defaultOpen={true}
      badge={gameAct ? `Act ${gameAct}` : "Campaign"}
    >
      <div className="space-y-6">
        {isLoading ? (
          <div className="text-sm text-[rgb(var(--color-text-muted))]">
            Loading campaign data...
          </div>
        ) : null}

        {error ? (
          <div className="rounded-lg border border-[rgb(var(--color-danger)/0.35)] bg-[rgb(var(--color-danger)/0.08)] px-4 py-3 text-sm text-[rgb(var(--color-danger))]">
            {error}
          </div>
        ) : null}

        {/* General Information */}
        <div>
          <h4 className="font-semibold text-[rgb(var(--color-text-primary))] mb-3 border-b border-[rgb(var(--color-surface-border)/0.6)] pb-1">
            General Information
          </h4>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-y-6 gap-x-4">
            <div>
              <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Game Act</div>
              <div className="text-lg font-medium text-[rgb(var(--color-text-primary))]">
                {gameAct ? `Act ${gameAct}` : 'Unknown'}
              </div>
            </div>
            <div>
              <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Player</div>
              <div className="text-lg font-medium text-[rgb(var(--color-text-primary))]">
                {display(playerName || character.name)}
              </div>
            </div>
            {lastSaved ? (
              <div className="col-span-2">
                <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Last Saved</div>
                <div className="text-lg font-medium text-[rgb(var(--color-text-primary))]">
                  {formatTimestamp(lastSaved)}
                </div>
              </div>
            ) : null}
          </div>
        </div>

        {/* Session Information */}
        <div>
          <h4 className="font-semibold text-[rgb(var(--color-text-primary))] mb-3 border-b border-[rgb(var(--color-surface-border)/0.6)] pb-1">
            Session Information
          </h4>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-y-6 gap-x-4">
            {moduleInfo?.campaign ? (
              <div>
                <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Campaign</div>
                <div className="text-lg font-medium text-[rgb(var(--color-text-primary))]">
                  {display(moduleInfo.campaign)}
                </div>
              </div>
            ) : null}
            {moduleInfo?.module_name ? (
              <div>
                <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Module</div>
                <div className="text-lg font-medium text-[rgb(var(--color-text-primary))]">
                  {display(moduleInfo.module_name)}
                </div>
              </div>
            ) : null}
            {moduleInfo?.area_name ? (
              <div>
                <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Current Area</div>
                <div className="text-lg font-medium text-[rgb(var(--color-text-primary))]">
                  {display(moduleInfo.area_name)}
                </div>
              </div>
            ) : null}
            {moduleInfo?.entry_area ? (
              <div>
                <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Entry Area</div>
                <div className="text-lg font-medium text-[rgb(var(--color-text-primary))]">
                  {display(moduleInfo.entry_area)}
                </div>
              </div>
            ) : null}
          </div>
          {moduleInfo?.module_description ? (
            <div className="mt-4 text-sm text-[rgb(var(--color-text-secondary))]">
              {display(moduleInfo.module_description)}
            </div>
          ) : null}
        </div>

        {/* Companion Status */}
        <div>
          <h4 className="font-semibold text-[rgb(var(--color-text-primary))] mb-3 border-b border-[rgb(var(--color-surface-border)/0.6)] pb-1">
            Companion Status
          </h4>
          <div className="grid grid-cols-3 gap-y-6 gap-x-4">
            <div>
              <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Recruited</div>
              <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">
                {formatNumber(recruitedCompanions.length)}
              </div>
            </div>
            <div>
              <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Met</div>
              <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">
                {formatNumber(metCompanions.length)}
              </div>
            </div>
            <div>
              <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Tracked</div>
              <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">
                {formatNumber(companions.length)}
              </div>
            </div>
          </div>
          {companions.length > 0 ? (
            <div className="mt-4 grid grid-cols-1 md:grid-cols-2 gap-3">
              {companions.map(([id, companion]) => (
                <div
                  key={id}
                  className="rounded-lg border border-[rgb(var(--color-surface-border)/0.5)] bg-[rgb(var(--color-surface-2)/0.45)] px-4 py-3"
                >
                  <div className="flex items-center justify-between gap-3">
                    <div className="font-medium text-[rgb(var(--color-text-primary))]">
                      {display(companion.name || id)}
                    </div>
                    <div className="text-xs uppercase text-[rgb(var(--color-text-muted))]">
                      {display(formatRecruitmentLabel(companion.recruitment))}
                    </div>
                  </div>
                  <div className="mt-2 text-sm text-[rgb(var(--color-text-secondary))]">
                    Influence: {companion.influence ?? 'Unknown'}
                  </div>
                </div>
              ))}
            </div>
          ) : !isLoading && !error ? (
            <div className="mt-4 text-sm text-[rgb(var(--color-text-muted))]">
              No tracked companions found in this save.
            </div>
          ) : null}
        </div>

        {/* Quest Progress */}
        <div>
          <h4 className="font-semibold text-[rgb(var(--color-text-primary))] mb-3 border-b border-[rgb(var(--color-surface-border)/0.6)] pb-1">
            Quest Progress
          </h4>

          <div className="grid grid-cols-3 gap-y-6 gap-x-4">
            <div>
              <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Completed</div>
              <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">
                {formatNumber(questOverview?.completed_count || 0)}
              </div>
            </div>
            <div>
              <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Active</div>
              <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">
                {formatNumber(questOverview?.active_count || 0)}
              </div>
            </div>
            <div>
              <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Complete</div>
              <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">
                {Math.round(questOverview?.completion_percentage || 0)}%
              </div>
            </div>
            <div>
              <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Quest Vars</div>
              <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">
                {formatNumber(questOverview?.total_quest_vars || 0)}
              </div>
            </div>
            <div>
              <div className="text-xs text-[rgb(var(--color-text-muted))] uppercase mb-1">Groups</div>
              <div className="text-xl font-bold text-[rgb(var(--color-text-primary))]">
                {formatNumber(Object.keys(questOverview?.quest_groups || {}).length)}
              </div>
            </div>
          </div>
        </div>
      </div>
    </CollapsibleSection>
  );
};

export default CampaignOverview;
