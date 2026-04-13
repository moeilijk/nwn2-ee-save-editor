use std::io::{Cursor, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use tracing::debug;

use super::decompress::gr2_decompress;
use super::error::{Gr2Error, Gr2Result};
use super::types::{
    BoneTransform, GR2_MAGIC, GR2_VERSION, Gr2Animation, Gr2Bone, Gr2Header, Gr2Info,
    Gr2SecurityLimits, Gr2Skeleton, Gr2Track, Relocation, SectionHeader, transform_flags,
};

pub struct Gr2Parser;

impl Gr2Parser {
    pub fn parse(data: &[u8]) -> Gr2Result<Gr2Skeleton> {
        Self::parse_with_limits(data, &Gr2SecurityLimits::default())
    }

    pub fn parse_animations(data: &[u8]) -> Gr2Result<Vec<Gr2Animation>> {
        Self::parse_animations_with_limits(data, &Gr2SecurityLimits::default())
    }

    fn parse_animations_with_limits(
        data: &[u8],
        limits: &Gr2SecurityLimits,
    ) -> Gr2Result<Vec<Gr2Animation>> {
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

        Self::extract_animations(&sections_data, &section_offsets, &header.info)
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

            // scale_shear is a 3×3 column-major matrix at offset 40 (9 floats, 36 bytes).
            // Extract diagonal elements [0],[4],[8] for scale (matches nwn2mdk).
            let scale = if flags & transform_flags::HAS_SCALE_SHEAR != 0 {
                [
                    Self::read_f32_at(sections_data, bone_base + 40),
                    Self::read_f32_at(sections_data, bone_base + 56),
                    Self::read_f32_at(sections_data, bone_base + 72),
                ]
            } else {
                [1.0, 1.0, 1.0]
            };

            // InverseWorld4x4 at offset 76 (16 × f32 = 64 bytes, row-major)
            let inverse_world_4x4: [f32; 16] =
                std::array::from_fn(|k| Self::read_f32_at(sections_data, bone_base + 76 + k * 4));

            bones.push(Gr2Bone {
                name: bone_name,
                parent_index,
                transform: BoneTransform {
                    position,
                    rotation,
                    scale,
                },
                inverse_world_4x4,
            });
        }

        Ok(Gr2Skeleton {
            name: skeleton_name,
            bones,
        })
    }

    pub fn audit_curve_formats(data: &[u8]) -> Vec<(String, u8, u8, u8)> {
        let limits = Gr2SecurityLimits::default();
        let mut cursor = Cursor::new(data);
        let header = match Self::read_header(&mut cursor) {
            Ok(h) => h,
            Err(_) => return Vec::new(),
        };
        let section_headers =
            match Self::read_section_headers(&mut cursor, header.info.sections_count) {
                Ok(h) => h,
                Err(_) => return Vec::new(),
            };
        let mut sections_data = match Self::load_sections(data, &section_headers, &limits) {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };
        let section_offsets = Self::compute_section_offsets(&section_headers);
        if Self::apply_relocations(data, &section_headers, &section_offsets, &mut sections_data)
            .is_err()
        {
            return Vec::new();
        }

        let root_base = section_offsets[header.info.root_section as usize];
        let fi_off = root_base + header.info.root_offset as usize;
        let anim_count = Self::read_i32_at(&sections_data, fi_off + 0x4C);
        let anim_array = Self::read_u32_at(&sections_data, fi_off + 0x50) as usize;
        if anim_count <= 0 {
            return Vec::new();
        }

        let anim_ptr = Self::read_u32_at(&sections_data, anim_array) as usize;
        let tg_count = Self::read_i32_at(&sections_data, anim_ptr + 0x10);
        let tg_ptr = Self::read_u32_at(&sections_data, anim_ptr + 0x14) as usize;
        if tg_count <= 0 {
            return Vec::new();
        }

        let mut results = Vec::new();
        for tg in 0..tg_count as usize {
            let tg_off = Self::read_u32_at(&sections_data, tg_ptr + tg * 4) as usize;
            let tt_count = Self::read_i32_at(&sections_data, tg_off + 0x0C);
            let tt_ptr = Self::read_u32_at(&sections_data, tg_off + 0x10) as usize;

            const STRIDE: usize = 28;
            for i in 0..tt_count as usize {
                let off = tt_ptr + i * STRIDE;
                let name_ptr = Self::read_u32_at(&sections_data, off) as usize;
                let name = Self::read_string_at(&sections_data, name_ptr);
                let pos_ptr = Self::read_u32_at(&sections_data, off + 0x08) as usize;
                let orient_ptr = Self::read_u32_at(&sections_data, off + 0x10) as usize;
                let scale_ptr = Self::read_u32_at(&sections_data, off + 0x18) as usize;

                let pos_fmt = Self::curve3_format(&sections_data, pos_ptr);
                let orient_fmt = Self::curve4_format(&sections_data, orient_ptr);
                let scale_fmt = Self::curve3_format(&sections_data, scale_ptr);
                results.push((name, pos_fmt, orient_fmt, scale_fmt));
            }
        }
        results
    }

    /// Dump raw curve data diagnostics for specific bones.
    /// Returns formatted text matching dumpgr2 output for comparison.
    pub fn dump_curve_diagnostics(data: &[u8], bone_names: &[&str]) -> String {
        use std::fmt::Write;
        let limits = Gr2SecurityLimits::default();
        let mut cursor = Cursor::new(data);
        let header = match Self::read_header(&mut cursor) {
            Ok(h) => h,
            Err(e) => return format!("Header error: {e}"),
        };
        let section_headers =
            match Self::read_section_headers(&mut cursor, header.info.sections_count) {
                Ok(h) => h,
                Err(e) => return format!("Section header error: {e}"),
            };
        let mut sd = match Self::load_sections(data, &section_headers, &limits) {
            Ok(d) => d,
            Err(e) => return format!("Section load error: {e}"),
        };
        let so = Self::compute_section_offsets(&section_headers);
        if let Err(e) = Self::apply_relocations(data, &section_headers, &so, &mut sd) {
            return format!("Relocation error: {e}");
        }

        let root_base = so[header.info.root_section as usize];
        let fi_off = root_base + header.info.root_offset as usize;
        let anim_count = Self::read_i32_at(&sd, fi_off + 0x4C);
        let anim_array = Self::read_u32_at(&sd, fi_off + 0x50) as usize;
        if anim_count <= 0 {
            return "No animations found".into();
        }

        let anim_ptr = Self::read_u32_at(&sd, anim_array) as usize;
        let duration = Self::read_f32_at(&sd, anim_ptr + 4);
        let tg_count = Self::read_i32_at(&sd, anim_ptr + 0x10);
        let tg_ptr = Self::read_u32_at(&sd, anim_ptr + 0x14) as usize;

        let mut out = String::new();
        let _ = writeln!(out, "Animation duration: {duration:.6}");

        #[allow(clippy::approx_constant)]
        const SCALE_TABLE: [f32; 16] = [
            1.4142135,
            0.70710677,
            0.35355338,
            0.35355338,
            0.35355338,
            0.17677669,
            0.17677669,
            0.17677669,
            -1.4142135,
            -0.70710677,
            -0.35355338,
            -0.35355338,
            -0.35355338,
            -0.17677669,
            -0.17677669,
            -0.17677669,
        ];
        const OFFSET_TABLE: [f32; 16] = [
            -0.70710677,
            -0.35355338,
            -0.53033006,
            -0.17677669,
            0.17677669,
            -0.17677669,
            -0.088388346,
            0.0,
            0.70710677,
            0.35355338,
            0.53033006,
            0.17677669,
            -0.17677669,
            0.17677669,
            0.088388346,
            -0.0,
        ];

        for tg in 0..tg_count as usize {
            let tg_off = Self::read_u32_at(&sd, tg_ptr + tg * 4) as usize;
            let tt_count = Self::read_i32_at(&sd, tg_off + 0x0C);
            let tt_ptr = Self::read_u32_at(&sd, tg_off + 0x10) as usize;

            for i in 0..tt_count as usize {
                let off = tt_ptr + i * 28;
                let name_ptr = Self::read_u32_at(&sd, off) as usize;
                let name = Self::read_string_at(&sd, name_ptr);

                if !bone_names.contains(&name.as_str()) {
                    continue;
                }

                let orient_ptr = Self::read_u32_at(&sd, off + 0x10) as usize;
                if orient_ptr == 0 || orient_ptr + 16 > sd.len() {
                    let _ = writeln!(out, "\n- Name: {name}\n  OrientationCurve: MISSING");
                    continue;
                }

                let fmt = sd[orient_ptr];
                let degree = sd[orient_ptr + 1];
                let _ = writeln!(out, "\n- Name: {name}");
                let _ = writeln!(out, "  OrientationCurve:");
                let _ = writeln!(
                    out,
                    "    Format: {fmt} ({})",
                    match fmt {
                        0 => "DaKeyframes32f",
                        1 => "DaK32fC32f",
                        2 => "DaIdentity",
                        3 => "DaConstant32f",
                        4 => "D3Constant32f",
                        5 => "D4Constant32f",
                        6 => "DaK16uC16u",
                        7 => "DaK8uC8u",
                        8 => "D4nK16uC15u",
                        9 => "D4nK8uC7u",
                        10 => "D3K16uC16u",
                        11 => "D3K8uC8u",
                        _ => "UNKNOWN",
                    }
                );
                let _ = writeln!(out, "    Degree: {degree}");

                // Raw header hex dump
                let header_end = (orient_ptr + 16).min(sd.len());
                let _ = write!(out, "    RawHeader:");
                for b in &sd[orient_ptr..header_end] {
                    let _ = write!(out, " {:02x}", b);
                }
                let _ = writeln!(out);

                if fmt == 8 || fmt == 9 {
                    let scale_offset_entries =
                        u16::from_le_bytes([sd[orient_ptr + 2], sd[orient_ptr + 3]]);
                    let one_over_knot_scale = Self::read_f32_at(&sd, orient_ptr + 4);
                    let kc_count = Self::read_i32_at(&sd, orient_ptr + 8);
                    let kc_ptr = Self::read_u32_at(&sd, orient_ptr + 12) as usize;

                    let _ = writeln!(out, "    ScaleOffsetTableEntries: {scale_offset_entries}");

                    let selectors: [usize; 4] = [
                        (scale_offset_entries & 0x0F) as usize,
                        ((scale_offset_entries >> 4) & 0x0F) as usize,
                        ((scale_offset_entries >> 8) & 0x0F) as usize,
                        ((scale_offset_entries >> 12) & 0x0F) as usize,
                    ];
                    let _ = writeln!(
                        out,
                        "    Selectors: [{}, {}, {}, {}]",
                        selectors[0], selectors[1], selectors[2], selectors[3]
                    );

                    let inv = if fmt == 9 {
                        1.0f32 / 127.0
                    } else {
                        1.0f32 / 32767.0
                    };
                    let scales: [f32; 4] = std::array::from_fn(|i| SCALE_TABLE[selectors[i]] * inv);
                    let offsets: [f32; 4] = std::array::from_fn(|i| OFFSET_TABLE[selectors[i]]);
                    let _ = writeln!(
                        out,
                        "    Scales: [{}, {}, {}, {}]",
                        scales[0], scales[1], scales[2], scales[3]
                    );
                    let _ = writeln!(
                        out,
                        "    Offsets: [{}, {}, {}, {}]",
                        offsets[0], offsets[1], offsets[2], offsets[3]
                    );
                    let _ = writeln!(out, "    OneOverKnotScale: {one_over_knot_scale}");
                    let _ = writeln!(out, "    KnotsControls_count: {kc_count}");
                    let _ = writeln!(out, "    kc_ptr offset: {kc_ptr}");

                    if kc_count > 0 && kc_ptr > 0 && kc_ptr < sd.len() {
                        let total = kc_count as usize;
                        let knots_count = total / 4;

                        // Dump knots
                        let _ = writeln!(out, "    Knots (first 5):");
                        let show_knots = knots_count.min(5);
                        if fmt == 9 {
                            for ki in 0..show_knots {
                                let raw = sd[kc_ptr + ki];
                                let time = f32::from(raw) / one_over_knot_scale;
                                let _ = writeln!(out, "      [{ki}] raw={raw} time={time}");
                            }
                        } else {
                            for ki in 0..show_knots {
                                let o = kc_ptr + ki * 2;
                                let raw = u16::from_le_bytes([sd[o], sd[o + 1]]);
                                let time = f32::from(raw) / one_over_knot_scale;
                                let _ = writeln!(out, "      [{ki}] raw={raw} time={time}");
                            }
                        }

                        // Dump controls
                        let _ = writeln!(out, "    Controls (first 5):");
                        let controls_start = if fmt == 9 {
                            kc_ptr + knots_count
                        } else {
                            kc_ptr + knots_count * 2
                        };
                        let num_controls = (total - knots_count) / 3;
                        let show = num_controls.min(5);
                        for ci in 0..show {
                            if fmt == 9 {
                                let base = controls_start + ci * 3;
                                if base + 3 > sd.len() {
                                    break;
                                }
                                let a = sd[base];
                                let b = sd[base + 1];
                                let c = sd[base + 2];
                                let _ = writeln!(out, "      [{ci}] raw=({a}, {b}, {c})");

                                // Decode
                                let swizzle1 = (((b & 0x80) >> 6) | ((c & 0x80) >> 7)) as usize;
                                let sw2 = (swizzle1 + 1) & 3;
                                let sw3 = (sw2 + 1) & 3;
                                let sw4 = (sw3 + 1) & 3;
                                let da = f32::from(a & 0x7F) * scales[sw2] + offsets[sw2];
                                let db = f32::from(b & 0x7F) * scales[sw3] + offsets[sw3];
                                let dc = f32::from(c & 0x7F) * scales[sw4] + offsets[sw4];
                                let sum_sq = da * da + db * db + dc * dc;
                                let mut dd = (1.0 - sum_sq).max(0.0).sqrt();
                                if (a & 0x80) != 0 {
                                    dd = -dd;
                                }
                                let mut q = [0.0f32; 4];
                                q[swizzle1] = dd;
                                q[sw2] = da;
                                q[sw3] = db;
                                q[sw4] = dc;
                                let _ = writeln!(
                                    out,
                                    "            swizzle={swizzle1} decoded=[{:.6}, {:.6}, {:.6}, {:.6}]",
                                    q[0], q[1], q[2], q[3]
                                );
                            } else {
                                let base = controls_start + ci * 6;
                                if base + 6 > sd.len() {
                                    break;
                                }
                                let a = u16::from_le_bytes([sd[base], sd[base + 1]]);
                                let b = u16::from_le_bytes([sd[base + 2], sd[base + 3]]);
                                let c = u16::from_le_bytes([sd[base + 4], sd[base + 5]]);
                                let _ = writeln!(out, "      [{ci}] raw=({a}, {b}, {c})");

                                let swizzle1 =
                                    (((b & 0x8000) >> 14) | ((c & 0x8000) >> 15)) as usize;
                                let sw2 = (swizzle1 + 1) & 3;
                                let sw3 = (sw2 + 1) & 3;
                                let sw4 = (sw3 + 1) & 3;
                                let da = f32::from(a & 0x7FFF) * scales[sw2] + offsets[sw2];
                                let db = f32::from(b & 0x7FFF) * scales[sw3] + offsets[sw3];
                                let dc = f32::from(c & 0x7FFF) * scales[sw4] + offsets[sw4];
                                let sum_sq = da * da + db * db + dc * dc;
                                let mut dd = (1.0 - sum_sq).max(0.0).sqrt();
                                if (a & 0x8000) != 0 {
                                    dd = -dd;
                                }
                                let mut q = [0.0f32; 4];
                                q[swizzle1] = dd;
                                q[sw2] = da;
                                q[sw3] = db;
                                q[sw4] = dc;
                                let _ = writeln!(
                                    out,
                                    "            swizzle={swizzle1} decoded=[{:.6}, {:.6}, {:.6}, {:.6}]",
                                    q[0], q[1], q[2], q[3]
                                );
                            }
                        }
                    }
                }
            }
        }
        out
    }

    fn extract_animations(
        sections_data: &[u8],
        section_offsets: &[usize],
        info: &Gr2Info,
    ) -> Gr2Result<Vec<Gr2Animation>> {
        let root_base = section_offsets.get(info.root_section as usize).ok_or(
            Gr2Error::InvalidSectionIndex {
                index: info.root_section,
                max: section_offsets.len() as u32 - 1,
            },
        )?;
        let file_info_offset = root_base + info.root_offset as usize;

        let anim_count = Self::read_i32_at(sections_data, file_info_offset + 0x4C);
        let anim_array_offset = Self::read_u32_at(sections_data, file_info_offset + 0x50) as usize;

        if anim_count <= 0 {
            return Err(Gr2Error::NoAnimations);
        }

        debug!("Found {} animation(s)", anim_count);

        let mut animations = Vec::new();
        for i in 0..anim_count as usize {
            let anim_ptr = Self::read_u32_at(sections_data, anim_array_offset + i * 4) as usize;
            if anim_ptr == 0 {
                continue;
            }

            let name_ptr = Self::read_u32_at(sections_data, anim_ptr) as usize;
            let name = Self::read_string_at(sections_data, name_ptr);
            let duration = Self::read_f32_at(sections_data, anim_ptr + 4);
            let time_step = Self::read_f32_at(sections_data, anim_ptr + 8);
            let track_group_count = Self::read_i32_at(sections_data, anim_ptr + 0x10);
            let track_groups_ptr = Self::read_u32_at(sections_data, anim_ptr + 0x14) as usize;

            debug!(
                "Animation '{}': duration={:.3}s step={:.4} track_groups={}",
                name, duration, time_step, track_group_count
            );

            let mut tracks = Vec::new();
            for tg in 0..track_group_count as usize {
                let tg_ptr = Self::read_u32_at(sections_data, track_groups_ptr + tg * 4) as usize;
                if tg_ptr == 0 {
                    continue;
                }
                Self::extract_track_group(sections_data, tg_ptr, &mut tracks);
            }

            animations.push(Gr2Animation {
                name,
                duration,
                time_step,
                tracks,
            });
        }

        Ok(animations)
    }

    fn extract_track_group(data: &[u8], tg_offset: usize, tracks: &mut Vec<Gr2Track>) {
        let transform_count = Self::read_i32_at(data, tg_offset + 0x0C);
        let transform_ptr = Self::read_u32_at(data, tg_offset + 0x10) as usize;

        if transform_count <= 0 || transform_ptr == 0 {
            return;
        }

        // GR2_transform_track layout (28 bytes, NO flags field):
        // +0x00: name_ptr (4)
        // +0x04: position_curve.keys_ptr (4) - type definition
        // +0x08: position_curve.curve_data_ptr (4)
        // +0x0C: orientation_curve.keys_ptr (4) - type definition
        // +0x10: orientation_curve.curve_data_ptr (4)
        // +0x14: scale_shear_curve.keys_ptr (4) - type definition
        // +0x18: scale_shear_curve.curve_data_ptr (4)
        const TRANSFORM_TRACK_SIZE: usize = 28;

        for i in 0..transform_count as usize {
            let tt_offset = transform_ptr + i * TRANSFORM_TRACK_SIZE;
            let bone_name_ptr = Self::read_u32_at(data, tt_offset) as usize;
            let bone_name = Self::read_string_at(data, bone_name_ptr);

            let pos_data_ptr = Self::read_u32_at(data, tt_offset + 0x08) as usize;
            let orient_data_ptr = Self::read_u32_at(data, tt_offset + 0x10) as usize;
            let scale_data_ptr = Self::read_u32_at(data, tt_offset + 0x18) as usize;

            let position_keys = if pos_data_ptr != 0 {
                Self::decode_curve3(data, pos_data_ptr)
            } else {
                Vec::new()
            };

            let rotation_keys = if orient_data_ptr != 0 {
                Self::decode_curve4(data, orient_data_ptr)
            } else {
                Vec::new()
            };

            let scale_keys = if scale_data_ptr != 0 {
                Self::decode_curve3(data, scale_data_ptr)
            } else {
                Vec::new()
            };

            tracks.push(Gr2Track {
                bone_name,
                position_keys,
                rotation_keys,
                scale_keys,
            });
        }
    }

    fn curve3_format(data: &[u8], data_ptr: usize) -> u8 {
        if data_ptr == 0 || data_ptr + 1 > data.len() {
            return 255;
        }
        data[data_ptr]
    }

    fn curve4_format(data: &[u8], data_ptr: usize) -> u8 {
        if data_ptr == 0 || data_ptr + 1 > data.len() {
            return 255;
        }
        data[data_ptr]
    }

    fn decode_curve3(data: &[u8], data_ptr: usize) -> Vec<(f32, [f32; 3])> {
        if data_ptr == 0 || data_ptr + 4 > data.len() {
            return Vec::new();
        }
        match data[data_ptr] {
            0 => Self::decode_da_keyframes32f::<3>(data, data_ptr),
            1 => {
                if let Some((degree, knots, controls)) =
                    Self::decode_k32f_c32f_raw::<3>(data, data_ptr)
                {
                    if degree == 0 {
                        knots.into_iter().zip(controls).collect()
                    } else {
                        sample_bspline_vec3_full(degree, &knots, &controls)
                    }
                } else {
                    Vec::new()
                }
            }
            2 => Vec::new(),
            4 => Self::decode_d3_constant32f(data, data_ptr),
            6 => {
                let (degree, keys) = Self::decode_k16u_c16u::<3>(data, data_ptr);
                sample_vec3_if_needed(degree, keys)
            }
            7 => {
                let (degree, keys) = Self::decode_k8u_c8u::<3>(data, data_ptr);
                sample_vec3_if_needed(degree, keys)
            }
            10 => {
                let (degree, keys) = Self::decode_d3_k16u_c16u(data, data_ptr);
                sample_vec3_if_needed(degree, keys)
            }
            11 => {
                let (degree, keys) = Self::decode_d3_k8u_c8u(data, data_ptr);
                sample_vec3_if_needed(degree, keys)
            }
            other => {
                debug!("Unhandled curve3 format: {other}");
                Vec::new()
            }
        }
    }

    fn decode_curve4(data: &[u8], data_ptr: usize) -> Vec<(f32, [f32; 4])> {
        if data_ptr == 0 || data_ptr + 4 > data.len() {
            return Vec::new();
        }
        match data[data_ptr] {
            0 => Self::decode_da_keyframes32f::<4>(data, data_ptr),
            1 => {
                if let Some((degree, knots, controls)) =
                    Self::decode_k32f_c32f_raw::<4>(data, data_ptr)
                {
                    if degree == 0 {
                        knots.into_iter().zip(controls).collect()
                    } else {
                        sample_bspline_quat_full(degree, &knots, &controls)
                    }
                } else {
                    Vec::new()
                }
            }
            2 => Vec::new(),
            5 => Self::decode_d4_constant32f(data, data_ptr),
            6 => {
                let (degree, keys) = Self::decode_k16u_c16u::<4>(data, data_ptr);
                sample_quat_if_needed(degree, keys)
            }
            7 => {
                let (degree, keys) = Self::decode_k8u_c8u::<4>(data, data_ptr);
                sample_quat_if_needed(degree, keys)
            }
            8 => Self::decode_d4n_k16u_c15u(data, data_ptr),
            9 => Self::decode_d4n_k8u_c7u(data, data_ptr),
            other => {
                debug!("Unhandled curve4 format: {other}");
                Vec::new()
            }
        }
    }

    fn decode_d3_constant32f(data: &[u8], data_ptr: usize) -> Vec<(f32, [f32; 3])> {
        if data_ptr + 4 + 12 > data.len() {
            return Vec::new();
        }
        let v = [
            Self::read_f32_at(data, data_ptr + 4),
            Self::read_f32_at(data, data_ptr + 8),
            Self::read_f32_at(data, data_ptr + 12),
        ];
        vec![(0.0, v)]
    }

    fn decode_d4_constant32f(data: &[u8], data_ptr: usize) -> Vec<(f32, [f32; 4])> {
        if data_ptr + 4 + 16 > data.len() {
            return Vec::new();
        }
        let v = [
            Self::read_f32_at(data, data_ptr + 4),
            Self::read_f32_at(data, data_ptr + 8),
            Self::read_f32_at(data, data_ptr + 12),
            Self::read_f32_at(data, data_ptr + 16),
        ];
        vec![(0.0, v)]
    }

    /// DaKeyframes32f (format 0) — evenly-spaced keyframes, no explicit knot vector.
    /// Layout: header(2) + dimension(2) + controls_count(4) + controls_ptr(4) = 12 bytes
    fn decode_da_keyframes32f<const N: usize>(
        data: &[u8],
        data_ptr: usize,
    ) -> Vec<(f32, [f32; N])> {
        if data_ptr + 12 > data.len() {
            return Vec::new();
        }
        let dimension = u16::from_le_bytes([data[data_ptr + 2], data[data_ptr + 3]]) as usize;
        if dimension != N {
            return Vec::new();
        }
        let controls_count = Self::read_i32_at(data, data_ptr + 4) as usize;
        let controls_ptr = Self::read_u32_at(data, data_ptr + 8) as usize;

        if controls_count == 0 || controls_ptr == 0 {
            return Vec::new();
        }
        let num_keys = controls_count / N;
        if num_keys == 0 || !controls_count.is_multiple_of(N) {
            return Vec::new();
        }
        if controls_ptr + controls_count * 4 > data.len() {
            return Vec::new();
        }

        let mut keys = Vec::with_capacity(num_keys);
        for i in 0..num_keys {
            let mut v = [0.0f32; N];
            #[allow(clippy::needless_range_loop)]
            for j in 0..N {
                v[j] = Self::read_f32_at(data, controls_ptr + (i * N + j) * 4);
            }
            keys.push(v);
        }

        // Generate evenly-spaced times; duration not available here so use index-based
        // (nwn2mdk computes: time_step = knots_count > 1 ? duration/(knots_count-1) : 0)
        // We use normalised [0..num_keys-1] — caller's animation duration rescales.
        keys.into_iter()
            .enumerate()
            .map(|(i, v)| (i as f32, v))
            .collect()
    }

    /// D3K16uC16u (format 10) — 3D position curve with inline scales/offsets and u16 data.
    /// Layout: header(2) + one_over_knot_scale_trunc(2) + control_scales[3](12) +
    ///         control_offsets[3](12) + knots_controls_count(4) + knots_controls_ptr(4) = 36 bytes
    fn decode_d3_k16u_c16u(data: &[u8], data_ptr: usize) -> (usize, Vec<(f32, [f32; 3])>) {
        if data_ptr + 36 > data.len() {
            return (0, Vec::new());
        }
        let degree = data[data_ptr + 1] as usize;
        let scale_trunc = u16::from_le_bytes([data[data_ptr + 2], data[data_ptr + 3]]);
        let one_over_knot_scale = f32::from_bits(u32::from(scale_trunc) << 16);

        let control_scales = [
            Self::read_f32_at(data, data_ptr + 4),
            Self::read_f32_at(data, data_ptr + 8),
            Self::read_f32_at(data, data_ptr + 12),
        ];
        let control_offsets = [
            Self::read_f32_at(data, data_ptr + 16),
            Self::read_f32_at(data, data_ptr + 20),
            Self::read_f32_at(data, data_ptr + 24),
        ];
        let kc_count = Self::read_i32_at(data, data_ptr + 28) as usize;
        let kc_ptr = Self::read_u32_at(data, data_ptr + 32) as usize;

        if kc_count == 0 || kc_ptr == 0 {
            return (0, Vec::new());
        }
        if !one_over_knot_scale.is_finite() || one_over_knot_scale <= 0.0 {
            return (0, Vec::new());
        }

        // Layout: knots_count u16 knots, then controls in groups of 3 u16
        let knots_count = kc_count / 4;
        let controls_count = kc_count - knots_count;
        if knots_count == 0 || controls_count == 0 || kc_ptr + kc_count * 2 > data.len() {
            return (0, Vec::new());
        }

        let controls_start = kc_ptr + knots_count * 2;
        let num_controls = controls_count / 3;
        let count = knots_count.min(num_controls);
        let mut keys = Vec::with_capacity(count);

        for i in 0..count {
            let knot_off = kc_ptr + i * 2;
            let knot_raw = u16::from_le_bytes([data[knot_off], data[knot_off + 1]]);
            let time = f32::from(knot_raw) / one_over_knot_scale;

            let ctrl_base = controls_start + i * 6;
            if ctrl_base + 6 > data.len() {
                break;
            }
            let raw0 = u16::from_le_bytes([data[ctrl_base], data[ctrl_base + 1]]);
            let raw1 = u16::from_le_bytes([data[ctrl_base + 2], data[ctrl_base + 3]]);
            let raw2 = u16::from_le_bytes([data[ctrl_base + 4], data[ctrl_base + 5]]);
            keys.push((
                time,
                [
                    f32::from(raw0) * control_scales[0] + control_offsets[0],
                    f32::from(raw1) * control_scales[1] + control_offsets[1],
                    f32::from(raw2) * control_scales[2] + control_offsets[2],
                ],
            ));
        }

        (degree, keys)
    }

    /// D3K8uC8u (format 11) — 3D position curve with inline scales/offsets and u8 data.
    /// Same layout as D3K16uC16u but with byte-sized knots and controls.
    fn decode_d3_k8u_c8u(data: &[u8], data_ptr: usize) -> (usize, Vec<(f32, [f32; 3])>) {
        if data_ptr + 36 > data.len() {
            return (0, Vec::new());
        }
        let degree = data[data_ptr + 1] as usize;
        let scale_trunc = u16::from_le_bytes([data[data_ptr + 2], data[data_ptr + 3]]);
        let one_over_knot_scale = f32::from_bits(u32::from(scale_trunc) << 16);

        let control_scales = [
            Self::read_f32_at(data, data_ptr + 4),
            Self::read_f32_at(data, data_ptr + 8),
            Self::read_f32_at(data, data_ptr + 12),
        ];
        let control_offsets = [
            Self::read_f32_at(data, data_ptr + 16),
            Self::read_f32_at(data, data_ptr + 20),
            Self::read_f32_at(data, data_ptr + 24),
        ];
        let kc_count = Self::read_i32_at(data, data_ptr + 28) as usize;
        let kc_ptr = Self::read_u32_at(data, data_ptr + 32) as usize;

        if kc_count == 0 || kc_ptr == 0 {
            return (0, Vec::new());
        }
        if !one_over_knot_scale.is_finite() || one_over_knot_scale <= 0.0 {
            return (0, Vec::new());
        }

        // Layout: knots_count u8 knots, then controls in groups of 3 u8
        let knots_count = kc_count / 4;
        let controls_count = kc_count - knots_count;
        if knots_count == 0 || controls_count == 0 || kc_ptr + kc_count > data.len() {
            return (0, Vec::new());
        }

        let controls_start = kc_ptr + knots_count;
        let num_controls = controls_count / 3;
        let count = knots_count.min(num_controls);
        let mut keys = Vec::with_capacity(count);

        for i in 0..count {
            let knot_raw = data[kc_ptr + i];
            let time = f32::from(knot_raw) / one_over_knot_scale;

            let ctrl_base = controls_start + i * 3;
            if ctrl_base + 3 > data.len() {
                break;
            }
            keys.push((
                time,
                [
                    f32::from(data[ctrl_base]) * control_scales[0] + control_offsets[0],
                    f32::from(data[ctrl_base + 1]) * control_scales[1] + control_offsets[1],
                    f32::from(data[ctrl_base + 2]) * control_scales[2] + control_offsets[2],
                ],
            ));
        }

        (degree, keys)
    }

    /// DaK32fC32f (format 1) — uncompressed 32-bit float knots and controls.
    /// Returns (degree, knots, controls) for caller to handle B-spline sampling.
    fn decode_k32f_c32f_raw<const N: usize>(
        data: &[u8],
        data_ptr: usize,
    ) -> Option<(usize, Vec<f32>, Vec<[f32; N]>)> {
        if data_ptr + 20 > data.len() {
            return None;
        }

        let degree = data[data_ptr + 1] as usize;
        let knot_count = Self::read_i32_at(data, data_ptr + 4) as usize;
        let knots_ptr = Self::read_u32_at(data, data_ptr + 8) as usize;
        let control_count = Self::read_i32_at(data, data_ptr + 12) as usize;
        let controls_ptr = Self::read_u32_at(data, data_ptr + 16) as usize;

        if knot_count == 0 || control_count == 0 || knots_ptr == 0 || controls_ptr == 0 {
            return None;
        }
        if knots_ptr + knot_count * 4 > data.len() || controls_ptr + control_count * 4 > data.len()
        {
            return None;
        }

        let num_controls = control_count / N;
        if num_controls == 0 || !control_count.is_multiple_of(N) {
            return None;
        }

        let mut knots = Vec::with_capacity(knot_count);
        for i in 0..knot_count {
            knots.push(Self::read_f32_at(data, knots_ptr + i * 4));
        }

        let mut controls = Vec::with_capacity(num_controls);
        for i in 0..num_controls {
            let mut v = [0.0f32; N];
            #[allow(clippy::needless_range_loop)]
            for j in 0..N {
                v[j] = Self::read_f32_at(data, controls_ptr + (i * N + j) * 4);
            }
            controls.push(v);
        }

        Some((degree, knots, controls))
    }

    /// D4nK8uC7u (format 9) — compressed unit quaternion.
    /// Based on nwn2mdk by Arbos (gr2.cpp, export_gr2.cpp).
    fn decode_d4n_k8u_c7u(data: &[u8], data_ptr: usize) -> Vec<(f32, [f32; 4])> {
        if data_ptr + 16 > data.len() {
            return Vec::new();
        }

        let degree = data[data_ptr + 1] as usize;
        let scale_offset_entries = u16::from_le_bytes([data[data_ptr + 2], data[data_ptr + 3]]);
        let one_over_knot_scale = Self::read_f32_at(data, data_ptr + 4);
        let knots_controls_count = Self::read_i32_at(data, data_ptr + 8);
        let kc_ptr = Self::read_u32_at(data, data_ptr + 12) as usize;

        if knots_controls_count <= 0 || kc_ptr == 0 {
            return Vec::new();
        }
        if !one_over_knot_scale.is_finite() || one_over_knot_scale <= 0.0 {
            return Vec::new();
        }

        // Exact values from nwn2mdk lookup tables — not approximations of SQRT_2
        #[allow(clippy::approx_constant)]
        #[rustfmt::skip]
        const SCALE_TABLE: [f32; 16] = [
             1.4142135,   0.70710677,  0.35355338,  0.35355338,
             0.35355338,  0.17677669,  0.17677669,  0.17677669,
            -1.4142135,  -0.70710677, -0.35355338, -0.35355338,
            -0.35355338, -0.17677669, -0.17677669, -0.17677669,
        ];
        #[rustfmt::skip]
        const OFFSET_TABLE: [f32; 16] = [
            -0.70710677, -0.35355338, -0.53033006,  -0.17677669,
             0.17677669, -0.17677669, -0.088388346,  0.0,
             0.70710677,  0.35355338,  0.53033006,   0.17677669,
            -0.17677669,  0.17677669,  0.088388346, -0.0,
        ];

        let selectors: [usize; 4] = [
            (scale_offset_entries & 0x0F) as usize,
            ((scale_offset_entries >> 4) & 0x0F) as usize,
            ((scale_offset_entries >> 8) & 0x0F) as usize,
            ((scale_offset_entries >> 12) & 0x0F) as usize,
        ];

        const INV_127: f32 = 1.0 / 127.0;
        let mut scales = [0.0f32; 4];
        let mut offsets = [0.0f32; 4];
        for i in 0..4 {
            scales[i] = SCALE_TABLE[selectors[i]] * INV_127;
            offsets[i] = OFFSET_TABLE[selectors[i]];
        }

        // Split knots and controls: first knots_count bytes are knots, rest are 3-byte control points
        let total = knots_controls_count as usize;
        let knots_count = total / 4;
        let controls_bytes = total - knots_count;
        let num_controls = controls_bytes / 3;

        if kc_ptr + total > data.len() || knots_count == 0 || num_controls == 0 {
            return Vec::new();
        }

        let count = knots_count.min(num_controls);
        let mut knot_times = Vec::with_capacity(count);
        let mut control_points = Vec::with_capacity(count);

        for i in 0..count {
            let time = f32::from(data[kc_ptr + i]) / one_over_knot_scale;

            let ctrl_base = kc_ptr + knots_count + i * 3;
            if ctrl_base + 3 > data.len() {
                break;
            }
            let a = data[ctrl_base];
            let b = data[ctrl_base + 1];
            let c = data[ctrl_base + 2];

            // 2-bit swizzle from high bits of b and c: which component is dropped
            let swizzle1 = (((b & 0x80) >> 6) | ((c & 0x80) >> 7)) as usize;
            let swizzle2 = (swizzle1 + 1) & 3;
            let swizzle3 = (swizzle2 + 1) & 3;
            let swizzle4 = (swizzle3 + 1) & 3;

            // Dequantize 7-bit values
            let da = f32::from(a & 0x7F) * scales[swizzle2] + offsets[swizzle2];
            let db = f32::from(b & 0x7F) * scales[swizzle3] + offsets[swizzle3];
            let dc = f32::from(c & 0x7F) * scales[swizzle4] + offsets[swizzle4];

            // Reconstruct dropped component
            let sum_sq = da * da + db * db + dc * dc;
            let mut dd = (1.0 - sum_sq).max(0.0).sqrt();
            if (a & 0x80) != 0 {
                dd = -dd;
            }

            let mut quat = [0.0f32; 4];
            quat[swizzle1] = dd;
            quat[swizzle2] = da;
            quat[swizzle3] = db;
            quat[swizzle4] = dc;

            knot_times.push(time);
            control_points.push(quat);
        }

        if degree == 0 {
            knot_times.into_iter().zip(control_points).collect()
        } else {
            sample_bspline_quat(degree, &knot_times, &control_points)
        }
    }

    fn decode_d4n_k16u_c15u(data: &[u8], data_ptr: usize) -> Vec<(f32, [f32; 4])> {
        if data_ptr + 16 > data.len() {
            return Vec::new();
        }

        let degree = data[data_ptr + 1] as usize;
        let scale_offset_entries = u16::from_le_bytes([data[data_ptr + 2], data[data_ptr + 3]]);
        let one_over_knot_scale = Self::read_f32_at(data, data_ptr + 4);
        let knots_controls_count = Self::read_i32_at(data, data_ptr + 8);
        let kc_ptr = Self::read_u32_at(data, data_ptr + 12) as usize;

        if knots_controls_count <= 0 || kc_ptr == 0 {
            return Vec::new();
        }
        if !one_over_knot_scale.is_finite() || one_over_knot_scale <= 0.0 {
            return Vec::new();
        }

        #[allow(clippy::approx_constant)]
        #[rustfmt::skip]
        const SCALE_TABLE: [f32; 16] = [
             1.4142135,   0.70710677,  0.35355338,  0.35355338,
             0.35355338,  0.17677669,  0.17677669,  0.17677669,
            -1.4142135,  -0.70710677, -0.35355338, -0.35355338,
            -0.35355338, -0.17677669, -0.17677669, -0.17677669,
        ];
        #[rustfmt::skip]
        const OFFSET_TABLE: [f32; 16] = [
            -0.70710677, -0.35355338, -0.53033006,  -0.17677669,
             0.17677669, -0.17677669, -0.088388346,  0.0,
             0.70710677,  0.35355338,  0.53033006,   0.17677669,
            -0.17677669,  0.17677669,  0.088388346, -0.0,
        ];

        let selectors: [usize; 4] = [
            (scale_offset_entries & 0x0F) as usize,
            ((scale_offset_entries >> 4) & 0x0F) as usize,
            ((scale_offset_entries >> 8) & 0x0F) as usize,
            ((scale_offset_entries >> 12) & 0x0F) as usize,
        ];

        const INV_32767: f32 = 1.0 / 32767.0;
        let mut scales = [0.0f32; 4];
        let mut offsets = [0.0f32; 4];
        for i in 0..4 {
            scales[i] = SCALE_TABLE[selectors[i]] * INV_32767;
            offsets[i] = OFFSET_TABLE[selectors[i]];
        }

        // D4nK16uC15u: knots_controls_count is total u16 WORDS.
        // Sequential layout: knots first (u16 each), then controls (3 × u16 each).
        // Ratio: 1 knot word + 3 control words = 4 words per entry.
        let total_words = knots_controls_count as usize;
        let count = total_words / 4;
        let total_bytes = total_words * 2;

        if count == 0 || kc_ptr + total_bytes > data.len() {
            return Vec::new();
        }

        let controls_start = kc_ptr + count * 2; // knots occupy count × 2 bytes

        let mut knot_times = Vec::with_capacity(count);
        let mut control_points = Vec::with_capacity(count);

        for i in 0..count {
            let knot_off = kc_ptr + i * 2;
            let knot_raw = u16::from_le_bytes([data[knot_off], data[knot_off + 1]]);
            let time = f32::from(knot_raw) / one_over_knot_scale;

            let ctrl_base = controls_start + i * 6;
            if ctrl_base + 6 > data.len() {
                break;
            }
            let a = u16::from_le_bytes([data[ctrl_base], data[ctrl_base + 1]]);
            let b = u16::from_le_bytes([data[ctrl_base + 2], data[ctrl_base + 3]]);
            let c = u16::from_le_bytes([data[ctrl_base + 4], data[ctrl_base + 5]]);

            let swizzle1 = (((b & 0x8000) >> 14) | ((c & 0x8000) >> 15)) as usize;
            let swizzle2 = (swizzle1 + 1) & 3;
            let swizzle3 = (swizzle2 + 1) & 3;
            let swizzle4 = (swizzle3 + 1) & 3;

            let da = f32::from(a & 0x7FFF) * scales[swizzle2] + offsets[swizzle2];
            let db = f32::from(b & 0x7FFF) * scales[swizzle3] + offsets[swizzle3];
            let dc = f32::from(c & 0x7FFF) * scales[swizzle4] + offsets[swizzle4];

            let sum_sq = da * da + db * db + dc * dc;
            let mut dd = (1.0 - sum_sq).max(0.0).sqrt();
            if (a & 0x8000) != 0 {
                dd = -dd;
            }

            let mut quat = [0.0f32; 4];
            quat[swizzle1] = dd;
            quat[swizzle2] = da;
            quat[swizzle3] = db;
            quat[swizzle4] = dc;

            let len =
                (quat[0] * quat[0] + quat[1] * quat[1] + quat[2] * quat[2] + quat[3] * quat[3])
                    .sqrt();
            if len > 0.0 && (len - 1.0).abs() > 0.001 {
                for v in &mut quat {
                    *v /= len;
                }
            }

            knot_times.push(time);
            control_points.push(quat);
        }

        if degree == 0 {
            knot_times.into_iter().zip(control_points).collect()
        } else {
            sample_bspline_quat(degree, &knot_times, &control_points)
        }
    }

    fn decode_k16u_c16u<const N: usize>(
        data: &[u8],
        data_ptr: usize,
    ) -> (usize, Vec<(f32, [f32; N])>) {
        let degree = data[data_ptr + 1] as usize;
        let scale_trunc = u16::from_le_bytes([data[data_ptr + 2], data[data_ptr + 3]]);
        let one_over_knot_scale = f32::from_bits(u32::from(scale_trunc) << 16);
        let cso_count = Self::read_i32_at(data, data_ptr + 4) as usize;
        let cso_ptr = Self::read_u32_at(data, data_ptr + 8) as usize;
        let kc_count = Self::read_i32_at(data, data_ptr + 12) as usize;
        let kc_ptr = Self::read_u32_at(data, data_ptr + 16) as usize;

        if cso_count == 0 || kc_count == 0 || cso_ptr == 0 || kc_ptr == 0 {
            return (0, Vec::new());
        }
        if !one_over_knot_scale.is_finite() || one_over_knot_scale <= 0.0 {
            return (0, Vec::new());
        }
        let dimension = cso_count / 2;
        if dimension != N || dimension == 0 {
            return (0, Vec::new());
        }
        let knot_count = kc_count / (dimension + 1);
        if knot_count == 0 {
            return (0, Vec::new());
        }
        if cso_ptr + cso_count * 4 > data.len() || kc_ptr + kc_count * 2 > data.len() {
            return (0, Vec::new());
        }

        let mut scales = vec![0.0f32; dimension];
        let mut offsets = vec![0.0f32; dimension];
        for j in 0..dimension {
            scales[j] = Self::read_f32_at(data, cso_ptr + j * 4);
            offsets[j] = Self::read_f32_at(data, cso_ptr + (j + dimension) * 4);
        }

        let mut keys = Vec::with_capacity(knot_count);
        for i in 0..knot_count {
            let knot_off = kc_ptr + i * 2;
            let knot_raw = u16::from_le_bytes([data[knot_off], data[knot_off + 1]]);
            let time = f32::from(knot_raw) / one_over_knot_scale;

            let ctrl_base = kc_ptr + knot_count * 2 + i * dimension * 2;
            let mut v = [0.0f32; N];
            for j in 0..N {
                let off = ctrl_base + j * 2;
                if off + 2 <= data.len() {
                    let raw = u16::from_le_bytes([data[off], data[off + 1]]);
                    v[j] = f32::from(raw) * scales[j] + offsets[j];
                }
            }
            keys.push((time, v));
        }
        (degree, keys)
    }

    fn decode_k8u_c8u<const N: usize>(
        data: &[u8],
        data_ptr: usize,
    ) -> (usize, Vec<(f32, [f32; N])>) {
        let degree = data[data_ptr + 1] as usize;
        let scale_trunc = u16::from_le_bytes([data[data_ptr + 2], data[data_ptr + 3]]);
        let one_over_knot_scale = f32::from_bits(u32::from(scale_trunc) << 16);
        let cso_count = Self::read_i32_at(data, data_ptr + 4) as usize;
        let cso_ptr = Self::read_u32_at(data, data_ptr + 8) as usize;
        let kc_count = Self::read_i32_at(data, data_ptr + 12) as usize;
        let kc_ptr = Self::read_u32_at(data, data_ptr + 16) as usize;

        if cso_count == 0 || kc_count == 0 || cso_ptr == 0 || kc_ptr == 0 {
            return (0, Vec::new());
        }
        if !one_over_knot_scale.is_finite() || one_over_knot_scale <= 0.0 {
            return (0, Vec::new());
        }
        let dimension = cso_count / 2;
        if dimension != N || dimension == 0 {
            return (0, Vec::new());
        }
        let knot_count = kc_count / (dimension + 1);
        if knot_count == 0 {
            return (0, Vec::new());
        }
        if cso_ptr + cso_count * 4 > data.len() || kc_ptr + kc_count > data.len() {
            return (0, Vec::new());
        }

        let mut scales = vec![0.0f32; dimension];
        let mut offsets = vec![0.0f32; dimension];
        for j in 0..dimension {
            scales[j] = Self::read_f32_at(data, cso_ptr + j * 4);
            offsets[j] = Self::read_f32_at(data, cso_ptr + (j + dimension) * 4);
        }

        let mut keys = Vec::with_capacity(knot_count);
        for i in 0..knot_count {
            let time = f32::from(data[kc_ptr + i]) / one_over_knot_scale;

            let ctrl_base = kc_ptr + knot_count + i * dimension;
            let mut v = [0.0f32; N];
            #[allow(clippy::needless_range_loop)]
            for j in 0..N {
                let off = ctrl_base + j;
                if off < data.len() {
                    v[j] = f32::from(data[off]) * scales[j] + offsets[j];
                }
            }
            keys.push((time, v));
        }
        (degree, keys)
    }
}

