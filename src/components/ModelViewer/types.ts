export interface MaterialData {
  diffuse_map: string;
  normal_map: string;
  tint_map: string;
  glow_map: string;
  flags: number;
}

export interface MeshData {
  name: string;
  mesh_type: string;
  part: string;
  tint_group: string;
  positions: number[];
  normals: number[];
  uvs: number[];
  tangents: number[];
  indices: number[];
  bone_weights: number[] | null;
  bone_indices: number[] | null;
  material: MaterialData;
}

export interface BoneData {
  name: string;
  parent_index: number;
  position: [number, number, number];
  rotation: [number, number, number, number];
  scale: [number, number, number];
}

export interface HookData {
  name: string;
  position: [number, number, number];
  orientation: [[number, number, number], [number, number, number], [number, number, number]];
}

export interface HairData {
  name: string;
  shortening_behavior: number;
  position: [number, number, number];
  orientation: [[number, number, number], [number, number, number], [number, number, number]];
}

export interface HelmData {
  name: string;
  hiding_behavior: number;
  position: [number, number, number];
  orientation: [[number, number, number], [number, number, number], [number, number, number]];
}

export interface ModelData {
  meshes: MeshData[];
  hooks: HookData[];
  hair: HairData[];
  helm: HelmData[];
  skeleton: { bones: BoneData[] } | null;
}
