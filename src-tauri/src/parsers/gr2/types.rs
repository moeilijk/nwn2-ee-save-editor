use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Gr2SecurityLimits {
    pub max_file_size: usize,
    pub max_decompressed_size: usize,
    pub max_bones: u32,
}

impl Default for Gr2SecurityLimits {
    fn default() -> Self {
        Self {
            max_file_size: 50 * 1024 * 1024,
            max_decompressed_size: 100 * 1024 * 1024,
            max_bones: 500,
        }
    }
}

pub const GR2_MAGIC: [u32; 4] = [0xcab067b8, 0x0fb16df8, 0x7e8c7284, 0x1e00195e];
pub const GR2_HEADER_SIZE: u32 = 352;
pub const GR2_TAG: u32 = 0x80000015;
pub const GR2_VERSION: u32 = 6;
pub const GR2_SECTION_COUNT: u32 = 6;

#[derive(Debug, Clone)]
pub struct Gr2Header {
    pub magic: [u32; 4],
    pub size: u32,
    pub format: u32,
    pub reserved: [u32; 2],
    pub info: Gr2Info,
}

#[derive(Debug, Clone)]
pub struct Gr2Info {
    pub version: u32,
    pub file_size: u32,
    pub crc32: u32,
    pub sections_offset: u32,
    pub sections_count: u32,
    pub type_section: u32,
    pub type_offset: u32,
    pub root_section: u32,
    pub root_offset: u32,
    pub tag: u32,
    pub extra: [u32; 4],
}

#[derive(Debug, Clone)]
pub struct SectionHeader {
    pub compression: u32,
    pub data_offset: u32,
    pub data_size: u32,
    pub decompressed_size: u32,
    pub alignment: u32,
    pub first16bit: u32,
    pub first8bit: u32,
    pub relocations_offset: u32,
    pub relocations_count: u32,
    pub marshallings_offset: u32,
    pub marshallings_count: u32,
}

#[derive(Debug, Clone)]
pub struct Relocation {
    pub offset: u32,
    pub target_section: u32,
    pub target_offset: u32,
}

pub mod transform_flags {
    pub const HAS_POSITION: u32 = 0x01;
    pub const HAS_ROTATION: u32 = 0x02;
    pub const HAS_SCALE_SHEAR: u32 = 0x04;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoneTransform {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

impl Default for BoneTransform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gr2Bone {
    pub name: String,
    pub parent_index: i32,
    pub transform: BoneTransform,
    pub inverse_world_4x4: [f32; 16],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gr2Skeleton {
    pub name: String,
    pub bones: Vec<Gr2Bone>,
}

#[derive(Debug, Clone)]
pub struct Gr2Track {
    pub bone_name: String,
    pub position_keys: Vec<(f32, [f32; 3])>,
    pub rotation_keys: Vec<(f32, [f32; 4])>,
    pub scale_keys: Vec<(f32, [f32; 3])>,
}

#[derive(Debug, Clone)]
pub struct Gr2Animation {
    pub name: String,
    pub duration: f32,
    pub time_step: f32,
    pub tracks: Vec<Gr2Track>,
}