/// Build padded knot vector for B-spline evaluation (matches nwn2mdk padded_knots).
fn padded_knots(knots: &[f32], degree: usize) -> Vec<f32> {
    let mut v = Vec::with_capacity(knots.len() + degree + 1);
    v.push(0.0);
    v.extend_from_slice(knots);
    let last = *knots.last().unwrap_or(&0.0);
    for i in 1..=degree {
        v[i] = 0.0;
        v.push(last);
    }
    v
}

/// Quaternion SLERP (matches FbxQuaternion::Slerp used by nwn2mdk).
fn quat_slerp(a: &[f32; 4], b: &[f32; 4], t: f32) -> [f32; 4] {
    let mut dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let mut b_adj = *b;
    if dot < 0.0 {
        dot = -dot;
        b_adj = [-b[0], -b[1], -b[2], -b[3]];
    }
    if dot > 0.9995 {
        // Linear interpolation for nearly identical quaternions
        let mut r = [0.0f32; 4];
        for i in 0..4 {
            r[i] = a[i] + t * (b_adj[i] - a[i]);
        }
        let len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2] + r[3] * r[3]).sqrt();
        if len > 0.0 {
            for v in &mut r {
                *v /= len;
            }
        }
        return r;
    }
    let theta = dot.acos();
    let sin_theta = theta.sin();
    let wa = ((1.0 - t) * theta).sin() / sin_theta;
    let wb = (t * theta).sin() / sin_theta;
    [
        wa * a[0] + wb * b_adj[0],
        wa * a[1] + wb * b_adj[1],
        wa * a[2] + wb * b_adj[2],
        wa * a[3] + wb * b_adj[3],
    ]
}

