use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::parsers::gr2::Gr2Parser;
use crate::parsers::mdb::MdbParser;
use crate::parsers::mdb::types::{RigidMeshPacket, SkinMeshPacket};
use crate::services::resource_manager::ResourceManager;

fn guess_body_skeleton(prefix: &str) -> Option<&'static str> {
    match prefix {
        "P_AAM_" | "P_ASM_" | "P_EDM_" | "P_EEM_" | "P_EHM_" | "P_ELM_" | "P_ERM_" | "P_ESM_"
        | "P_EWM_" | "P_HAM_" | "P_HEM_" | "P_HFM_" | "P_HHM_" | "P_HIM_" | "P_HPM_" | "P_HTM_"
        | "P_HWM_" => Some("P_HHM_skel"),

        "P_AAF_" | "P_ASF_" | "P_EDF_" | "P_EEF_" | "P_EHF_" | "P_ELF_" | "P_ERF_" | "P_ESF_"
        | "P_EWF_" | "P_HAF_" | "P_HEF_" | "P_HFF_" | "P_HHF_" | "P_HIF_" | "P_HPF_" | "P_HTF_"
        | "P_HWF_" => Some("P_HHF_skel"),

        "P_DDM_" | "P_DGM_" | "P_DUM_" => Some("P_DDM_skel"),
        "P_DDF_" | "P_DGF_" | "P_DUF_" => Some("P_DDF_skel"),

        "P_GGM_" | "P_GSM_" => Some("P_GGM_skel"),
        "P_GGF_" | "P_GSF_" => Some("P_GGF_skel"),

        "P_OOM_" | "P_OGM_" => Some("P_OOM_skel"),
        "P_OOF_" | "P_OGF_" => Some("P_OOF_skel"),

        _ => None,
    }
}

fn guess_cloak_skeleton(prefix: &str) -> Option<&'static str> {
    match prefix {
        "P_DDM_" => Some("P_DDMcapewing_skel"),
        "P_DDF_" => Some("P_DDFcapewing_skel"),
        "P_EEM_" | "P_HHM_" => Some("P_HHMcapewing_skel"),
        "P_EEF_" | "P_HHF_" => Some("P_HHFcapewing_skel"),
        "P_GGM_" => Some("P_GGMcapewing_skel"),
        "P_GGF_" => Some("P_GGFcapewing_skel"),
        "P_OOM_" => Some("P_OOMcapewing_skel"),
        "P_OOF_" => Some("P_OOFcapewing_skel"),
        _ => None,
    }
}

fn guess_tail_skeleton(prefix: &str) -> Option<&'static str> {
    match prefix {
        "P_DDM_" => Some("P_DDMtail_skel"),
        "P_DDF_" => Some("P_DDFtail_skel"),
        "P_EEM_" | "P_HHM_" | "P_HTM_" => Some("P_HHMtail_skel"),
        "P_EEF_" | "P_HHF_" | "P_HTF_" => Some("P_HHFtail_skel"),
        "P_GGM_" => Some("P_GGMtail_skel"),
        "P_GGF_" => Some("P_GGFtail_skel"),
        "P_OOM_" => Some("P_OOMtail_skel"),
        "P_OOF_" => Some("P_OOFtail_skel"),
        _ => None,
    }
}

