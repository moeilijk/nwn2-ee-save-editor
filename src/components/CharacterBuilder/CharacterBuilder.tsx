import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  AvailableRace,
  AvailableClass,
  AbilityScores,
  AbilitiesState,
  FeatSlots,
  FeatSummary,
  FeatInfo,
} from '@/lib/bindings';

// ─── Types ───────────────────────────────────────────────────────────────────

interface SubraceInfo {
  name: string;
  description: string | null;
  ability_adjustments: { Str: number; Dex: number; Con: number; Int: number; Wis: number; Cha: number };
}

interface ClassEntry {
  class_id: number;
  level: number;
}

interface FilteredFeatsResponse {
  feats: FeatInfo[];
  total: number;
  page: number;
  pages: number;
  has_next: boolean;
  has_previous: boolean;
}

// ─── Point Buy ───────────────────────────────────────────────────────────────

const POINT_BUY_COSTS = [0, 1, 2, 3, 4, 5, 6, 8, 10, 13, 16]; // index = score - 8
const POINT_BUY_BUDGET = 32;
const SCORE_MIN = 8;
const SCORE_MAX = 18;

function pointCost(score: number): number {
  const idx = score - SCORE_MIN;
  if (idx < 0) return 0;
  if (idx >= POINT_BUY_COSTS.length) return POINT_BUY_COSTS[POINT_BUY_COSTS.length - 1];
  return POINT_BUY_COSTS[idx];
}

