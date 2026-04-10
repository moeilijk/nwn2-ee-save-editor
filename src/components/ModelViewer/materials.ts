import * as THREE from 'three';
import { DDSLoader } from 'three/examples/jsm/loaders/DDSLoader.js';
import { invoke } from '@tauri-apps/api/core';

const ddsLoader = new DDSLoader();

export interface TintColors {
  channel1: [number, number, number];
  channel2: [number, number, number];
  channel3: [number, number, number];
}

export async function loadDDSTexture(textureName: string): Promise<THREE.CompressedTexture | null> {
  if (!textureName) return null;

  try {
    const bytes: number[] = await invoke('get_texture_bytes', { name: textureName });
    const buffer = new Uint8Array(bytes).buffer;
    const dds = ddsLoader.parse(buffer, false);

    const texture = new THREE.CompressedTexture(
      dds.mipmaps,
      dds.width,
      dds.height,
      dds.format as THREE.CompressedPixelFormat,
    );
    texture.needsUpdate = true;
    texture.wrapS = THREE.RepeatWrapping;
    texture.wrapT = THREE.RepeatWrapping;

    return texture;
  } catch (err) {
    console.warn(`[texture] Failed to load '${textureName}':`, err);
    return null;
  }
}

function injectTintShader(
  mat: THREE.MeshStandardMaterial,
  tintMapTexture: THREE.CompressedTexture,
  tintColors: TintColors,
): void {
  const uniforms = {
    tintMap: { value: tintMapTexture },
    tintColor1: { value: new THREE.Vector3(...tintColors.channel1) },
    tintColor2: { value: new THREE.Vector3(...tintColors.channel2) },
    tintColor3: { value: new THREE.Vector3(...tintColors.channel3) },
  };

  (mat.userData as Record<string, unknown>).tintUniforms = uniforms;

  mat.onBeforeCompile = (shader) => {
    shader.uniforms.tintMap = uniforms.tintMap;
    shader.uniforms.tintColor1 = uniforms.tintColor1;
    shader.uniforms.tintColor2 = uniforms.tintColor2;
    shader.uniforms.tintColor3 = uniforms.tintColor3;

    shader.fragmentShader = shader.fragmentShader.replace(
      '#include <common>',
      `#include <common>
uniform sampler2D tintMap;
uniform vec3 tintColor1;
uniform vec3 tintColor2;
uniform vec3 tintColor3;
`,
    );

    // NWN2 tint_map: R/G/B channels are masks for 3 independent tint colors.
    // Each channel blends between white (no tint) and the tint color.
    // Applied as a multiply on the diffuse color after texture sampling.
    shader.fragmentShader = shader.fragmentShader.replace(
      '#include <color_fragment>',
      `#include <color_fragment>
{
  #if defined(USE_MAP)
    vec2 tintUv = vMapUv;
  #elif defined(USE_NORMALMAP)
    vec2 tintUv = vNormalMapUv;
  #else
    vec2 tintUv = vec2(0.0);
  #endif
  vec4 tintMask = texture2D(tintMap, tintUv);
  vec3 tint = mix(vec3(1.0), tintColor1, tintMask.r)
            * mix(vec3(1.0), tintColor2, tintMask.g)
            * mix(vec3(1.0), tintColor3, tintMask.b);
  diffuseColor.rgb *= tint;
}
`,
    );
  };
}

export function updateTintUniforms(
  scene: THREE.Scene,
  tintGroup: string,
  tintColors: TintColors,
): void {
  scene.traverse((obj) => {
    if (!(obj instanceof THREE.Mesh || obj instanceof THREE.SkinnedMesh)) return;
    if (obj.userData.tintGroup !== tintGroup) return;

    const mat = obj.material as THREE.MeshStandardMaterial;
    const uniforms = (mat.userData as Record<string, unknown>).tintUniforms as
      | Record<string, { value: THREE.Vector3 }>
      | undefined;
    if (!uniforms) return;

    uniforms.tintColor1.value.set(...tintColors.channel1);
    uniforms.tintColor2.value.set(...tintColors.channel2);
    uniforms.tintColor3.value.set(...tintColors.channel3);
  });
}

export async function createMaterial(
  materialData: { diffuse_map: string; normal_map: string; tint_map?: string; glow_map: string; flags: number },
  tintColors?: TintColors,
): Promise<THREE.MeshStandardMaterial> {
  const mat = new THREE.MeshStandardMaterial({
    side: THREE.DoubleSide,
    roughness: 0.85,
    metalness: 0.0,
    envMapIntensity: 0.3,
  });

  const diffuse = await loadDDSTexture(materialData.diffuse_map);
  if (diffuse) {
    mat.map = diffuse;
  }

  if (materialData.tint_map && tintColors) {
    const tintTex = await loadDDSTexture(materialData.tint_map);
    if (tintTex) {
      if (!diffuse) {
        const white = new THREE.DataTexture(new Uint8Array([255, 255, 255, 255]), 1, 1);
        white.needsUpdate = true;
        mat.map = white;
      }
      injectTintShader(mat, tintTex, tintColors);
    }
  } else if (!diffuse && materialData.tint_map) {
    const tint = await loadDDSTexture(materialData.tint_map);
    if (tint) {
      mat.map = tint;
    }
  }

  const normal = await loadDDSTexture(materialData.normal_map);
  if (normal) {
    mat.normalMap = normal;
  }

  if (materialData.flags & 0x01) {
    mat.alphaTest = 0.5;
    mat.transparent = true;
  }

  if (materialData.flags & 0x20) {
    const glow = await loadDDSTexture(materialData.glow_map);
    if (glow) {
      mat.emissiveMap = glow;
      mat.emissive = new THREE.Color(1, 1, 1);
    }
  }

  return mat;
}
