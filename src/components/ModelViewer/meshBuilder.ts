import * as THREE from 'three';
import type { BoneData, MeshData } from './types';

export function buildSkeleton(
  skeletonData: { bones: BoneData[] },
): { skeleton: THREE.Skeleton; rootBone: THREE.Bone } {
  const bones: THREE.Bone[] = skeletonData.bones.map((b) => {
    const bone = new THREE.Bone();
    bone.name = b.name;
    bone.position.set(b.position[0] * 100, b.position[2] * 100, -b.position[1] * 100);
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
  return { skeleton: new THREE.Skeleton(bones), rootBone };
}

export function buildMesh(
  meshData: MeshData,
  material: THREE.Material,
  skeleton?: THREE.Skeleton,
  rootBone?: THREE.Bone,
  preserveInverses = false,
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
    skinnedMesh.userData.tintGroup = meshData.tint_group;
    if (!rootBone.parent) {
      skinnedMesh.add(rootBone);
    }
    if (preserveInverses) {
      skinnedMesh.bind(skeleton, new THREE.Matrix4());
    } else {
      skinnedMesh.bind(skeleton);
    }
    return skinnedMesh;
  }

  const mesh = new THREE.Mesh(geometry, material);
  mesh.name = meshData.name;
  mesh.userData.tintGroup = meshData.tint_group;
  return mesh;
}