function totalCost(scores: AbilityScores): number {
  return (
    pointCost(scores.Str) +
    pointCost(scores.Dex) +
    pointCost(scores.Con) +
    pointCost(scores.Int) +
    pointCost(scores.Wis) +
    pointCost(scores.Cha)
  );
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

const ABILITIES: { key: keyof AbilityScores; label: string }[] = [
  { key: 'Str', label: 'Strength' },
  { key: 'Dex', label: 'Dexterity' },
  { key: 'Con', label: 'Constitution' },
  { key: 'Int', label: 'Intelligence' },
  { key: 'Wis', label: 'Wisdom' },
  { key: 'Cha', label: 'Charisma' },
];

function modStr(v: number) {
  const m = Math.floor((v - 10) / 2);
  return m >= 0 ? `+${m}` : `${m}`;
}

function adjStr(v: number) {
  if (v === 0) return '';
  return v > 0 ? `+${v}` : `${v}`;
}

// ─── Step indicator ──────────────────────────────────────────────────────────

const STEPS = ['Race', 'Class', 'Abilities', 'Feats'];

function StepBar({ current }: { current: number }) {
  return (
    <div className="flex items-center mb-8">
      {STEPS.map((label, i) => (
        <React.Fragment key={label}>
          <div className="flex flex-col items-center">
            <div
              className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold transition-colors ${
                i < current
                  ? 'bg-[rgb(var(--color-primary))] text-white opacity-60'
                  : i === current
                  ? 'bg-[rgb(var(--color-primary))] text-white'
                  : 'bg-[rgb(var(--color-surface-2))] text-[rgb(var(--color-text-muted))]'
              }`}
            >
              {i < current ? '✓' : i + 1}
            </div>
            <span className={`text-xs mt-1 ${i === current ? 'text-[rgb(var(--color-text-primary))] font-semibold' : 'text-[rgb(var(--color-text-muted))]'}`}>
              {label}
            </span>
          </div>
          {i < STEPS.length - 1 && (
            <div className={`flex-1 h-0.5 mx-2 mb-4 transition-colors ${i < current ? 'bg-[rgb(var(--color-primary))]' : 'bg-[rgb(var(--color-surface-2))]'}`} />
          )}
        </React.Fragment>
      ))}
    </div>
  );
}

// ─── Step 0: Race ────────────────────────────────────────────────────────────

function StepRace({
  onDone,
}: {
  onDone: () => void;
}) {
  const [races, setRaces] = useState<AvailableRace[]>([]);
  const [subraces, setSubraces] = useState<SubraceInfo[]>([]);
  const [selectedRaceId, setSelectedRaceId] = useState<number | null>(null);
  const [selectedSubrace, setSelectedSubrace] = useState<string | null>(null);
  const [currentRaceId, setCurrentRaceId] = useState<number | null>(null);
  const [applying, setApplying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      const [allRaces, currentRace] = await Promise.all([
        invoke<AvailableRace[]>('get_available_races'),
        invoke<number>('get_race_id').catch(() => null),
      ]);
      setRaces(allRaces.filter(r => r.is_playable));
      setCurrentRaceId(currentRace);
      setSelectedRaceId(currentRace);
    })();
  }, []);

  useEffect(() => {
    if (selectedRaceId === null) return;
    invoke<SubraceInfo[]>('get_available_subraces', { raceId: selectedRaceId })
      .then(setSubraces)
      .catch(() => setSubraces([]));
  }, [selectedRaceId]);

  const apply = async () => {
    if (selectedRaceId === null) return;
    setApplying(true);
    setError(null);
    try {
      await invoke('change_race', { raceId: selectedRaceId, subrace: selectedSubrace });
      onDone();
    } catch (e) {
      setError(String(e));
    } finally {
      setApplying(false);
    }
  };

  const selectedRace = races.find(r => r.id === selectedRaceId);

  return (
    <div>
      <h2 className="text-lg font-semibold text-[rgb(var(--color-text-primary))] mb-1">Choose Race</h2>
      <p className="text-sm text-[rgb(var(--color-text-muted))] mb-4">
        Select a race for your character. Racial ability adjustments are applied on top of your base scores.
      </p>

      {error && <div className="mb-4 p-3 rounded bg-red-900/30 text-red-300 text-sm">{error}</div>}

      <div className="grid grid-cols-2 gap-2 mb-4 max-h-[380px] overflow-y-auto pr-1">
        {races.map(race => {
          const isSelected = selectedRaceId === race.id;
          const isCurrent = currentRaceId === race.id;

          return (
            <button
              key={race.id}
              onClick={() => { setSelectedRaceId(race.id); setSelectedSubrace(null); }}
              className={`text-left p-3 rounded-lg border transition-all ${
                isSelected
                  ? 'border-[rgb(var(--color-primary))] bg-[rgb(var(--color-primary)/0.1)]'
                  : 'border-[rgb(var(--color-surface-border))] bg-[rgb(var(--color-surface-1))] hover:border-[rgb(var(--color-primary)/0.5)]'
              }`}
            >
              <div className="flex items-center gap-2 mb-1">
                <span className="font-semibold text-sm text-[rgb(var(--color-text-primary))]">{race.name}</span>
                {isCurrent && (
                  <span className="text-xs px-1.5 py-0.5 rounded bg-[rgb(var(--color-primary)/0.2)] text-[rgb(var(--color-primary))]">current</span>
                )}
              </div>
            </button>
          );
        })}
      </div>

      {subraces.length > 0 && (
        <div className="mb-4">
          <label className="text-sm font-medium text-[rgb(var(--color-text-secondary))] block mb-2">
            Subrace
          </label>
          <div className="flex flex-wrap gap-2">
            <button
              onClick={() => setSelectedSubrace(null)}
              className={`px-3 py-1.5 rounded-lg text-sm border transition-colors ${
                selectedSubrace === null
                  ? 'border-[rgb(var(--color-primary))] bg-[rgb(var(--color-primary)/0.1)] text-[rgb(var(--color-primary))]'
                  : 'border-[rgb(var(--color-surface-border))] text-[rgb(var(--color-text-secondary))]'
              }`}
            >
              None
            </button>
            {subraces.map(sr => (
              <button
                key={sr.name}
                onClick={() => setSelectedSubrace(sr.name)}
                className={`px-3 py-1.5 rounded-lg text-sm border transition-colors ${
                  selectedSubrace === sr.name
                    ? 'border-[rgb(var(--color-primary))] bg-[rgb(var(--color-primary)/0.1)] text-[rgb(var(--color-primary))]'
                    : 'border-[rgb(var(--color-surface-border))] text-[rgb(var(--color-text-secondary))]'
                }`}
              >
                {sr.name}
              </button>
            ))}
          </div>
        </div>
      )}

      {selectedRace && (
        <div className="p-3 rounded-lg bg-[rgb(var(--color-surface-1))] text-sm text-[rgb(var(--color-text-secondary))] mb-4 border border-[rgb(var(--color-surface-border))]">
          <span className="font-semibold text-[rgb(var(--color-text-primary))]">{selectedRace.name}</span>
        </div>
      )}

      <div className="flex justify-end">
        <button
          onClick={apply}
          disabled={applying || selectedRaceId === null}
          className="px-5 py-2 rounded-lg bg-[rgb(var(--color-primary))] text-white font-semibold text-sm disabled:opacity-50 hover:opacity-90 transition-opacity"
        >
          {applying ? 'Applying...' : 'Next: Class →'}
        </button>
      </div>
    </div>
  );
}

// ─── Step 1: Class ───────────────────────────────────────────────────────────

function StepClass({ onDone, onBack }: { onDone: () => void; onBack: () => void }) {
  const [classes, setClasses] = useState<AvailableClass[]>([]);
  const [selectedClassId, setSelectedClassId] = useState<number | null>(null);
  const [currentClassId, setCurrentClassId] = useState<number | null>(null);
  const [applying, setApplying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      const [allClasses, entries] = await Promise.all([
        invoke<AvailableClass[]>('get_available_classes'),
        invoke<ClassEntry[]>('get_class_entries').catch(() => [] as ClassEntry[]),
      ]);
      // Only show non-prestige classes for level 1
      setClasses(allClasses.filter(c => !c.is_prestige));
      const startingClass = entries[0]?.class_id ?? null;
      setCurrentClassId(startingClass);
      setSelectedClassId(startingClass);
    })();
  }, []);

  const apply = async () => {
    if (selectedClassId === null) return;
    setApplying(true);
    setError(null);
    try {
      if (currentClassId !== null && currentClassId !== selectedClassId) {
        await invoke('change_class', { oldClassId: currentClassId, newClassId: selectedClassId });
      }
      onDone();
    } catch (e) {
      setError(String(e));
    } finally {
      setApplying(false);
    }
  };

  const selectedClass = classes.find(c => c.id === selectedClassId);

  return (
    <div>
      <h2 className="text-lg font-semibold text-[rgb(var(--color-text-primary))] mb-1">Choose Starting Class</h2>
      <p className="text-sm text-[rgb(var(--color-text-muted))] mb-4">
        Prestige classes are excluded — they cannot be taken at level 1.
      </p>

      {error && <div className="mb-4 p-3 rounded bg-red-900/30 text-red-300 text-sm">{error}</div>}

      <div className="grid grid-cols-2 gap-2 mb-4 max-h-[380px] overflow-y-auto pr-1">
        {classes.map(cls => {
          const isSelected = selectedClassId === cls.id;
          const isCurrent = currentClassId === cls.id;

          return (
            <button
              key={cls.id}
              onClick={() => setSelectedClassId(cls.id)}
              className={`text-left p-3 rounded-lg border transition-all ${
                isSelected
                  ? 'border-[rgb(var(--color-primary))] bg-[rgb(var(--color-primary)/0.1)]'
                  : 'border-[rgb(var(--color-surface-border))] bg-[rgb(var(--color-surface-1))] hover:border-[rgb(var(--color-primary)/0.5)]'
              }`}
            >
              <div className="flex items-center gap-2 mb-0.5">
                <span className="font-semibold text-sm text-[rgb(var(--color-text-primary))]">{cls.name}</span>
                {isCurrent && (
                  <span className="text-xs px-1.5 py-0.5 rounded bg-[rgb(var(--color-primary)/0.2)] text-[rgb(var(--color-primary))]">current</span>
                )}
              </div>
              <div className="flex gap-2 text-xs text-[rgb(var(--color-text-muted))]">
                {cls.hit_die > 0 && <span>d{cls.hit_die}</span>}
              </div>
            </button>
          );
        })}
      </div>


      <div className="flex justify-between">
        <button
          onClick={onBack}
          className="px-5 py-2 rounded-lg border border-[rgb(var(--color-surface-border))] text-sm text-[rgb(var(--color-text-secondary))] hover:bg-[rgb(var(--color-surface-1))] transition-colors"
        >
          ← Back
        </button>
        <button
          onClick={apply}
          disabled={applying || selectedClassId === null}
          className="px-5 py-2 rounded-lg bg-[rgb(var(--color-primary))] text-white font-semibold text-sm disabled:opacity-50 hover:opacity-90 transition-opacity"
        >
          {applying ? 'Applying...' : 'Next: Abilities →'}
        </button>
      </div>
    </div>
  );
}

// ─── Step 2: Abilities ───────────────────────────────────────────────────────

function StepAbilities({ onDone, onBack }: { onDone: () => void; onBack: () => void }) {
  const [scores, setScores] = useState<AbilityScores>({ Str: 8, Dex: 8, Con: 8, Int: 8, Wis: 8, Cha: 8 });
  const [racialMods, setRacialMods] = useState<AbilityScores>({ Str: 0, Dex: 0, Con: 0, Int: 0, Wis: 0, Cha: 0 });
  const [applying, setApplying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<AbilitiesState>('get_abilities_state').then(state => {
      setScores(state.point_buy.starting_scores);
      setRacialMods(state.racial_modifiers as unknown as AbilityScores); // AbilityModifiers has same shape
    }).catch(() => {});
  }, []);

  const spent = totalCost(scores);
  const remaining = POINT_BUY_BUDGET - spent;

  const adjust = (key: keyof AbilityScores, delta: number) => {
    setScores(prev => {
      const next = Math.max(SCORE_MIN, Math.min(SCORE_MAX, prev[key] + delta));
      const newScores = { ...prev, [key]: next };
      if (totalCost(newScores) > POINT_BUY_BUDGET) return prev;
      return newScores;
    });
  };

  const apply = async () => {
    setApplying(true);
    setError(null);
    try {
      await invoke('apply_point_buy', { newScores: scores });
      onDone();
    } catch (e) {
      setError(String(e));
    } finally {
      setApplying(false);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-lg font-semibold text-[rgb(var(--color-text-primary))]">Ability Scores</h2>
          <p className="text-sm text-[rgb(var(--color-text-muted))]">Point buy — 32 points to spend</p>
        </div>
        <div className={`text-2xl font-bold ${remaining < 0 ? 'text-red-400' : remaining === 0 ? 'text-green-400' : 'text-[rgb(var(--color-primary))]'}`}>
          {remaining} <span className="text-sm font-normal text-[rgb(var(--color-text-muted))]">left</span>
        </div>
      </div>

      {error && <div className="mb-4 p-3 rounded bg-red-900/30 text-red-300 text-sm">{error}</div>}

      <div className="space-y-2 mb-6">
        {ABILITIES.map(({ key, label }) => {
          const base = scores[key];
          const racial = (racialMods as unknown as Record<string, number>)[key] ?? 0;
          const effective = base + racial;
          const costIncrement = pointCost(base + 1) - pointCost(base);
          const canIncrease = base < SCORE_MAX && remaining >= costIncrement;
          const canDecrease = base > SCORE_MIN;

          return (
            <div key={key} className="flex items-center gap-3 p-2 rounded-lg bg-[rgb(var(--color-surface-1))] border border-[rgb(var(--color-surface-border))]">
              <span className="w-24 text-sm font-medium text-[rgb(var(--color-text-secondary))]">{label}</span>

              <button
                onClick={() => adjust(key, -1)}
                disabled={!canDecrease}
                className="w-7 h-7 rounded flex items-center justify-center bg-[rgb(var(--color-surface-2))] text-[rgb(var(--color-text-primary))] disabled:opacity-30 hover:bg-[rgb(var(--color-surface-border))] transition-colors font-bold"
              >−</button>

              <span className="w-6 text-center text-lg font-bold text-[rgb(var(--color-text-primary))]">{base}</span>

              <button
                onClick={() => adjust(key, 1)}
                disabled={!canIncrease}
                className="w-7 h-7 rounded flex items-center justify-center bg-[rgb(var(--color-surface-2))] text-[rgb(var(--color-text-primary))] disabled:opacity-30 hover:bg-[rgb(var(--color-surface-border))] transition-colors font-bold"
              >+</button>

              <div className="flex items-center gap-2 ml-auto text-sm">
                {racial !== 0 && (
                  <span className={`text-xs px-1.5 rounded ${racial > 0 ? 'text-green-400 bg-green-900/20' : 'text-red-400 bg-red-900/20'}`}>
                    {racial > 0 ? `+${racial}` : racial}
                  </span>
                )}
                <span className="font-semibold text-[rgb(var(--color-text-primary))] w-8 text-center">{effective}</span>
                <span className="text-[rgb(var(--color-text-muted))] w-8 text-center">{modStr(effective)}</span>
                <span className="text-xs text-[rgb(var(--color-text-muted))] w-12 text-right">{pointCost(base)} pts</span>
              </div>
            </div>
          );
        })}
      </div>

      <div className="flex justify-between">
        <button
          onClick={onBack}
          className="px-5 py-2 rounded-lg border border-[rgb(var(--color-surface-border))] text-sm text-[rgb(var(--color-text-secondary))] hover:bg-[rgb(var(--color-surface-1))] transition-colors"
        >
          ← Back
        </button>
        <button
          onClick={apply}
          disabled={applying || remaining < 0}
          className="px-5 py-2 rounded-lg bg-[rgb(var(--color-primary))] text-white font-semibold text-sm disabled:opacity-50 hover:opacity-90 transition-opacity"
        >
          {applying ? 'Applying...' : 'Next: Feats →'}
        </button>
      </div>
    </div>
  );
}

// ─── Step 3: Feats ───────────────────────────────────────────────────────────

function StepFeats({ onDone, onBack }: { onDone: () => void; onBack: () => void }) {
  const [slots, setSlots] = useState<FeatSlots | null>(null);
  const [summary, setSummary] = useState<FeatSummary | null>(null);
  const [featNameMap, setFeatNameMap] = useState<Map<number, string>>(new Map());
  const [availableFeats, setAvailableFeats] = useState<FeatInfo[]>([]);
  const [totalAvailable, setTotalAvailable] = useState(0);
  const [page, setPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [search, setSearch] = useState('');
  const [searchInput, setSearchInput] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const LIMIT = 20;

  const refresh = useCallback(async () => {
    const [s, sm, allFeats] = await Promise.all([
      invoke<FeatSlots>('get_feat_slots'),
      invoke<FeatSummary>('get_feat_summary'),
      invoke<FeatInfo[]>('get_all_feats').catch(() => [] as FeatInfo[]),
    ]);
    setSlots(s);
    setSummary(sm);
    setFeatNameMap(new Map(allFeats.map(f => [f.id, f.name])));
  }, []);

  const loadFeats = useCallback(async (p: number, q: string) => {
    try {
      const res = await invoke<FilteredFeatsResponse>('get_filtered_feats', {
        page: p,
        limit: LIMIT,
        featType: null,
        search: q || null,
      });
      setAvailableFeats(res.feats);
      setTotalAvailable(res.total);
      setTotalPages(res.pages);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    loadFeats(page, search);
  }, [loadFeats, page, search]);

  const removeFeat = async (featId: number) => {
    setActionError(null);
    try {
      await invoke('remove_feat', { featId });
      await Promise.all([refresh(), loadFeats(page, search)]);
    } catch (e) {
      setActionError(String(e));
    }
  };

  const addFeat = async (featId: number) => {
    setActionError(null);
    try {
      await invoke('add_feat', { featId });
      await Promise.all([refresh(), loadFeats(page, search)]);
    } catch (e) {
      setActionError(String(e));
    }
  };

  const currentFeatIds = new Set(summary?.feats?.map(f => f.feat_id) ?? []);

  const handleSearch = () => {
    setPage(1);
    setSearch(searchInput);
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-lg font-semibold text-[rgb(var(--color-text-primary))]">Feats</h2>
          <p className="text-sm text-[rgb(var(--color-text-muted))]">Add or remove feats for your character</p>
        </div>
        {slots && (
          <div className="text-sm text-right">
            <span className="text-[rgb(var(--color-text-muted))]">Slots: </span>
            <span className={`font-bold ${slots.open_slots > 0 ? 'text-yellow-400' : 'text-[rgb(var(--color-text-primary))]'}`}>
              {slots.filled_slots}/{slots.total_slots}
            </span>
            {slots.open_slots > 0 && (
              <span className="ml-1 text-yellow-400 text-xs">({slots.open_slots} open)</span>
            )}
          </div>
        )}
      </div>

      {error && <div className="mb-3 p-2 rounded bg-red-900/30 text-red-300 text-sm">{error}</div>}
      {actionError && <div className="mb-3 p-2 rounded bg-red-900/30 text-red-300 text-sm">{actionError}</div>}

      {/* Current feats */}
      {summary?.feats && summary.feats.length > 0 && (
        <div className="mb-4">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-[rgb(var(--color-text-muted))] mb-2">
            Current Feats ({summary.feats.length})
          </h3>
          <div className="flex flex-wrap gap-1.5 max-h-[120px] overflow-y-auto p-2 rounded-lg bg-[rgb(var(--color-surface-1))] border border-[rgb(var(--color-surface-border))]">
            {summary.feats.map(fe => (
              <span
                key={fe.feat_id}
                className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs bg-[rgb(var(--color-surface-2))] text-[rgb(var(--color-text-secondary))] border border-[rgb(var(--color-surface-border))]"
              >
                {featNameMap.get(fe.feat_id) ?? `Feat #${fe.feat_id}`}
                <button
                  onClick={() => removeFeat(fe.feat_id)}
                  className="hover:text-red-400 transition-colors leading-none"
                  title="Remove feat"
                >
                  ×
                </button>
              </span>
            ))}
          </div>
        </div>
      )}

      {/* Search */}
      <div className="flex gap-2 mb-3">
        <input
          type="text"
          value={searchInput}
          onChange={e => setSearchInput(e.target.value)}
          onKeyDown={e => e.key === 'Enter' && handleSearch()}
          placeholder="Search feats..."
          className="flex-1 px-3 py-1.5 rounded-lg bg-[rgb(var(--color-surface-1))] border border-[rgb(var(--color-surface-border))] text-sm text-[rgb(var(--color-text-primary))] placeholder:text-[rgb(var(--color-text-muted))] focus:outline-none focus:border-[rgb(var(--color-primary))]"
        />
        <button
          onClick={handleSearch}
          className="px-3 py-1.5 rounded-lg bg-[rgb(var(--color-surface-2))] text-sm text-[rgb(var(--color-text-secondary))] hover:bg-[rgb(var(--color-surface-border))] transition-colors"
        >
          Search
        </button>
      </div>

      {/* Available feats */}
      <div className="space-y-1 max-h-[240px] overflow-y-auto pr-1 mb-3">
        {availableFeats.map(feat => {
          const already = currentFeatIds.has(feat.id);
          return (
            <div
              key={feat.id}
              className={`flex items-center gap-2 p-2 rounded-lg text-sm border transition-colors ${
                already
                  ? 'border-[rgb(var(--color-primary)/0.3)] bg-[rgb(var(--color-primary)/0.05)]'
                  : 'border-[rgb(var(--color-surface-border))] bg-[rgb(var(--color-surface-1))]'
              }`}
            >
              <div className="flex-1 min-w-0">
                <span className={`font-medium ${already ? 'text-[rgb(var(--color-primary))]' : 'text-[rgb(var(--color-text-primary))]'}`}>
                  {feat.name}
                </span>
                {feat.description && (
                  <p className="text-xs text-[rgb(var(--color-text-muted))] truncate mt-0.5">{feat.description}</p>
                )}
              </div>
              {already ? (
                <button
                  onClick={() => removeFeat(feat.id)}
                  className="shrink-0 px-2 py-1 rounded text-xs border border-red-500/50 text-red-400 hover:bg-red-900/20 transition-colors"
                >
                  Remove
                </button>
              ) : (
                <button
                  onClick={() => addFeat(feat.id)}
                  className="shrink-0 px-2 py-1 rounded text-xs border border-[rgb(var(--color-primary)/0.5)] text-[rgb(var(--color-primary))] hover:bg-[rgb(var(--color-primary)/0.1)] transition-colors"
                >
                  Add
                </button>
              )}
            </div>
          );
        })}
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="flex items-center justify-center gap-2 mb-3 text-sm">
          <button
            onClick={() => setPage(p => Math.max(1, p - 1))}
            disabled={page === 1}
            className="px-2 py-1 rounded border border-[rgb(var(--color-surface-border))] disabled:opacity-30 hover:bg-[rgb(var(--color-surface-1))] transition-colors"
          >
            ‹
          </button>
          <span className="text-[rgb(var(--color-text-muted))]">
            {page} / {totalPages} ({totalAvailable} feats)
          </span>
          <button
            onClick={() => setPage(p => Math.min(totalPages, p + 1))}
            disabled={page === totalPages}
            className="px-2 py-1 rounded border border-[rgb(var(--color-surface-border))] disabled:opacity-30 hover:bg-[rgb(var(--color-surface-1))] transition-colors"
          >
            ›
          </button>
        </div>
      )}

      <div className="flex justify-between">
        <button
          onClick={onBack}
          className="px-5 py-2 rounded-lg border border-[rgb(var(--color-surface-border))] text-sm text-[rgb(var(--color-text-secondary))] hover:bg-[rgb(var(--color-surface-1))] transition-colors"
        >
          ← Back
        </button>
        <button
          onClick={onDone}
          className="px-5 py-2 rounded-lg bg-[rgb(var(--color-primary))] text-white font-semibold text-sm hover:opacity-90 transition-opacity"
        >
          Done ✓
        </button>
      </div>
    </div>
  );
}

