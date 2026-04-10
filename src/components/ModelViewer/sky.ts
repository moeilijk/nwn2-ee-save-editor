import * as THREE from 'three';

function drawSkyGradient(ctx: CanvasRenderingContext2D, w: number, h: number) {
  const gradient = ctx.createLinearGradient(0, 0, 0, h);
  gradient.addColorStop(0, '#1b5594');
  gradient.addColorStop(0.25, '#2a6cb6');
  gradient.addColorStop(0.5, '#5a9fd4');
  gradient.addColorStop(0.75, '#8cbce0');
  gradient.addColorStop(1.0, '#c2d9e8');
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, w, h);
}

function drawFog(ctx: CanvasRenderingContext2D, w: number, h: number) {
  ctx.save();

  const fogGradient = ctx.createLinearGradient(0, h * 0.7, 0, h);
  fogGradient.addColorStop(0, 'rgba(194, 217, 232, 0)');
  fogGradient.addColorStop(0.5, 'rgba(194, 217, 232, 0.12)');
  fogGradient.addColorStop(1.0, 'rgba(194, 217, 232, 0.3)');
  ctx.fillStyle = fogGradient;
  ctx.fillRect(0, 0, w, h);

  for (let i = 0; i < 3; i++) {
    const y = h * (0.75 + i * 0.07);
    const alpha = 0.03 + i * 0.015;
    const bandGradient = ctx.createLinearGradient(0, y - h * 0.04, 0, y + h * 0.04);
    bandGradient.addColorStop(0, `rgba(194, 217, 232, 0)`);
    bandGradient.addColorStop(0.5, `rgba(194, 217, 232, ${alpha})`);
    bandGradient.addColorStop(1, `rgba(194, 217, 232, 0)`);
    ctx.fillStyle = bandGradient;
    ctx.fillRect(0, y - h * 0.04, w, h * 0.08);
  }

  ctx.restore();
}

export function createEnvTexture(): THREE.Texture {
  const canvas = document.createElement('canvas');
  canvas.width = 2;
  canvas.height = 512;
  drawSkyGradient(canvas.getContext('2d')!, 2, 512);
  const texture = new THREE.CanvasTexture(canvas);
  texture.mapping = THREE.EquirectangularReflectionMapping;
  return texture;
}

export function createSkyBackground(): THREE.Texture {
  const w = 2048;
  const h = 1024;
  const canvas = document.createElement('canvas');
  canvas.width = w;
  canvas.height = h;
  const ctx = canvas.getContext('2d')!;
  drawSkyGradient(ctx, w, h);
  drawFog(ctx, w, h);

  const texture = new THREE.CanvasTexture(canvas);
  texture.mapping = THREE.EquirectangularReflectionMapping;
  return texture;
}