/// De Boor's algorithm for quaternion B-spline evaluation (matches nwn2mdk de_boor_rotation).
fn de_boor_quat(k: usize, knots: &[f32], controls: &[[f32; 4]], t: f32) -> [f32; 4] {
    if k == 0 || controls.is_empty() || knots.len() <= 2 * k {
        return controls.first().copied().unwrap_or([0.0, 0.0, 0.0, 1.0]);
    }

    let upper = knots.len() - k - 1;
    let mut i = k;
    while i < upper && knots[i] <= t {
        i += 1;
    }
    i = i.saturating_sub(1).max(k);

    let end = (i + 1).min(controls.len());
    let start = i.saturating_sub(k);
    if end - start <= k {
        return controls.first().copied().unwrap_or([0.0, 0.0, 0.0, 1.0]);
    }

    let mut d: Vec<[f32; 4]> = (0..=k).map(|j| controls[start + j]).collect();

    for r in 1..=k {
        for j in (r..=k).rev() {
            let ki = start + j;
            let denom = knots[ki + 1 + k - r] - knots[ki];
            let alpha = if denom != 0.0 {
                (t - knots[ki]) / denom
            } else {
                1.0
            };
            d[j] = quat_slerp(&d[j - 1], &d[j], alpha);
        }
    }
    d[k]
}

