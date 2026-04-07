export const T = {
  bg: '#f0eadd',
  surface: '#faf7f2',
  surfaceAlt: '#f5f0e8',
  sidebar: '#1e1e22',
  sidebarText: '#c8c4bc',
  sidebarActive: '#e8dcc8',
  sidebarAccent: '#a0522d',
  navbar: '#2a2a2e',
  text: '#2d2d30',
  textMuted: '#4a453c',
  border: '#d9d0c1',
  borderLight: '#e5ddd0',
  accent: '#a0522d',
  accentLight: '#c4865a',
  positive: '#2e7d32',
  negative: '#c62828',
  gold: '#b8952f',
  sectionBg: '#eee8da',
  sectionBorder: '#c9bfae',
} as const;

export const FEAT_TYPE_COLORS: Record<string, string> = {
  General: '#43a047', Proficiency: '#6d4c41', 'Skill/Save': '#00acc1',
  Metamagic: '#8e24aa', Divine: '#f9a825', Epic: '#e53935',
  Class: '#1e88e5', Background: '#00897b', Spellcasting: '#5c6bc0',
  History: '#f57f17', Heritage: '#c62828', 'Item Creation': '#689f38',
  Racial: '#00838f', Domain: '#7b1fa2',
};

export const SPELL_SCHOOL_COLORS: Record<string, string> = {
  Abjuration: '#1565c0', Conjuration: '#2e7d32', Enchantment: '#7b1fa2',
  Evocation: '#d84315', Transmutation: '#0277bd', Necromancy: '#b71c1c',
  Divination: '#00838f', Illusion: '#ad1457', Universal: '#546e7a',
};

export const RARITY_COLORS: Record<string, string> = {
  common: T.text,
  uncommon: T.positive,
  rare: '#1565c0',
  epic: '#6a1b9a',
};

export function formatBytes(bytes: number) {
  if (bytes === 0) return '-';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

const NOISE_SVG = encodeURIComponent(
  `<svg xmlns='http://www.w3.org/2000/svg' width='200' height='200'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='4' stitchTiles='stitch'/></filter><rect width='100%' height='100%' filter='url(#n)' opacity='0.15'/></svg>`
);
export const PATTERN_BG = `url("data:image/svg+xml,${NOISE_SVG}")`;