fn resolve_skeleton_name(mesh_name: &str, stored_skeleton: &str) -> String {
    let upper = mesh_name.to_uppercase();
    let prefix = if upper.len() >= 6 { &upper[..6] } else { "" };

    if (upper.contains("_CLOAK") || upper.contains("_WINGS"))
        && let Some(skel) = guess_cloak_skeleton(prefix)
    {
        return skel.to_string();
    }

    if upper.contains("_TAIL")
        && let Some(skel) = guess_tail_skeleton(prefix)
    {
        return skel.to_string();
    }

    if let Some(skel) = guess_body_skeleton(prefix) {
        return skel.to_string();
    }

    stored_skeleton.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelData {
    pub meshes: Vec<MeshData>,
    pub hooks: Vec<HookData>,
    pub hair: Vec<HairData>,
    pub helm: Vec<HelmData>,
    pub skeleton: Option<SkeletonData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshData {
    pub name: String,
    pub mesh_type: String,
    pub positions: Vec<f32>,
    pub normals: Vec<f32>,
    pub uvs: Vec<f32>,
    pub tangents: Vec<f32>,
    pub indices: Vec<u16>,
    pub bone_weights: Option<Vec<f32>>,
    pub bone_indices: Option<Vec<u8>>,
    pub material: MaterialData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialData {
    pub diffuse_map: String,
    pub normal_map: String,
    pub tint_map: String,
    pub glow_map: String,
    pub flags: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkeletonData {
    pub bones: Vec<BoneData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneData {
    pub name: String,
    pub parent_index: i32,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookData {
    pub name: String,
    pub position: [f32; 3],
    pub orientation: [[f32; 3]; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HairData {
    pub name: String,
    pub shortening_behavior: u32,
    pub position: [f32; 3],
    pub orientation: [[f32; 3]; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmData {
    pub name: String,
    pub hiding_behavior: u32,
    pub position: [f32; 3],
    pub orientation: [[f32; 3]; 3],
}

pub fn load_model(resource_manager: &ResourceManager, resref: &str) -> Result<ModelData, String> {
    let mdb_data = resource_manager
        .get_resource_bytes(resref, "mdb")
        .map_err(|e| format!("Failed to load MDB {resref}: {e}"))?;

    let mdb =
        MdbParser::parse(&mdb_data).map_err(|e| format!("Failed to parse MDB {resref}: {e}"))?;

    let mut meshes = Vec::new();

    for rm in &mdb.rigid_meshes {
        meshes.push(flatten_rigid_mesh(rm));
    }

    for sm in &mdb.skin_meshes {
        meshes.push(flatten_skin_mesh(sm));
    }

    let skeleton = mdb.skin_meshes.first().and_then(|sm| {
        let skeleton_name = resolve_skeleton_name(&sm.name, &sm.skeleton_name);
        debug!(
            "Resolving skeleton '{}' for mesh '{}'",
            skeleton_name, sm.name
        );

        match resource_manager.get_resource_bytes(&skeleton_name, "gr2") {
            Ok(gr2_data) => match Gr2Parser::parse(&gr2_data) {
                Ok(skel) => {
                    debug!(
                        "Loaded skeleton '{}' with {} bones",
                        skel.name,
                        skel.bones.len()
                    );
                    Some(SkeletonData {
                        bones: skel
                            .bones
                            .iter()
                            .map(|b| BoneData {
                                name: b.name.clone(),
                                parent_index: b.parent_index,
                                position: b.transform.position,
                                rotation: b.transform.rotation,
                                scale: b.transform.scale,
                            })
                            .collect(),
                    })
                }
                Err(e) => {
                    warn!("Failed to parse GR2 {}: {}", skeleton_name, e);
                    None
                }
            },
            Err(e) => {
                warn!("Skeleton not found {}: {}", skeleton_name, e);
                None
            }
        }
    });

    let hooks = mdb
        .hooks
        .iter()
        .map(|h| HookData {
            name: h.name.clone(),
            position: h.position,
            orientation: h.orientation,
        })
        .collect();

    let hair = mdb
        .hair
        .iter()
        .map(|h| HairData {
            name: h.name.clone(),
            shortening_behavior: h.shortening_behavior as u32,
            position: h.position,
            orientation: h.orientation,
        })
        .collect();

    let helm = mdb
        .helm
        .iter()
        .map(|h| HelmData {
            name: h.name.clone(),
            hiding_behavior: h.hiding_behavior as u32,
            position: h.position,
            orientation: h.orientation,
        })
        .collect();

    Ok(ModelData {
        meshes,
        hooks,
        hair,
        helm,
        skeleton,
    })
}

fn flatten_rigid_mesh(rm: &RigidMeshPacket) -> MeshData {
    let vc = rm.vertices.len();
    let mut positions = Vec::with_capacity(vc * 3);
    let mut normals = Vec::with_capacity(vc * 3);
    let mut uvs = Vec::with_capacity(vc * 2);
    let mut tangents = Vec::with_capacity(vc * 3);

    for v in &rm.vertices {
        positions.extend_from_slice(&v.position);
        normals.extend_from_slice(&v.normal);
        tangents.extend_from_slice(&v.tangent);
        uvs.push(v.uvw[0]);
        uvs.push(v.uvw[1]);
    }

    let indices: Vec<u16> = rm.faces.iter().flat_map(|f| f.indices).collect();

    MeshData {
        name: rm.name.clone(),
        mesh_type: "rigid".to_string(),
        positions,
        normals,
        uvs,
        tangents,
        indices,
        bone_weights: None,
        bone_indices: None,
        material: MaterialData {
            diffuse_map: rm.material.diffuse_map_name.clone(),
            normal_map: rm.material.normal_map_name.clone(),
            tint_map: rm.material.tint_map_name.clone(),
            glow_map: rm.material.glow_map_name.clone(),
            flags: rm.material.flags,
        },
    }
}

fn flatten_skin_mesh(sm: &SkinMeshPacket) -> MeshData {
    let vc = sm.vertices.len();
    let mut positions = Vec::with_capacity(vc * 3);
    let mut normals = Vec::with_capacity(vc * 3);
    let mut uvs = Vec::with_capacity(vc * 2);
    let mut tangents = Vec::with_capacity(vc * 3);
    let mut bone_weights = Vec::with_capacity(vc * 4);
    let mut bone_indices = Vec::with_capacity(vc * 4);

    for v in &sm.vertices {
        positions.extend_from_slice(&v.position);
        normals.extend_from_slice(&v.normal);
        tangents.extend_from_slice(&v.tangent);
        uvs.push(v.uvw[0]);
        uvs.push(v.uvw[1]);
        bone_weights.extend_from_slice(&v.bone_weights);
        bone_indices.extend_from_slice(&v.bone_indices);
    }

    let indices: Vec<u16> = sm.faces.iter().flat_map(|f| f.indices).collect();

    MeshData {
        name: sm.name.clone(),
        mesh_type: "skin".to_string(),
        positions,
        normals,
        uvs,
        tangents,
        indices,
        bone_weights: Some(bone_weights),
        bone_indices: Some(bone_indices),
        material: MaterialData {
            diffuse_map: sm.material.diffuse_map_name.clone(),
            normal_map: sm.material.normal_map_name.clone(),
            tint_map: sm.material.tint_map_name.clone(),
            glow_map: sm.material.glow_map_name.clone(),
            flags: sm.material.flags,
        },
    }
}
