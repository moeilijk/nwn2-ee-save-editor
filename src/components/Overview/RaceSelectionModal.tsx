import React, { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Card, CardContent } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import type { AvailableRace } from '@/lib/bindings';

const X = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
  </svg>
);

const Search = ({ className }: { className?: string }) => (
  <svg className={className} fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
  </svg>
);

interface SubraceInfo {
  name: string;
  label: string;
}

interface RaceSelectionModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectRace: (raceId: number, raceName: string, subrace: string | null) => void;
  currentRaceName?: string;
  currentSubrace?: string | null;
}

export default function RaceSelectionModal({
  isOpen,
  onClose,
  onSelectRace,
  currentRaceName,
  currentSubrace,
}: RaceSelectionModalProps) {
  const [races, setRaces] = useState<AvailableRace[]>([]);
  const [subraces, setSubraces] = useState<SubraceInfo[]>([]);
  const [selectedRace, setSelectedRace] = useState<AvailableRace | null>(null);
  const [selectedSubrace, setSelectedSubrace] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    setSearchQuery('');
    setIsLoading(true);
    invoke<AvailableRace[]>('get_available_races')
      .then(r => {
        const playable = r.filter(race => race.is_playable);
        setRaces(playable);
        const current = playable.find(race => race.name === currentRaceName) ?? playable[0] ?? null;
        setSelectedRace(current);
        setSelectedSubrace(currentSubrace ?? null);
      })
      .catch(e => console.error('[RaceModal] error:', e))
      .finally(() => setIsLoading(false));
  }, [isOpen]);

  useEffect(() => {
    if (!selectedRace) { setSubraces([]); return; }
    invoke<SubraceInfo[]>('get_available_subraces', { raceId: selectedRace.id })
      .then(setSubraces)
      .catch(() => setSubraces([]));
  }, [selectedRace]);

  const filteredRaces = useMemo(() => {
    if (!searchQuery) return races;
    const q = searchQuery.toLowerCase();
    return races.filter(r => r.name.toLowerCase().includes(q));
  }, [races, searchQuery]);

  const handleConfirm = () => {
    if (!selectedRace) return;
    onSelectRace(selectedRace.id, selectedRace.name, selectedSubrace);
    onClose();
  };

  if (!isOpen) return null;


  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
      <Card className="w-[90vw] max-w-5xl h-[80vh] flex flex-col bg-[rgb(var(--color-surface-1))] border-[rgb(var(--color-surface-border))] shadow-2xl relative overflow-hidden">

        <div className="absolute top-4 right-4 z-20">
          <Button onClick={onClose} variant="ghost" size="sm" className="h-8 w-8 p-0 bg-black/20 hover:bg-black/40 text-white rounded-full">
            <X className="w-5 h-5" />
          </Button>
        </div>

        <CardContent padding="p-0" className="flex flex-col h-full">
          {isLoading ? (
            <div className="flex-1 flex flex-col items-center justify-center text-[rgb(var(--color-text-muted))] gap-3">
              <div className="w-12 h-12 rounded-full border-4 border-[rgb(var(--color-primary)/0.2)] border-t-[rgb(var(--color-primary))] animate-spin" />
              <span className="text-lg font-medium">Loading races...</span>
            </div>
          ) : (
            <>
              <div className="flex-1 flex overflow-hidden">

                {/* Left: race list */}
                <div className="w-1/3 min-w-[280px] flex flex-col border-r border-[rgb(var(--color-surface-border))] bg-[rgb(var(--color-surface-2))/30]">
                  <div className="p-4 border-b border-[rgb(var(--color-surface-border))] pt-12">
                    <div className="relative">
                      <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
                      <Input
                        placeholder="Search races..."
                        value={searchQuery}
                        onChange={e => setSearchQuery(e.target.value)}
                        className="pl-9 bg-[rgb(var(--color-surface-1))]"
                        autoFocus
                      />
                    </div>
                    <div className="text-xs text-[rgb(var(--color-text-muted))] text-right mt-2">
                      {filteredRaces.length} races
                    </div>
                  </div>

                  <div className="flex-1 overflow-y-auto p-2 space-y-1">
                    {filteredRaces.map(race => (
                      <Button
                        key={race.id}
                        variant={selectedRace?.id === race.id ? 'secondary' : 'ghost'}
                        className={`w-full justify-start text-left h-auto py-3 px-4 transition-all duration-200 ${
                          selectedRace?.id === race.id
                            ? 'bg-[rgb(var(--color-primary))/15] border-l-2 border-[rgb(var(--color-primary))] text-[rgb(var(--color-primary-light))] shadow-sm'
                            : 'border-l-2 border-transparent text-[rgb(var(--color-text-secondary))] hover:bg-[rgb(var(--color-surface-2))] hover:text-[rgb(var(--color-text-primary))]'
                        }`}
                        onClick={() => { setSelectedRace(race); setSelectedSubrace(null); }}
                      >
                        <span className="font-semibold block truncate">{race.name}</span>
                      </Button>
                    ))}
                  </div>
                </div>

                {/* Right: race details */}
                <div className="flex-1 flex flex-col overflow-hidden bg-[rgb(var(--color-surface-1))]">
                  {selectedRace ? (
                    <div className="flex-1 overflow-y-auto p-8">
                      <h1 className="text-3xl font-bold text-[rgb(var(--color-primary-light))] uppercase tracking-wide mb-2">
                        {selectedRace.name}
                      </h1>
                      <div className="h-1 w-20 bg-[rgb(var(--color-primary))] rounded-full mb-6" />

                      {/* Subraces */}
                      {subraces.length > 0 && (
                        <div className="mb-6">
                          <h4 className="text-sm uppercase tracking-wider font-bold text-[rgb(var(--color-primary-light))] border-b border-[rgb(var(--color-primary))/0.3] pb-1 mb-3">
                            Subrace
                          </h4>
                          <div className="flex flex-wrap gap-2">
                            <button
                              onClick={() => setSelectedSubrace(null)}
                              className={`px-3 py-1.5 rounded-lg text-sm border transition-colors ${selectedSubrace === null ? 'border-[rgb(var(--color-primary))] bg-[rgb(var(--color-primary)/0.1)] text-[rgb(var(--color-primary))]' : 'border-[rgb(var(--color-surface-border))] text-[rgb(var(--color-text-secondary))]'}`}
                            >
                              None
                            </button>
                            {subraces.map(sr => (
                              <button
                                key={sr.label}
                                onClick={() => setSelectedSubrace(sr.label)}
                                className={`px-3 py-1.5 rounded-lg text-sm border transition-colors ${selectedSubrace === sr.label ? 'border-[rgb(var(--color-primary))] bg-[rgb(var(--color-primary)/0.1)] text-[rgb(var(--color-primary))]' : 'border-[rgb(var(--color-surface-border))] text-[rgb(var(--color-text-secondary))]'}`}
                              >
                                {sr.name}
                              </button>
                            ))}
                          </div>
                        </div>
                      )}

                    </div>
                  ) : (
                    <div className="flex-1 flex items-center justify-center p-8 text-center">
                      <div>
                        <h3 className="text-xl font-medium text-[rgb(var(--color-text-muted))] mb-2">No Race Selected</h3>
                        <p className="text-[rgb(var(--color-text-muted))]">Select a race from the list to view details.</p>
                      </div>
                    </div>
                  )}
                </div>
              </div>

              <div className="flex justify-end gap-3 p-4 border-t border-[rgb(var(--color-surface-border))] bg-[rgb(var(--color-surface-2))/50]">
                <Button variant="outline" onClick={onClose}>Cancel</Button>
                <Button onClick={handleConfirm} disabled={!selectedRace} className="px-8 font-semibold">
                  Confirm Selection
                </Button>
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
