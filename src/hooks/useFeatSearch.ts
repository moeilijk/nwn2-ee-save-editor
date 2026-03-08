
import { useMemo, useCallback } from 'react';

export function useFeatSearch<T extends { id: number; label: string; name: string; description?: string }>(
  feats: T[],
  searchTerm: string,
) {
  const searchResults = useMemo(() => {
    if (!searchTerm || searchTerm.trim().length < 3) {
      return feats;
    }

    const needle = searchTerm.trim().toLowerCase();
    return feats.filter(feat => {
      if (feat.name.toLowerCase().includes(needle)) return true;
      if (feat.label.toLowerCase().includes(needle)) return true;
      if (feat.description && feat.description.toLowerCase().includes(needle)) return true;
      return false;
    });
  }, [feats, searchTerm]);

  const highlightSearchTerm = useCallback((text: string): string => {
    if (!searchTerm || searchTerm.trim().length < 3) {
      return text;
    }
    const regex = new RegExp(`(${searchTerm.trim()})`, 'gi');
    return text.replace(regex, '<mark>$1</mark>');
  }, [searchTerm]);

  const isSearching = searchTerm && searchTerm.trim().length >= 3;

  const searchStats = useMemo(() => ({
    totalResults: searchResults.length,
    totalFeats: feats.length,
    isFiltered: isSearching,
    searchTerm: searchTerm.trim(),
  }), [searchResults.length, feats.length, isSearching, searchTerm]);

  return {
    searchResults,
    isSearching,
    searchStats,
    highlightSearchTerm,
  };
}