/// De Boor's algorithm for position B-spline evaluation (matches nwn2mdk de_boor_position).
fn de_boor_vec3(k: usize, knots: &[f32], controls: &[[f32; 3]], t: f32) -> [f32; 3] {
    if k == 0 || controls.is_empty() || knots.len() <= 2 * k {
        return controls.first().copied().unwrap_or([0.0, 0.0, 0.0]);
    }

    let upper = knots.len() - k - 1;
    let mut i = k;
    while i < upper && knots[i] <= t {
        i += 1;
    }
    i = i.saturating_sub(1).max(k);

    let end = (i + 1).min(controls.len());
    let start = i.saturating_sub(k);
    if end - start <= k {
        return controls.first().copied().unwrap_or([0.0, 0.0, 0.0]);
    }

    let mut d: Vec<[f32; 3]> = (0..=k).map(|j| controls[start + j]).collect();

    for r in 1..=k {
        for j in (r..=k).rev() {
            let ki = start + j;
            let denom = knots[ki + 1 + k - r] - knots[ki];
            let alpha = if denom != 0.0 {
                (t - knots[ki]) / denom
            } else {
                0.0
            };
            d[j] = [
                d[j - 1][0] * (1.0 - alpha) + d[j][0] * alpha,
                d[j - 1][1] * (1.0 - alpha) + d[j][1] * alpha,
                d[j - 1][2] * (1.0 - alpha) + d[j][2] * alpha,
            ];
        }
    }
    d[k]
}

