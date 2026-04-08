use std::io::{Cursor, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use tracing::debug;

use super::error::{MdbError, MdbResult};
use super::types::{
    Face, HairPacket, HairShorteningBehavior, HelmHidingBehavior, HelmPacket, HookPacket, Material,
    MdbFile, MdbHeader, MdbSecurityLimits, PacketKey, PacketType, RigidMeshPacket, RigidVertex,
    SkinMeshPacket, SkinVertex,
};

pub struct MdbParser;

impl MdbParser {
    pub fn parse(data: &[u8]) -> MdbResult<MdbFile> {
        Self::parse_with_limits(data, &MdbSecurityLimits::default())
    }

    pub fn parse_with_limits(data: &[u8], limits: &MdbSecurityLimits) -> MdbResult<MdbFile> {
        if data.len() > limits.max_file_size {
            return Err(MdbError::SecurityViolation {
                message: format!(
                    "File size {} exceeds maximum {}",
                    data.len(),
                    limits.max_file_size
                ),
            });
        }

        let mut cursor = Cursor::new(data);
        let header = Self::read_header(&mut cursor)?;

        if header.packet_count > limits.max_packet_count {
            return Err(MdbError::SecurityViolation {
                message: format!(
                    "Packet count {} exceeds maximum {}",
                    header.packet_count, limits.max_packet_count
                ),
            });
        }

        let keys = Self::read_packet_keys(&mut cursor, header.packet_count, data.len())?;

        let mut rigid_meshes = Vec::new();
        let mut skin_meshes = Vec::new();
        let mut hooks = Vec::new();
        let mut hair = Vec::new();
        let mut helm = Vec::new();

        for key in &keys {
            cursor.seek(SeekFrom::Start(u64::from(key.offset)))?;

            let mut type_bytes = [0u8; 4];
            cursor.read_exact(&mut type_bytes)?;
            let _body_size = cursor.read_u32::<LittleEndian>()?;

            match key.packet_type {
                PacketType::Rigid => {
                    let mesh = Self::read_rigid_mesh(&mut cursor, limits)?;
                    debug!(
                        "Parsed RIGD mesh '{}': {} verts, {} faces",
                        mesh.name,
                        mesh.vertices.len(),
                        mesh.faces.len()
                    );
                    rigid_meshes.push(mesh);
                }
                PacketType::Skin => {
                    let mesh = Self::read_skin_mesh(&mut cursor, limits)?;
                    debug!(
                        "Parsed SKIN mesh '{}': {} verts, {} faces",
                        mesh.name,
                        mesh.vertices.len(),
                        mesh.faces.len()
                    );
                    skin_meshes.push(mesh);
                }
                PacketType::Hook => {
                    hooks.push(Self::read_hook(&mut cursor)?);
                }
                PacketType::Hair => {
                    hair.push(Self::read_hair(&mut cursor)?);
                }
                PacketType::Helm => {
                    helm.push(Self::read_helm(&mut cursor)?);
                }
                _ => {
                    debug!("Skipping packet type {:?}", key.packet_type);
                }
            }
        }

        Ok(MdbFile {
            header,
            rigid_meshes,
            skin_meshes,
            hooks,
            hair,
            helm,
        })
    }

    fn read_header(cursor: &mut Cursor<&[u8]>) -> MdbResult<MdbHeader> {
        let mut signature = [0u8; 4];
        cursor.read_exact(&mut signature)?;

        if &signature != b"NWN2" {
            return Err(MdbError::InvalidSignature {
                found: String::from_utf8_lossy(&signature).to_string(),
            });
        }

        let major_version = cursor.read_u16::<LittleEndian>()?;
        let minor_version = cursor.read_u16::<LittleEndian>()?;
        let packet_count = cursor.read_u32::<LittleEndian>()?;

        Ok(MdbHeader {
            signature,
            major_version,
            minor_version,
            packet_count,
        })
    }

    fn read_packet_keys(
        cursor: &mut Cursor<&[u8]>,
        count: u32,
        file_size: usize,
    ) -> MdbResult<Vec<PacketKey>> {
        let mut keys = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let mut type_bytes = [0u8; 4];
            cursor.read_exact(&mut type_bytes)?;
            let offset = cursor.read_u32::<LittleEndian>()?;

            let packet_type =
                PacketType::from_bytes(&type_bytes).ok_or_else(|| MdbError::InvalidPacketType {
                    found: String::from_utf8_lossy(&type_bytes).to_string(),
                })?;

            if offset as usize >= file_size {
                return Err(MdbError::InvalidOffset { offset, file_size });
            }

            keys.push(PacketKey {
                packet_type,
                offset,
            });
        }
        Ok(keys)
    }

    fn read_fixed_string(cursor: &mut Cursor<&[u8]>, len: usize) -> MdbResult<String> {
        let mut buf = vec![0u8; len];
        cursor.read_exact(&mut buf)?;
        let end = buf.iter().position(|&b| b == 0).unwrap_or(len);
        Ok(String::from_utf8_lossy(&buf[..end]).to_string())
    }

    fn read_vec3(cursor: &mut Cursor<&[u8]>) -> MdbResult<[f32; 3]> {
        Ok([
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
            cursor.read_f32::<LittleEndian>()?,
        ])
    }

    fn read_material(cursor: &mut Cursor<&[u8]>) -> MdbResult<Material> {
        let diffuse_map_name = Self::read_fixed_string(cursor, 32)?;
        let normal_map_name = Self::read_fixed_string(cursor, 32)?;
        let tint_map_name = Self::read_fixed_string(cursor, 32)?;
        let glow_map_name = Self::read_fixed_string(cursor, 32)?;
        let diffuse_color = Self::read_vec3(cursor)?;
        let specular_color = Self::read_vec3(cursor)?;
        let specular_level = cursor.read_f32::<LittleEndian>()?;
        let specular_power = cursor.read_f32::<LittleEndian>()?;
        let flags = cursor.read_u32::<LittleEndian>()?;

        Ok(Material {
            diffuse_map_name,
            normal_map_name,
            tint_map_name,
            glow_map_name,
            diffuse_color,
            specular_color,
            specular_level,
            specular_power,
            flags,
        })
    }

    fn read_orientation(cursor: &mut Cursor<&[u8]>) -> MdbResult<[[f32; 3]; 3]> {
        let mut m = [[0.0f32; 3]; 3];
        for row in &mut m {
            for val in row.iter_mut() {
                *val = cursor.read_f32::<LittleEndian>()?;
            }
        }
        Ok(m)
    }

    fn read_rigid_mesh(
        cursor: &mut Cursor<&[u8]>,
        limits: &MdbSecurityLimits,
    ) -> MdbResult<RigidMeshPacket> {
        let name = Self::read_fixed_string(cursor, 32)?;
        let material = Self::read_material(cursor)?;
        let vertex_count = cursor.read_u32::<LittleEndian>()?;
        let face_count = cursor.read_u32::<LittleEndian>()?;

        if vertex_count > limits.max_vertex_count {
            return Err(MdbError::SecurityViolation {
                message: format!(
                    "Vertex count {vertex_count} exceeds maximum {}",
                    limits.max_vertex_count
                ),
            });
        }
        if face_count > limits.max_face_count {
            return Err(MdbError::SecurityViolation {
                message: format!(
                    "Face count {face_count} exceeds maximum {}",
                    limits.max_face_count
                ),
            });
        }

        let mut vertices = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            vertices.push(RigidVertex {
                position: Self::read_vec3(cursor)?,
                normal: Self::read_vec3(cursor)?,
                tangent: Self::read_vec3(cursor)?,
                binormal: Self::read_vec3(cursor)?,
                uvw: Self::read_vec3(cursor)?,
            });
        }

        let mut faces = Vec::with_capacity(face_count as usize);
        for _ in 0..face_count {
            faces.push(Face {
                indices: [
                    cursor.read_u16::<LittleEndian>()?,
                    cursor.read_u16::<LittleEndian>()?,
                    cursor.read_u16::<LittleEndian>()?,
                ],
            });
        }

        Ok(RigidMeshPacket {
            name,
            material,
            vertices,
            faces,
        })
    }

    fn read_skin_mesh(
        cursor: &mut Cursor<&[u8]>,
        limits: &MdbSecurityLimits,
    ) -> MdbResult<SkinMeshPacket> {
        let name = Self::read_fixed_string(cursor, 32)?;
        let skeleton_name = Self::read_fixed_string(cursor, 32)?;
        let material = Self::read_material(cursor)?;
        let vertex_count = cursor.read_u32::<LittleEndian>()?;
        let face_count = cursor.read_u32::<LittleEndian>()?;

        if vertex_count > limits.max_vertex_count {
            return Err(MdbError::SecurityViolation {
                message: format!(
                    "Vertex count {vertex_count} exceeds maximum {}",
                    limits.max_vertex_count
                ),
            });
        }
        if face_count > limits.max_face_count {
            return Err(MdbError::SecurityViolation {
                message: format!(
                    "Face count {face_count} exceeds maximum {}",
                    limits.max_face_count
                ),
            });
        }

        let mut vertices = Vec::with_capacity(vertex_count as usize);
        for _ in 0..vertex_count {
            let position = Self::read_vec3(cursor)?;
            let normal = Self::read_vec3(cursor)?;

            let bone_weights = [
                cursor.read_f32::<LittleEndian>()?,
                cursor.read_f32::<LittleEndian>()?,
                cursor.read_f32::<LittleEndian>()?,
                cursor.read_f32::<LittleEndian>()?,
            ];

            let mut bone_indices = [0u8; 4];
            cursor.read_exact(&mut bone_indices)?;

            let tangent = Self::read_vec3(cursor)?;
            let binormal = Self::read_vec3(cursor)?;
            let uvw = Self::read_vec3(cursor)?;
            let bone_count = cursor.read_f32::<LittleEndian>()?;

            vertices.push(SkinVertex {
                position,
                normal,
                bone_weights,
                bone_indices,
                tangent,
                binormal,
                uvw,
                bone_count,
            });
        }

        let mut faces = Vec::with_capacity(face_count as usize);
        for _ in 0..face_count {
            faces.push(Face {
                indices: [
                    cursor.read_u16::<LittleEndian>()?,
                    cursor.read_u16::<LittleEndian>()?,
                    cursor.read_u16::<LittleEndian>()?,
                ],
            });
        }

        Ok(SkinMeshPacket {
            name,
            skeleton_name,
            material,
            vertices,
            faces,
        })
    }

    fn read_hook(cursor: &mut Cursor<&[u8]>) -> MdbResult<HookPacket> {
        let name = Self::read_fixed_string(cursor, 32)?;
        let point_type = cursor.read_u16::<LittleEndian>()?;
        let point_size = cursor.read_u16::<LittleEndian>()?;
        let position = Self::read_vec3(cursor)?;
        let orientation = Self::read_orientation(cursor)?;

        Ok(HookPacket {
            name,
            point_type,
            point_size,
            position,
            orientation,
        })
    }

    fn read_hair(cursor: &mut Cursor<&[u8]>) -> MdbResult<HairPacket> {
        let name = Self::read_fixed_string(cursor, 32)?;
        let behavior_val = cursor.read_u32::<LittleEndian>()?;
        let shortening_behavior = match behavior_val {
            1 => HairShorteningBehavior::Short,
            2 => HairShorteningBehavior::Ponytail,
            _ => HairShorteningBehavior::Low,
        };
        let position = Self::read_vec3(cursor)?;
        let orientation = Self::read_orientation(cursor)?;

        Ok(HairPacket {
            name,
            shortening_behavior,
            position,
            orientation,
        })
    }

    fn read_helm(cursor: &mut Cursor<&[u8]>) -> MdbResult<HelmPacket> {
        let name = Self::read_fixed_string(cursor, 32)?;
        let behavior_val = cursor.read_u32::<LittleEndian>()?;
        let hiding_behavior = match behavior_val {
            1 => HelmHidingBehavior::HairHidden,
            2 => HelmHidingBehavior::PartialHair,
            3 => HelmHidingBehavior::HeadHidden,
            _ => HelmHidingBehavior::NoneHidden,
        };
        let position = Self::read_vec3(cursor)?;
        let orientation = Self::read_orientation(cursor)?;

        Ok(HelmPacket {
            name,
            hiding_behavior,
            position,
            orientation,
        })
    }
}
