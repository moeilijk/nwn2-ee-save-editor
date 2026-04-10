import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

const iconCache = new Map<string, string>();
const failedIcons = new Set<string>();
const pendingRequests = new Map<string, Promise<string>>();

export async function fetchIcon(resref: string): Promise<string> {
  const cached = iconCache.get(resref);
  if (cached) return cached;

  if (failedIcons.has(resref)) return '';

  const pending = pendingRequests.get(resref);
  if (pending) return pending;

  const request = invoke<string>('get_icon_png', { name: resref })
    .then((dataUrl) => {
      iconCache.set(resref, dataUrl);
      pendingRequests.delete(resref);
      return dataUrl;
    })
    .catch((err) => {
      pendingRequests.delete(resref);
      failedIcons.add(resref);
      console.warn(`[icon] Failed to load '${resref}':`, err);
      return '';
    });

  pendingRequests.set(resref, request);
  return request;
}

export function useIcon(resref: string | null | undefined): string {
  const [dataUrl, setDataUrl] = useState<string>(() => {
    if (!resref) return '';
    return iconCache.get(resref) || '';
  });

  useEffect(() => {
    if (!resref) {
      setDataUrl('');
      return;
    }

    const cached = iconCache.get(resref);
    if (cached) {
      setDataUrl(cached);
      return;
    }

    let cancelled = false;
    fetchIcon(resref).then((url) => {
      if (!cancelled) setDataUrl(url);
    });

    return () => { cancelled = true; };
  }, [resref]);

  return dataUrl;
}