const SAMPLE_FPS: f32 = 30.0;

/// Sample a quaternion B-spline at 30fps using De Boor's algorithm.
fn sample_bspline_quat(
    degree: usize,
    knots: &[f32],
    controls: &[[f32; 4]],
) -> Vec<(f32, [f32; 4])> {
    if controls.is_empty() {
        return Vec::new();
    }
    if controls.len() == 1 || knots.len() < degree + 1 {
        return vec![(0.0, controls[0])];
    }
    let padded = padded_knots(knots, degree);
    let duration = *knots.last().unwrap_or(&0.0);
    let time_step = 1.0 / SAMPLE_FPS;
    let mut keys = Vec::new();
    let mut i = 0u32;
    loop {
        let t = i as f32 * time_step;
        if t > duration + time_step * 0.5 {
            break;
        }
        let t_clamped = t.min(duration);
        keys.push((
            t_clamped,
            de_boor_quat(degree, &padded, controls, t_clamped),
        ));
        i += 1;
    }
    keys
}

/// Sample a position B-spline at 30fps using De Boor's algorithm.
fn sample_bspline_vec3(
    degree: usize,
    knots: &[f32],
    controls: &[[f32; 3]],
) -> Vec<(f32, [f32; 3])> {
    if controls.is_empty() {
        return Vec::new();
    }
    if controls.len() == 1 || knots.len() < degree + 1 {
        return vec![(0.0, controls[0])];
    }
    let padded = padded_knots(knots, degree);
    let duration = *knots.last().unwrap_or(&0.0);
    let time_step = 1.0 / SAMPLE_FPS;
    let mut keys = Vec::new();
    let mut i = 0u32;
    loop {
        let t = i as f32 * time_step;
        if t > duration + time_step * 0.5 {
            break;
        }
        let t_clamped = t.min(duration);
        keys.push((
            t_clamped,
            de_boor_vec3(degree, &padded, controls, t_clamped),
        ));
        i += 1;
    }
    keys
}

