import { useEffect, useRef, useCallback, useState } from 'react';
import * as THREE from 'three';
import { invoke } from '@tauri-apps/api/core';
import { createMaterial, updateTintUniforms, type TintColors } from '../ModelViewer/materials';
import { buildSkeleton, buildMesh, buildAnimationClips } from '../ModelViewer/meshBuilder';
import { useTranslations } from '@/hooks/useTranslations';
import { useThreeScene, clearSceneModels, frameBounds } from '../ModelViewer/useThreeScene';
import type { AttachedPart, MeshData, ModelData } from '../ModelViewer/types';
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
  tintCloak?: TintChannels | null;
  tintArmor?: TintChannels | null;
  height: number;
  girth: number;
}

export function CharacterViewer3D({ refreshKey, refreshPart, tintHead, tintHair, tintCloak, tintArmor, height, girth }: CharacterViewer3DProps) {
  const t = useTranslations();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const skeletonRef = useRef<{ skeleton: THREE.Skeleton; rootBone: THREE.Bone } | null>(null);
  const mixerRef = useRef<THREE.AnimationMixer | null>(null);
  const attachedMixersRef = useRef<Map<string, THREE.AnimationMixer>>(new Map());
  const timerRef = useRef<THREE.Timer>(new THREE.Timer());
  const idleTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const tintHeadRef = useRef(tintHead);
  const tintHairRef = useRef(tintHair);
  const tintCloakRef = useRef(tintCloak);
  const tintArmorRef = useRef(tintArmor);
  const heightRef = useRef(height);
  const girthRef = useRef(girth);
  tintHeadRef.current = tintHead;
  tintHairRef.current = tintHair;
  tintCloakRef.current = tintCloak;
  tintArmorRef.current = tintArmor;
  heightRef.current = height;
  girthRef.current = girth;

  const onAnimate = useCallback((scene: THREE.Scene) => {
    const model = scene.getObjectByName('__model');
    if (model) {
      model.scale.set(girthRef.current, heightRef.current, girthRef.current);
    }
    timerRef.current.update();
    const delta = timerRef.current.getDelta();
    if (mixerRef.current) mixerRef.current.update(delta);
    for (const m of attachedMixersRef.current.values()) m.update(delta);
  }, []);

  const { container: containerRef, scene: sceneRef, camera: cameraRef, controls: controlsRef } = useThreeScene(onAnimate);

  function getTintColors(): Record<string, TintColors> {
    const headColors = tintChannelsToColors(tintHeadRef.current);
    const hairColors = tintChannelsToColors(tintHairRef.current);
    const white: TintColors = { channel1: [1, 1, 1], channel2: [1, 1, 1], channel3: [1, 1, 1] };
    const fhairColors: TintColors = { channel1: hairColors.channel1, channel2: [1, 1, 1], channel3: [1, 1, 1] };
    const cloakColors = tintCloakRef.current ? tintChannelsToColors(tintCloakRef.current) : white;
    const armorColors = tintArmorRef.current ? tintChannelsToColors(tintArmorRef.current) : white;
    return { head: headColors, hair: hairColors, fhair: fhairColors, body: armorColors, cloak: cloakColors };
  }

  const partGroupName = (part: string) => `__part_${part}`;

  async function buildPartGroup(
    meshes: MeshData[],
    partName: string,
    tintMap: Record<string, TintColors>,
    skeleton?: THREE.Skeleton,
    rootBone?: THREE.Bone,
  ): Promise<THREE.Group> {
    const group = new THREE.Group();
    group.name = partGroupName(partName);
    for (const meshData of meshes) {
      if (/_L\d+$/i.test(meshData.name)) continue;
      const colors = tintMap[meshData.tint_group];
      const material = await createMaterial(meshData.material, colors);
      const obj = buildMesh(meshData, material, skeleton, rootBone);
      group.add(obj);
    }
    return group;
  }

  async function buildAttachedPart(
    attached: AttachedPart,
    tintMap: Record<string, TintColors>,
  ): Promise<{ group: THREE.Group; mixer: THREE.AnimationMixer | null } | null> {
    if (!attached.skeleton) return null;
    const { skeleton: attSkel, rootBone: attRoot } = buildSkeleton(attached.skeleton);
    const group = await buildPartGroup(attached.meshes, attached.name, tintMap, attSkel, attRoot);

    let mixer: THREE.AnimationMixer | null = null;
    if (attached.animations.length > 0) {
      const boneNames = attSkel.bones.map((b) => b.name);
      const clips = buildAnimationClips(attached.animations, boneNames);
      const idleClip = clips.find((c) => c.name.toLowerCase().includes('idle')) ?? clips[0];
      if (idleClip) {
        mixer = new THREE.AnimationMixer(group);
        const action = mixer.clipAction(idleClip);
        action.setLoop(THREE.LoopRepeat, Infinity);
        action.play();
      }
    }
    return { group, mixer };
  }

  function disposeAttachedMixer(partName: string) {
    const mixer = attachedMixersRef.current.get(partName);
    if (mixer) {
      mixer.stopAllAction();
      attachedMixersRef.current.delete(partName);
    }
  }

  const loadCharacter = useCallback(async () => {
    const scene = sceneRef.current;
    const camera = cameraRef.current;
    const controls = controlsRef.current;
    if (!scene || !camera || !controls) return;

    if (idleTimerRef.current) {
      clearTimeout(idleTimerRef.current);
      idleTimerRef.current = null;
    }
    if (mixerRef.current) {
      mixerRef.current.stopAllAction();
      mixerRef.current = null;
    }
    for (const m of attachedMixersRef.current.values()) m.stopAllAction();
    attachedMixersRef.current.clear();

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

      // Pre-compute all materials in parallel
      const allMeshEntries: { meshData: MeshData; partName: string }[] = [];
      for (const [partName, meshes] of Object.entries(partBuckets)) {
        for (const meshData of meshes) {
          if (/_L\d+$/i.test(meshData.name)) continue;
          allMeshEntries.push({ meshData, partName });
        }
      }

      const materialPromises = allMeshEntries.map(({ meshData }) => {
        const colors = tintMap[meshData.tint_group];
        return createMaterial(meshData.material, colors);
      });
      const allMaterials = await Promise.all(materialPromises);

      const modelGroup = new THREE.Group();
      modelGroup.name = '__model';

      const partGroups = new Map<string, THREE.Group>();
      for (let i = 0; i < allMeshEntries.length; i++) {
        const { meshData, partName } = allMeshEntries[i];
        const material = allMaterials[i];

        let group = partGroups.get(partName);
        if (!group) {
          group = new THREE.Group();
          group.name = partGroupName(partName);
          partGroups.set(partName, group);
          modelGroup.add(group);
        }

        const obj = buildMesh(meshData, material, skeleton, rootBone);
        group.add(obj);
      }


      for (const attached of data.attached_parts ?? []) {
        const built = await buildAttachedPart(attached, tintMap);
        if (!built) continue;
        modelGroup.add(built.group);
        if (built.mixer) attachedMixersRef.current.set(attached.name, built.mixer);
      }

      scene.add(modelGroup);
      frameBounds(camera, controls, scene, modelGroup);

      // Set up idle animation with fidget rotation
      if (data.animations && data.animations.length > 0 && skeletonRef.current) {
        const boneNames = skeletonRef.current.skeleton.bones.map((b) => b.name);

        const clips = buildAnimationClips(data.animations, boneNames);
        if (clips.length > 0) {
          const mixer = new THREE.AnimationMixer(modelGroup);
          mixerRef.current = mixer;
          timerRef.current = new THREE.Timer();

          const idleClips = clips.filter((c) => {
            const n = c.name.toLowerCase();
            const isFidget = n.includes('fidget') || n.includes('fid_');
            return n.includes('idle') && !isFidget;
          });
          const fidgetClips = clips.filter((c) => {
             const n = c.name.toLowerCase();
             return n.includes('fidget') || n.includes('fid_');
          });

          if (idleClips.length === 0 && clips.length > 0) {
            // If no clear idle found, use the first clip that isn't a fidget, 
            // or just the first clip if all are fidgets.
            const fallback = clips.find(c => !(c.name.toLowerCase().includes('fidget') || c.name.toLowerCase().includes('fid_'))) || clips[0];
            idleClips.push(fallback);
          }

          const actions = idleClips.map((c) => {
            const a = mixer.clipAction(c);
            a.setLoop(THREE.LoopOnce, 1);
            a.clampWhenFinished = true;
            return a;
          });
          const fidgetActions = fidgetClips.map((c) => {
            const a = mixer.clipAction(c);
            a.setLoop(THREE.LoopOnce, 1);
            a.clampWhenFinished = true;
            return a;
          });

          let currentAction: THREE.AnimationAction | null = null;
          let lastFidgetIdx = -1;
          let lastWasFidget = false;
          const playNext = () => {
            const useFidget = !lastWasFidget && Math.random() < 0.1 && fidgetActions.length > 0;
            const pool = useFidget ? fidgetActions : actions;
            
            let idx = Math.floor(Math.random() * pool.length);
            if (useFidget && fidgetActions.length > 1 && idx === lastFidgetIdx) {
              idx = (idx + 1) % fidgetActions.length;
            }
            if (useFidget) lastFidgetIdx = idx;

            const next = pool[idx];
            lastWasFidget = useFidget;

            if (currentAction && currentAction !== next) {
              currentAction.crossFadeTo(next, 0.3, true);
            }
            next.reset().play();
            currentAction = next;
          };

          mixer.addEventListener('finished', (e: any) => {
            if (e.action === currentAction) {
              playNext();
            }
          });

          playNext();
        }
      }


    } catch (err) {
      setError(err instanceof Error ? err.message : typeof err === 'object' ? JSON.stringify(err) : String(err));
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

      const old = modelGroup.getObjectByName(partGroupName(part));
      if (old) modelGroup.remove(old);
      disposeAttachedMixer(part);

      const attached = data.attached_parts?.find((p) => p.name === part);
      if (attached) {
        const built = await buildAttachedPart(attached, tintMap);
        if (built) {
          modelGroup.add(built.group);
          if (built.mixer) attachedMixersRef.current.set(part, built.mixer);
        }
      } else if (data.meshes.length > 0) {
        const newGroup = await buildPartGroup(data.meshes, part, tintMap, skel?.skeleton, skel?.rootBone);
        modelGroup.add(newGroup);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : typeof err === 'object' ? JSON.stringify(err) : String(err));
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
        <div className="t-base" style={{ position: 'absolute', top: 8, left: 8, color: '#ff4444' }}>
          {error}
        </div>
      )}
    </div>
  );
}
