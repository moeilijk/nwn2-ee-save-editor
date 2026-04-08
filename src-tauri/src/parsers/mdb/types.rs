use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MdbSecurityLimits {
    pub max_file_size: usize,
    pub max_packet_count: u32,
    pub max_vertex_count: u32,
    pub max_face_count: u32,
}

impl Default for MdbSecurityLimits {
    fn default() -> Self {
        Self {
            max_file_size: 50 * 1024 * 1024,
            max_packet_count: 1000,
            max_vertex_count: 1_000_000,
            max_face_count: 1_000_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MdbHeader {
    pub signature: [u8; 4],
    pub major_version: u16,
    pub minor_version: u16,
    pub packet_count: u32,
}

#[derive(Debug, Clone)]
pub struct PacketKey {
    pub packet_type: PacketType,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    Rigid,
    Skin,
    Collision2,
    Collision3,
    Hook,
    Walk,
    CollisionSpheres,
    Terrain,
    Helm,
    Hair,
}

impl PacketType {
    pub fn from_bytes(bytes: &[u8; 4]) -> Option<Self> {
        match bytes {
            b"RIGD" => Some(Self::Rigid),
            b"SKIN" => Some(Self::Skin),
            b"COL2" => Some(Self::Collision2),
            b"COL3" => Some(Self::Collision3),
            b"HOOK" => Some(Self::Hook),
            b"WALK" => Some(Self::Walk),
            b"COLS" => Some(Self::CollisionSpheres),
            b"TRRN" => Some(Self::Terrain),
            b"HELM" => Some(Self::Helm),
            b"HAIR" => Some(Self::Hair),
            _ => None,
        }
    }
}

pub mod material_flags {
    pub const ALPHA_TEST: u32 = 0x01;
    pub const ALPHA_BLEND: u32 = 0x02;
    pub const ADDITIVE_BLEND: u32 = 0x04;
    pub const ENVIRONMENT_MAPPING: u32 = 0x08;
    pub const CUTSCENE_MESH: u32 = 0x10;
    pub const GLOW: u32 = 0x20;
    pub const CAST_NO_SHADOWS: u32 = 0x40;
    pub const PROJECTED_TEXTURES: u32 = 0x80;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub diffuse_map_name: String,
    pub normal_map_name: String,
    pub tint_map_name: String,
    pub glow_map_name: String,
    pub diffuse_color: [f32; 3],
    pub specular_color: [f32; 3],
    pub specular_level: f32,
    pub specular_power: f32,
    pub flags: u32,
}

#[derive(Debug, Clone)]
pub struct RigidVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 3],
    pub binormal: [f32; 3],
    pub uvw: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct SkinVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub bone_weights: [f32; 4],
    pub bone_indices: [u8; 4],
    pub tangent: [f32; 3],
    pub binormal: [f32; 3],
    pub uvw: [f32; 3],
    pub bone_count: f32,
}

#[derive(Debug, Clone)]
pub struct Face {
    pub indices: [u16; 3],
}

#[derive(Debug, Clone)]
pub struct RigidMeshPacket {
    pub name: String,
    pub material: Material,
    pub vertices: Vec<RigidVertex>,
    pub faces: Vec<Face>,
}

#[derive(Debug, Clone)]
pub struct SkinMeshPacket {
    pub name: String,
    pub skeleton_name: String,
    pub material: Material,
    pub vertices: Vec<SkinVertex>,
    pub faces: Vec<Face>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HairShorteningBehavior {
    Low = 0,
    Short = 1,
    Ponytail = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelmHidingBehavior {
    NoneHidden = 0,
    HairHidden = 1,
    PartialHair = 2,
    HeadHidden = 3,
}

#[derive(Debug, Clone)]
pub struct HookPacket {
    pub name: String,
    pub point_type: u16,
    pub point_size: u16,
    pub position: [f32; 3],
    pub orientation: [[f32; 3]; 3],
}

#[derive(Debug, Clone)]
pub struct HairPacket {
    pub name: String,
    pub shortening_behavior: HairShorteningBehavior,
    pub position: [f32; 3],
    pub orientation: [[f32; 3]; 3],
}

#[derive(Debug, Clone)]
pub struct HelmPacket {
    pub name: String,
    pub hiding_behavior: HelmHidingBehavior,
    pub position: [f32; 3],
    pub orientation: [[f32; 3]; 3],
}

#[derive(Debug, Clone)]
pub struct MdbFile {
    pub header: MdbHeader,
    pub rigid_meshes: Vec<RigidMeshPacket>,
    pub skin_meshes: Vec<SkinMeshPacket>,
    pub hooks: Vec<HookPacket>,
    pub hair: Vec<HairPacket>,
    pub helm: Vec<HelmPacket>,
}