fn sample_vec3_if_needed(degree: usize, keys: Vec<(f32, [f32; 3])>) -> Vec<(f32, [f32; 3])> {
    if degree == 0 || keys.len() <= 1 {
        return keys;
    }
    let (knots, controls): (Vec<f32>, Vec<[f32; 3]>) = keys.into_iter().unzip();
    sample_bspline_vec3(degree, &knots, &controls)
}

fn sample_quat_if_needed(degree: usize, keys: Vec<(f32, [f32; 4])>) -> Vec<(f32, [f32; 4])> {
    if degree == 0 || keys.len() <= 1 {
        return keys;
    }
    let (knots, controls): (Vec<f32>, Vec<[f32; 4]>) = keys.into_iter().unzip();
    sample_bspline_quat(degree, &knots, &controls)
}

/// Sample a quaternion B-spline with a FULL knot vector (DaK32fC32f format).
/// Unlike sample_bspline_quat, this does NOT call padded_knots.
fn sample_bspline_quat_full(
    degree: usize,
    knots: &[f32],
    controls: &[[f32; 4]],
) -> Vec<(f32, [f32; 4])> {
    if controls.is_empty() {
        return Vec::new();
    }
    if controls.len() == 1 {
        return vec![(knots.first().copied().unwrap_or(0.0), controls[0])];
    }
    let t_start = knots.first().copied().unwrap_or(0.0);
    let t_end = knots.last().copied().unwrap_or(0.0);
    if t_end <= t_start {
        return vec![(t_start, controls[0])];
    }
    let time_step = 1.0 / SAMPLE_FPS;
    let mut keys = Vec::new();
    let mut i = 0u32;
    loop {
        let t = t_start + i as f32 * time_step;
        if t > t_end + time_step * 0.5 {
            break;
        }
        let t_clamped = t.clamp(t_start, t_end);
        keys.push((t_clamped, de_boor_quat(degree, knots, controls, t_clamped)));
        i += 1;
    }
    keys
}

/// Sample a position B-spline with a FULL knot vector (DaK32fC32f format).
fn sample_bspline_vec3_full(
    degree: usize,
    knots: &[f32],
    controls: &[[f32; 3]],
) -> Vec<(f32, [f32; 3])> {
    if controls.is_empty() {
        return Vec::new();
    }
    if controls.len() == 1 {
        return vec![(knots.first().copied().unwrap_or(0.0), controls[0])];
    }
    let t_start = knots.first().copied().unwrap_or(0.0);
    let t_end = knots.last().copied().unwrap_or(0.0);
    if t_end <= t_start {
        return vec![(t_start, controls[0])];
    }
    let time_step = 1.0 / SAMPLE_FPS;
    let mut keys = Vec::new();
    let mut i = 0u32;
    loop {
        let t = t_start + i as f32 * time_step;
        if t > t_end + time_step * 0.5 {
            break;
        }
        let t_clamped = t.clamp(t_start, t_end);
        keys.push((t_clamped, de_boor_vec3(degree, knots, controls, t_clamped)));
        i += 1;
    }
    keys
}
