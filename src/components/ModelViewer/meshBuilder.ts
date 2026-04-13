import * as THREE from 'three';
import type { AnimationData, BoneData, MeshData } from './types';

export function buildSkeleton(
  skeletonData: { bones: BoneData[] },
): { skeleton: THREE.Skeleton; rootBone: THREE.Bone } {
  const bones: THREE.Bone[] = skeletonData.bones.map((b) => {
    const bone = new THREE.Bone();
    bone.name = b.name;
    bone.position.set(b.position[0], b.position[2], -b.position[1]);
    bone.quaternion.set(b.rotation[0], b.rotation[2], -b.rotation[1], b.rotation[3]);
    bone.scale.set(b.scale[0], b.scale[2], b.scale[1]);
    return bone;
  });

  skeletonData.bones.forEach((b, i) => {
    if (b.parent_index >= 0 && b.parent_index < bones.length) {
      bones[b.parent_index].add(bones[i]);
    }
  });

  const rootBone = bones.find((_, i) => skeletonData.bones[i].parent_index === -1) ?? bones[0];
  rootBone.updateMatrixWorld(true);

  const boneInverses = skeletonData.bones.map((b, i) => {
    const m = new THREE.Matrix4();
    if (b.inverse_world_4x4 && b.inverse_world_4x4.length === 16) {
      m.fromArray(b.inverse_world_4x4);
    } else {
      m.copy(bones[i].matrixWorld).invert();
    }
    return m;
  });

  return { skeleton: new THREE.Skeleton(bones, boneInverses), rootBone };
}

export function buildMesh(
  meshData: MeshData,
  material: THREE.Material,
  skeleton?: THREE.Skeleton,
  rootBone?: THREE.Bone,
): THREE.Object3D {
  const geometry = new THREE.BufferGeometry();

  const posArray = new Float32Array(meshData.positions.length);
  for (let i = 0; i < meshData.positions.length; i += 3) {
    posArray[i] = meshData.positions[i] * 100;
    posArray[i + 1] = meshData.positions[i + 2] * 100;
    posArray[i + 2] = -meshData.positions[i + 1] * 100;
  }

  const normArray = new Float32Array(meshData.normals.length);
  for (let i = 0; i < meshData.normals.length; i += 3) {
    normArray[i] = meshData.normals[i];
    normArray[i + 1] = meshData.normals[i + 2];
    normArray[i + 2] = -meshData.normals[i + 1];
  }

  geometry.setAttribute('position', new THREE.BufferAttribute(posArray, 3));
  geometry.setAttribute('normal', new THREE.BufferAttribute(normArray, 3));
  geometry.setAttribute('uv', new THREE.BufferAttribute(new Float32Array(meshData.uvs), 2));
  geometry.setIndex(new THREE.BufferAttribute(new Uint16Array(meshData.indices), 1));

  if (meshData.mesh_type === 'skin' && meshData.bone_weights && meshData.bone_indices && skeleton && rootBone) {
    geometry.setAttribute('skinWeight', new THREE.BufferAttribute(new Float32Array(meshData.bone_weights), 4));
    geometry.setAttribute('skinIndex', new THREE.BufferAttribute(new Uint16Array(meshData.bone_indices.map(Number)), 4));
    const skinnedMesh = new THREE.SkinnedMesh(geometry, material);
    skinnedMesh.name = meshData.name;
    skinnedMesh.castShadow = true;
    skinnedMesh.receiveShadow = true;
    skinnedMesh.userData.tintGroup = meshData.tint_group;
    if (!rootBone.parent) {
      skinnedMesh.add(rootBone);
    }
    skinnedMesh.bind(skeleton);
    return skinnedMesh;
  }

  const mesh = new THREE.Mesh(geometry, material);
  mesh.name = meshData.name;
  mesh.castShadow = true;
  mesh.receiveShadow = true;
  mesh.userData.tintGroup = meshData.tint_group;
  return mesh;
}

export function buildAnimationClips(
  animations: AnimationData[],
  boneNames: string[],
): THREE.AnimationClip[] {
  return animations
    .map((anim) => {
      const tracks: THREE.KeyframeTrack[] = [];

      for (const track of anim.tracks) {
        if (!boneNames.includes(track.bone_name)) continue;
        const bonePath = track.bone_name;

        if (track.rotations && track.times.length > 0) {
          const rotations = new Float32Array(track.rotations.length);
          for (let i = 0; i < track.rotations.length; i += 4) {
            rotations[i] = track.rotations[i];         // x
            rotations[i + 1] = track.rotations[i + 2]; // z -> y
            rotations[i + 2] = -track.rotations[i + 1]; // -y -> z
            rotations[i + 3] = track.rotations[i + 3]; // w
          }
          const times = new Float32Array(
            track.rotations.length / 4 === track.times.length
              ? track.times
              : evenTimes(anim.duration, track.rotations.length / 4),
          );
          tracks.push(
            new THREE.QuaternionKeyframeTrack(
              `${bonePath}.quaternion`,
              times as unknown as number[],
              rotations as unknown as number[],
            ),
          );
        }

        if (track.positions && track.times.length > 0) {
          const positions = new Float32Array(track.positions.length);
          for (let i = 0; i < track.positions.length; i += 3) {
            positions[i] = track.positions[i];
            positions[i + 1] = track.positions[i + 2];
            positions[i + 2] = -track.positions[i + 1];
          }
          const times = new Float32Array(
            track.positions.length / 3 === track.times.length
              ? track.times
              : evenTimes(anim.duration, track.positions.length / 3),
          );
          tracks.push(
            new THREE.VectorKeyframeTrack(
              `${bonePath}.position`,
              times as unknown as number[],
              positions as unknown as number[],
            ),
          );
        }

        if (track.scales && track.times.length > 0) {
          const scales = new Float32Array(track.scales.length);
          for (let i = 0; i < track.scales.length; i += 3) {
            scales[i] = track.scales[i];
            scales[i + 1] = track.scales[i + 2];
            scales[i + 2] = track.scales[i + 1];
          }
          const times = new Float32Array(
            track.scales.length / 3 === track.times.length
              ? track.times
              : evenTimes(anim.duration, track.scales.length / 3),
          );
          tracks.push(
            new THREE.VectorKeyframeTrack(
              `${bonePath}.scale`,
              times as unknown as number[],
              scales as unknown as number[],
            ),
          );
        }
      }

      if (tracks.length === 0) return null;
      return new THREE.AnimationClip(anim.name, anim.duration, tracks);
    })
    .filter((clip): clip is THREE.AnimationClip => clip !== null);
}

function evenTimes(duration: number, count: number): number[] {
  const times: number[] = [];
  for (let i = 0; i < count; i++) {
    times.push((i / Math.max(count - 1, 1)) * duration);
  }
  return times;
}