// ─── Done screen ─────────────────────────────────────────────────────────────

function StepDone({ onRestart }: { onRestart: () => void }) {
  return (
    <div className="text-center py-12">
      <div className="text-5xl mb-4">✓</div>
      <h2 className="text-xl font-bold text-[rgb(var(--color-text-primary))] mb-2">Changes Applied</h2>
      <p className="text-sm text-[rgb(var(--color-text-muted))] mb-6">
        Race, class, ability scores and feats have been updated.<br />
        Remember to save your character with the Save button above.
      </p>
      <button
        onClick={onRestart}
        className="px-5 py-2 rounded-lg border border-[rgb(var(--color-surface-border))] text-sm text-[rgb(var(--color-text-secondary))] hover:bg-[rgb(var(--color-surface-1))] transition-colors"
      >
        Start Over
      </button>
    </div>
  );
}

// ─── Main ─────────────────────────────────────────────────────────────────────

const CharacterBuilder: React.FC = () => {
  const [step, setStep] = useState(0);
  const TOTAL_STEPS = 4;

  return (
    <div className="max-w-2xl mx-auto">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-[rgb(var(--color-text-primary))]">Character Foundation</h1>
        <p className="text-sm text-[rgb(var(--color-text-muted))] mt-1">
          Adjust your character's level 1 choices: race, starting class, ability scores, and feats.
          Each step applies changes immediately to your loaded character.
        </p>
      </div>

      {step < TOTAL_STEPS && <StepBar current={step} />}

      <div className="bg-[rgb(var(--color-surface-1))] rounded-xl border border-[rgb(var(--color-surface-border))] p-6">
        {step === 0 && <StepRace onDone={() => setStep(1)} />}
        {step === 1 && <StepClass onDone={() => setStep(2)} onBack={() => setStep(0)} />}
        {step === 2 && <StepAbilities onDone={() => setStep(3)} onBack={() => setStep(1)} />}
        {step === 3 && <StepFeats onDone={() => setStep(4)} onBack={() => setStep(2)} />}
        {step === 4 && <StepDone onRestart={() => setStep(0)} />}
      </div>
    </div>
  );
};

export default CharacterBuilder;
