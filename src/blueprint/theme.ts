export const T = {
  bg: '#f2ece0',
  surface: '#faf7f2',
  surfaceAlt: '#f5f0e8',
  sidebar: '#1e1e22',
  sidebarText: '#c8c4bc',
  sidebarActive: '#e8dcc8',
  sidebarAccent: '#a0522d',
  navbar: '#2a2a2e',
  text: '#2d2d30',
  textMuted: '#8a8478',
  border: '#d9d0c1',
  borderLight: '#e5ddd0',
  accent: '#a0522d',
  accentLight: '#c4865a',
  positive: '#5a7a5a',
  negative: '#9c4040',
  gold: '#b8952f',
  sectionBg: '#eee8da',
  sectionBorder: '#c9bfae',
} as const;

const NOISE_SVG = encodeURIComponent(
  `<svg xmlns='http://www.w3.org/2000/svg' width='200' height='200'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='4' stitchTiles='stitch'/></filter><rect width='100%' height='100%' filter='url(#n)' opacity='0.15'/></svg>`
);
export const PATTERN_BG = `url("data:image/svg+xml,${NOISE_SVG}")`;
