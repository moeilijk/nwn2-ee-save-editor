import { useEffect, useRef, useCallback, useState } from 'react';
import * as THREE from 'three';
import { invoke } from '@tauri-apps/api/core';
import { createMaterial, updateTintUniforms, type TintColors } from '../ModelViewer/materials';
import { buildSkeleton, buildMesh } from '../ModelViewer/meshBuilder';
import { useThreeScene, clearSceneModels, frameBounds } from '../ModelViewer/useThreeScene';
import type { MeshData, ModelData } from '../ModelViewer/types';
import type { TintChannels } from '@/lib/bindings';
import { Spinner } from '@blueprintjs/core';

function tintChannelsToColors(tc: TintChannels): TintColors {
  return {
    channel1: [tc.channel1.r / 255, tc.channel1.g / 255, tc.channel1.b / 255],
    channel2: [tc.channel2.r / 255, tc.channel2.g / 255, tc.channel2.b / 255],
    channel3: [tc.channel3.r / 255, tc.channel3.g / 255, tc.channel3.b / 255],
  };
}

type PartType = 'head' | 'hair' | 'fhair' | 'wings' | 'tail' | 'helm' | 'body' | 'cloak';

interface CharacterViewer3DProps {
  refreshKey: number;
  refreshPart: { parts: PartType[]; key: number } | null;
  tintHead: TintChannels;
  tintHair: TintChannels;
  height: number;
  girth: number;
}

export function CharacterViewer3D({ refreshKey, refreshPart, tintHead, tintHair, height, girth }: CharacterViewer3DProps) {
  const skeletonRef = useRef<{ skeleton: THREE.Skeleton; rootBone: THREE.Bone } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const tintHeadRef = useRef(tintHead);
  const tintHairRef = useRef(tintHair);
  const heightRef = useRef(height);
  const girthRef = useRef(girth);
  tintHeadRef.current = tintHead;
  tintHairRef.current = tintHair;
  heightRef.current = height;
  girthRef.current = girth;

  const onAnimate = useCallback((scene: THREE.Scene) => {
    const model = scene.getObjectByName('__model');
    if (model) {
      model.scale.set(girthRef.current, heightRef.current, girthRef.current);
    }
  }, []);

  const { container: containerRef, scene: sceneRef, camera: cameraRef, controls: controlsRef } = useThreeScene(onAnimate);

  function getTintColors(): Record<string, TintColors> {
    const headColors = tintChannelsToColors(tintHeadRef.current);
    const hairColors = tintChannelsToColors(tintHairRef.current);
    const white: TintColors = { channel1: [1, 1, 1], channel2: [1, 1, 1], channel3: [1, 1, 1] };
    const fhairColors: TintColors = { channel1: hairColors.channel1, channel2: [1, 1, 1], channel3: [1, 1, 1] };
    return { head: headColors, hair: hairColors, fhair: fhairColors, body: white };
  }

  const partGroupName = (part: string) => `__part_${part}`;

  async function buildPartGroup(
    meshes: MeshData[],
    partName: string,
    tintMap: Record<string, TintColors>,
    skeleton?: THREE.Skeleton,
    rootBone?: THREE.Bone,
    preserveInverses = false,
  ): Promise<THREE.Group> {
    const group = new THREE.Group();
    group.name = partGroupName(partName);
    for (const meshData of meshes) {
      if (/_L\d+$/i.test(meshData.name)) continue;
      const colors = tintMap[meshData.tint_group];
      const material = await createMaterial(meshData.material, colors);
      const obj = buildMesh(meshData, material, skeleton, rootBone, preserveInverses);
      group.add(obj);
    }
    return group;
  }

  const loadCharacter = useCallback(async () => {
    const scene = sceneRef.current;
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    if (!scene || !camera || !controls) return;

    clearSceneModels(scene);
    setLoading(true);
    setError(null);

    try {
      const data: ModelData = await invoke('load_character_model');
      const tintMap = getTintColors();

      let skeleton: THREE.Skeleton | undefined;
      let rootBone: THREE.Bone | undefined;

      if (data.skeleton) {
        const result = buildSkeleton(data.skeleton);
        skeleton = result.skeleton;
        rootBone = result.rootBone;
        skeletonRef.current = result;
      }

      const partBuckets: Record<string, MeshData[]> = {};
      for (const meshData of data.meshes) {
        (partBuckets[meshData.part] ??= []).push(meshData);
      }

      const modelGroup = new THREE.Group();
      modelGroup.name = '__model';

      for (const [partName, meshes] of Object.entries(partBuckets)) {
        const partGroup = await buildPartGroup(meshes, partName, tintMap, skeleton, rootBone);
        modelGroup.add(partGroup);
      }
      scene.add(modelGroup);

      frameBounds(camera, controls, scene, modelGroup);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const replacePart = useCallback(async (part: PartType) => {
    const scene = sceneRef.current;
    if (!scene) return;

    const modelGroup = scene.getObjectByName('__model');
    if (!modelGroup) return;

    try {
      const data: ModelData = await invoke('load_character_part', { part });
      const tintMap = getTintColors();
      const skel = skeletonRef.current;

      const newGroup = data.meshes.length > 0
        ? await buildPartGroup(data.meshes, part, tintMap, skel?.skeleton, skel?.rootBone, true)
        : null;

      const old = modelGroup.getObjectByName(partGroupName(part));
      if (old) modelGroup.remove(old);
      if (newGroup) modelGroup.add(newGroup);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    loadCharacter();
  }, [refreshKey, loadCharacter]);

  useEffect(() => {
    if (refreshPart) {
      (async () => {
        for (const part of refreshPart.parts) {
          await replacePart(part);
        }
      })();
    }
  }, [refreshPart, replacePart]);

  useEffect(() => {
    const scene = sceneRef.current;
    if (!scene) return;
    const headColors = tintChannelsToColors(tintHead);
    const hairColors = tintChannelsToColors(tintHair);
    const fhairColors: TintColors = { channel1: hairColors.channel1, channel2: [1, 1, 1], channel3: [1, 1, 1] };
    updateTintUniforms(scene, 'head', headColors);
    updateTintUniforms(scene, 'hair', hairColors);
    updateTintUniforms(scene, 'fhair', fhairColors);
  }, [tintHead, tintHair, sceneRef]);

  return (
    <div style={{ position: 'relative', width: '100%', height: '100%', minHeight: 400 }}>
      <div ref={containerRef} style={{ width: '100%', height: '100%' }} />
      {loading && (
        <div style={{ position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)' }}>
          <Spinner />
        </div>
      )}
      {error && (
        <div style={{ position: 'absolute', top: 8, left: 8, color: '#ff4444', fontSize: 12 }}>
          {error}
        </div>
      )}
    </div>
  );
}
