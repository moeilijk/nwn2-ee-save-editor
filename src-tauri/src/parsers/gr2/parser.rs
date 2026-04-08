use std::io::{Cursor, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use tracing::debug;

use super::decompress::gr2_decompress;
use super::error::{Gr2Error, Gr2Result};
use super::types::{
    BoneTransform, GR2_MAGIC, GR2_VERSION, Gr2Bone, Gr2Header, Gr2Info, Gr2SecurityLimits,
    Gr2Skeleton, Relocation, SectionHeader, transform_flags,
};

pub struct Gr2Parser;

impl Gr2Parser {
    pub fn parse(data: &[u8]) -> Gr2Result<Gr2Skeleton> {
        Self::parse_with_limits(data, &Gr2SecurityLimits::default())
    }

    pub fn parse_with_limits(data: &[u8], limits: &Gr2SecurityLimits) -> Gr2Result<Gr2Skeleton> {
        if data.len() > limits.max_file_size {
            return Err(Gr2Error::SecurityViolation {
                message: format!(
                    "File size {} exceeds max {}",
                    data.len(),
                    limits.max_file_size
                ),
            });
        }

        let mut cursor = Cursor::new(data);

        let header = Self::read_header(&mut cursor)?;
        let section_headers = Self::read_section_headers(&mut cursor, header.info.sections_count)?;
        let mut sections_data = Self::load_sections(data, &section_headers, limits)?;
        let section_offsets = Self::compute_section_offsets(&section_headers);
        Self::apply_relocations(data, &section_headers, &section_offsets, &mut sections_data)?;

        Self::extract_skeleton(&sections_data, &section_offsets, &header.info, limits)
    }

    fn read_header(cursor: &mut Cursor<&[u8]>) -> Gr2Result<Gr2Header> {
        let magic = [
            cursor.read_u32::<LittleEndian>()?,
            cursor.read_u32::<LittleEndian>()?,
            cursor.read_u32::<LittleEndian>()?,
            cursor.read_u32::<LittleEndian>()?,
        ];

        if magic != GR2_MAGIC {
            return Err(Gr2Error::InvalidMagic);
        }

        let size = cursor.read_u32::<LittleEndian>()?;
        let format = cursor.read_u32::<LittleEndian>()?;
        let reserved = [
            cursor.read_u32::<LittleEndian>()?,
            cursor.read_u32::<LittleEndian>()?,
        ];

        let info = Gr2Info {
            version: cursor.read_u32::<LittleEndian>()?,
            file_size: cursor.read_u32::<LittleEndian>()?,
            crc32: cursor.read_u32::<LittleEndian>()?,
            sections_offset: cursor.read_u32::<LittleEndian>()?,
            sections_count: cursor.read_u32::<LittleEndian>()?,
            type_section: cursor.read_u32::<LittleEndian>()?,
            type_offset: cursor.read_u32::<LittleEndian>()?,
            root_section: cursor.read_u32::<LittleEndian>()?,
            root_offset: cursor.read_u32::<LittleEndian>()?,
            tag: cursor.read_u32::<LittleEndian>()?,
            extra: [
                cursor.read_u32::<LittleEndian>()?,
                cursor.read_u32::<LittleEndian>()?,
                cursor.read_u32::<LittleEndian>()?,
                cursor.read_u32::<LittleEndian>()?,
            ],
        };

        if info.version != GR2_VERSION {
            return Err(Gr2Error::InvalidVersion {
                found: info.version,
            });
        }

        Ok(Gr2Header {
            magic,
            size,
            format,
            reserved,
            info,
        })
    }

    fn read_section_headers(
        cursor: &mut Cursor<&[u8]>,
        count: u32,
    ) -> Gr2Result<Vec<SectionHeader>> {
        let mut headers = Vec::with_capacity(count as usize);
        for _ in 0..count {
            headers.push(SectionHeader {
                compression: cursor.read_u32::<LittleEndian>()?,
                data_offset: cursor.read_u32::<LittleEndian>()?,
                data_size: cursor.read_u32::<LittleEndian>()?,
                decompressed_size: cursor.read_u32::<LittleEndian>()?,
                alignment: cursor.read_u32::<LittleEndian>()?,
                first16bit: cursor.read_u32::<LittleEndian>()?,
                first8bit: cursor.read_u32::<LittleEndian>()?,
                relocations_offset: cursor.read_u32::<LittleEndian>()?,
                relocations_count: cursor.read_u32::<LittleEndian>()?,
                marshallings_offset: cursor.read_u32::<LittleEndian>()?,
                marshallings_count: cursor.read_u32::<LittleEndian>()?,
            });
        }
        Ok(headers)
    }

    fn compute_section_offsets(headers: &[SectionHeader]) -> Vec<usize> {
        let mut offsets = Vec::with_capacity(headers.len());
        let mut offset = 0usize;
        for h in headers {
            offsets.push(offset);
            offset += h.decompressed_size as usize;
        }
        offsets
    }

    fn load_sections(
        data: &[u8],
        headers: &[SectionHeader],
        limits: &Gr2SecurityLimits,
    ) -> Gr2Result<Vec<u8>> {
        let total_size: usize = headers.iter().map(|h| h.decompressed_size as usize).sum();

        if total_size > limits.max_decompressed_size {
            return Err(Gr2Error::SecurityViolation {
                message: format!(
                    "Total decompressed size {} exceeds max {}",
                    total_size, limits.max_decompressed_size
                ),
            });
        }

        let mut output = vec![0u8; total_size];
        let mut out_offset = 0usize;

        for (i, header) in headers.iter().enumerate() {
            if header.decompressed_size == 0 {
                continue;
            }

            let start = header.data_offset as usize;
            let end = std::cmp::min(start + header.data_size as usize + 4, data.len());

            if start >= data.len() {
                return Err(Gr2Error::InvalidOffset {
                    offset: start,
                    size: data.len(),
                });
            }

            let compressed = &data[start..end];
            let dest = &mut output[out_offset..out_offset + header.decompressed_size as usize];

            match header.compression {
                0 => {
                    let copy_len = std::cmp::min(compressed.len(), dest.len());
                    dest[..copy_len].copy_from_slice(&compressed[..copy_len]);
                }
                2 => {
                    let decompressed = gr2_decompress(
                        compressed,
                        header.first16bit,
                        header.first8bit,
                        header.decompressed_size,
                    )?;
                    dest.copy_from_slice(&decompressed);
                }
                other => {
                    return Err(Gr2Error::UnsupportedCompression(other));
                }
            }

            debug!(
                "Section {}: {} bytes (compression={})",
                i, header.decompressed_size, header.compression
            );
            out_offset += header.decompressed_size as usize;
        }

        Ok(output)
    }

    fn apply_relocations(
        file_data: &[u8],
        headers: &[SectionHeader],
        section_offsets: &[usize],
        sections_data: &mut [u8],
    ) -> Gr2Result<()> {
        for (i, header) in headers.iter().enumerate() {
            if header.relocations_count == 0 {
                continue;
            }

            let mut cursor = Cursor::new(file_data);
            cursor.seek(SeekFrom::Start(u64::from(header.relocations_offset)))?;

            for _ in 0..header.relocations_count {
                let rel = Relocation {
                    offset: cursor.read_u32::<LittleEndian>()?,
                    target_section: cursor.read_u32::<LittleEndian>()?,
                    target_offset: cursor.read_u32::<LittleEndian>()?,
                };

                if rel.target_section as usize >= section_offsets.len() {
                    return Err(Gr2Error::InvalidSectionIndex {
                        index: rel.target_section,
                        max: section_offsets.len() as u32 - 1,
                    });
                }

                let source_pos = section_offsets[i] + rel.offset as usize;
                let target_pos =
                    section_offsets[rel.target_section as usize] + rel.target_offset as usize;

                if source_pos + 4 <= sections_data.len() {
                    let bytes = (target_pos as u32).to_le_bytes();
                    sections_data[source_pos..source_pos + 4].copy_from_slice(&bytes);
                }
            }
        }

        Ok(())
    }

    fn read_u32_at(data: &[u8], offset: usize) -> u32 {
        if offset + 4 > data.len() {
            return 0;
        }
        u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ])
    }

    fn read_i32_at(data: &[u8], offset: usize) -> i32 {
        Self::read_u32_at(data, offset) as i32
    }

    fn read_f32_at(data: &[u8], offset: usize) -> f32 {
        if offset + 4 > data.len() {
            return 0.0;
        }
        f32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ])
    }

    fn read_string_at(data: &[u8], offset: usize) -> String {
        if offset >= data.len() {
            return String::new();
        }
        let end = data[offset..]
            .iter()
            .position(|&b| b == 0)
            .map(|p| offset + p)
            .unwrap_or(data.len());
        String::from_utf8_lossy(&data[offset..end]).to_string()
    }

    fn extract_skeleton(
        sections_data: &[u8],
        section_offsets: &[usize],
        info: &Gr2Info,
        limits: &Gr2SecurityLimits,
    ) -> Gr2Result<Gr2Skeleton> {
        let root_base = section_offsets.get(info.root_section as usize).ok_or(
            Gr2Error::InvalidSectionIndex {
                index: info.root_section,
                max: section_offsets.len() as u32 - 1,
            },
        )?;
        let file_info_offset = root_base + info.root_offset as usize;

        let skeletons_count = Self::read_i32_at(sections_data, file_info_offset + 28);
        let skeletons_array_offset =
            Self::read_u32_at(sections_data, file_info_offset + 32) as usize;

        if skeletons_count <= 0 {
            return Err(Gr2Error::NoSkeleton);
        }

        debug!("Found {} skeleton(s)", skeletons_count);

        let skeleton_offset = Self::read_u32_at(sections_data, skeletons_array_offset) as usize;

        let name_offset = Self::read_u32_at(sections_data, skeleton_offset) as usize;
        let bones_count = Self::read_i32_at(sections_data, skeleton_offset + 4);
        let bones_offset = Self::read_u32_at(sections_data, skeleton_offset + 8) as usize;

        let skeleton_name = Self::read_string_at(sections_data, name_offset);

        if bones_count as u32 > limits.max_bones {
            return Err(Gr2Error::SecurityViolation {
                message: format!(
                    "Bone count {} exceeds max {}",
                    bones_count, limits.max_bones
                ),
            });
        }

        debug!("Skeleton '{}': {} bones", skeleton_name, bones_count);

        let mut bones = Vec::with_capacity(bones_count as usize);

        const BONE_SIZE: usize = 156;

        for i in 0..bones_count as usize {
            let bone_base = bones_offset + i * BONE_SIZE;

            let bone_name_offset = Self::read_u32_at(sections_data, bone_base) as usize;
            let bone_name = Self::read_string_at(sections_data, bone_name_offset);
            let parent_index = Self::read_i32_at(sections_data, bone_base + 4);

            let flags = Self::read_u32_at(sections_data, bone_base + 8);

            let position = if flags & transform_flags::HAS_POSITION != 0 {
                [
                    Self::read_f32_at(sections_data, bone_base + 12),
                    Self::read_f32_at(sections_data, bone_base + 16),
                    Self::read_f32_at(sections_data, bone_base + 20),
                ]
            } else {
                [0.0, 0.0, 0.0]
            };

            let rotation = if flags & transform_flags::HAS_ROTATION != 0 {
                [
                    Self::read_f32_at(sections_data, bone_base + 24),
                    Self::read_f32_at(sections_data, bone_base + 28),
                    Self::read_f32_at(sections_data, bone_base + 32),
                    Self::read_f32_at(sections_data, bone_base + 36),
                ]
            } else {
                [0.0, 0.0, 0.0, 1.0]
            };

            let scale = if flags & transform_flags::HAS_SCALE_SHEAR != 0 {
                [
                    Self::read_f32_at(sections_data, bone_base + 40),
                    Self::read_f32_at(sections_data, bone_base + 52),
                    Self::read_f32_at(sections_data, bone_base + 64),
                ]
            } else {
                [1.0, 1.0, 1.0]
            };

            bones.push(Gr2Bone {
                name: bone_name,
                parent_index,
                transform: BoneTransform {
                    position,
                    rotation,
                    scale,
                },
            });
        }

        Ok(Gr2Skeleton {
            name: skeleton_name,
            bones,
        })
    }
}
