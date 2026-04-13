use std::io::Read;

use app_lib::parsers::gr2::decompress::gr2_decompress;
use app_lib::parsers::gr2::{Gr2Error, Gr2Parser};

const LOD_MERGED_PATH: &str =
    "C:/Program Files (x86)/Steam/steamapps/common/NWN2 Enhanced Edition/enhanced/data/lod-merged";

const GAME_DATA_PATH: &str =
    "C:/Program Files (x86)/Steam/steamapps/common/NWN2 Enhanced Edition/Data/lod-merged.zip";

fn extract_file_from_zip(zip_path: &str, name_pattern: &str) -> Option<Vec<u8>> {
    let file = std::fs::File::open(zip_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).ok()?;
        if entry.name().to_ascii_lowercase().ends_with(name_pattern) {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).ok()?;
            return Some(buf);
        }
    }
    None
}

#[test]
fn test_parse_invalid_magic() {
    let mut data = vec![0u8; 64];
    data[0] = 0xAA;
    data[1] = 0xBB;
    data[2] = 0xCC;
    data[3] = 0xDD;

    let result = Gr2Parser::parse(&data);
    assert!(
        result.is_err(),
        "Parser must reject data with invalid magic"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, Gr2Error::InvalidMagic),
        "Expected InvalidMagic, got: {err}"
    );
}

#[test]
fn test_parse_empty_file() {
    let result = Gr2Parser::parse(&[]);
    assert!(result.is_err(), "Parser must reject empty input");
    let err = result.unwrap_err();
    assert!(
        matches!(err, Gr2Error::Io(_)),
        "Expected Io error on empty input, got: {err}"
    );
}

#[test]
fn test_decompress_empty_input() {
    let result = gr2_decompress(&[], 0, 0, 16);
    assert!(
        result.is_ok(),
        "Empty compressed input must produce zeroed output, got: {:?}",
        result.err()
    );
    let output = result.unwrap();
    assert_eq!(
        output.len(),
        16,
        "Output length must match decompressed_size"
    );
    assert!(
        output.iter().all(|&b| b == 0),
        "Output for empty input must be all zeros"
    );
}

#[test]
fn test_decompress_too_short() {
    let compressed = vec![0u8; 20];

    let result = gr2_decompress(&compressed, 0, 0, 64);
    assert!(
        result.is_err(),
        "Compressed data shorter than 36 bytes must fail"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, Gr2Error::DecompressFailed { .. }),
        "Expected DecompressFailed for truncated input, got: {err}"
    );
}

#[test]
#[ignore]
fn test_parse_real_troll_skeleton() {
    let data = extract_file_from_zip(GAME_DATA_PATH, "c_troll_skel.gr2")
        .expect("c_troll_skel.gr2 not found in lod-merged.zip");

    println!("File size: {} bytes", data.len());

    let result = Gr2Parser::parse(&data);
    match &result {
        Ok(skeleton) => {
            println!("Skeleton name: '{}'", skeleton.name);
            println!("Bone count: {}", skeleton.bones.len());
            for (i, bone) in skeleton.bones.iter().take(5).enumerate() {
                println!(
                    "  bone[{}]: '{}' parent={}",
                    i, bone.name, bone.parent_index
                );
            }
        }
        Err(e) => {
            println!("Parse error: {e}");
        }
    }

    assert!(
        result.is_ok(),
        "Failed to parse c_troll_skel.gr2: {:?}",
        result.err()
    );
}

#[test]
#[ignore]
fn test_diagnostic_troll_skeleton_sections() {
    let data = extract_file_from_zip(GAME_DATA_PATH, "c_troll_skel.gr2")
        .expect("c_troll_skel.gr2 not found in lod-merged.zip");

    println!("File size: {} bytes", data.len());

    if data.len() < 80 {
        println!("File too short for header");
        return;
    }

    let magic = [
        u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
        u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
        u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
    ];
    println!(
        "Magic: {:#010x} {:#010x} {:#010x} {:#010x}",
        magic[0], magic[1], magic[2], magic[3]
    );

    let mut off = 16;
    let _size = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
    off += 4;
    let _fmt = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
    off += 4;
    off += 8;

    let version = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
    off += 4;
    off += 4; // file_size
    off += 4; // crc32
    off += 4; // sections_offset
    let sections_count =
        u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
    off += 4;
    off += 4 * 9; // type_section, type_offset, root_section, root_offset, tag, extra[0..3]

    println!("Version: {version}, Sections: {sections_count}");

    for i in 0..sections_count as usize {
        if off + 44 > data.len() {
            break;
        }
        let compression =
            u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        off += 4;
        let data_offset =
            u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        off += 4;
        let data_size =
            u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        off += 4;
        let decomp_size =
            u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        off += 4;
        let alignment =
            u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        off += 4;
        let stop0 = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        off += 4;
        let stop1 = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
        off += 4;
        off += 4 * 4; // relocs_offset, relocs_count, marshal_offset, marshal_count

        println!(
            "Section {i}: compression={compression} data_offset={data_offset} data_size={data_size} decompressed={decomp_size} alignment={alignment} stop0={stop0} stop1={stop1}"
        );

        if compression == 2 && data_size > 0 {
            let start = data_offset as usize;
            let end = std::cmp::min(start + data_size as usize + 4, data.len());
            if start < data.len() {
                let compressed = &data[start..end];
                println!("  Attempting decompression of section {i}...");
                match gr2_decompress(compressed, stop0, stop1, decomp_size) {
                    Ok(decompressed) => {
                        println!("  Decompressed OK: {} bytes", decompressed.len());
                    }
                    Err(e) => println!("  Decompression FAILED: {e}"),
                }
            }
        }
    }
}

// --- Animation parsing tests ---

fn extract_gr2_from_lod_merged(name_pattern: &str) -> Option<Vec<u8>> {
    // Loose files in enhanced/data/lod-merged/ use a different format,
    // so always use the standard GR2 files from Data/lod-merged.zip
    extract_file_from_zip(GAME_DATA_PATH, name_pattern)
}

#[test]
fn test_parse_animations_invalid_magic() {
    let data = vec![0u8; 64];
    let result = Gr2Parser::parse_animations(&data);
    assert!(result.is_err(), "Must reject invalid magic");
}

#[test]
#[ignore]
fn test_parse_animations_from_skeleton_file() {
    let data = extract_file_from_zip(GAME_DATA_PATH, "c_troll_skel.gr2")
        .expect("c_troll_skel.gr2 not found in lod-merged.zip");

    let result = Gr2Parser::parse_animations(&data);
    assert!(
        result.is_err(),
        "Skeleton-only GR2 should return NoAnimations error"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, Gr2Error::NoAnimations),
        "Expected NoAnimations, got: {err}"
    );
}

#[test]
#[ignore]
fn test_parse_real_idle_animation() {
    let data = extract_gr2_from_lod_merged("p_hhm_idle.gr2").expect("P_HHM_idle.gr2 not found");

    println!("File size: {} bytes", data.len());

    let result = Gr2Parser::parse_animations(&data);
    match &result {
        Ok(animations) => {
            println!("Animation count: {}", animations.len());
            for (i, anim) in animations.iter().enumerate() {
                println!(
                    "  anim[{}]: '{}' duration={:.3}s time_step={:.4} tracks={}",
                    i,
                    anim.name,
                    anim.duration,
                    anim.time_step,
                    anim.tracks.len()
                );
                for (j, track) in anim.tracks.iter().take(10).enumerate() {
                    println!(
                        "    track[{}]: '{}' pos_keys={} rot_keys={} scale_keys={}",
                        j,
                        track.bone_name,
                        track.position_keys.len(),
                        track.rotation_keys.len(),
                        track.scale_keys.len(),
                    );
                    if let Some((t, v)) = track.rotation_keys.first() {
                        println!(
                            "      first_rot: t={t:.4} q=[{:.4},{:.4},{:.4},{:.4}]",
                            v[0], v[1], v[2], v[3]
                        );
                    }
                    if let Some((t, v)) = track.position_keys.first() {
                        println!(
                            "      first_pos: t={t:.4} p=[{:.4},{:.4},{:.4}]",
                            v[0], v[1], v[2]
                        );
                    }
                }
                if anim.tracks.len() > 10 {
                    println!("    ... and {} more tracks", anim.tracks.len() - 10);
                }
            }
        }
        Err(e) => println!("Parse error: {e}"),
    }

    assert!(
        result.is_ok(),
        "Failed to parse P_HHM_idle.gr2: {:?}",
        result.err()
    );
    let animations = result.unwrap();
    assert!(!animations.is_empty(), "Expected at least one animation");
    assert!(
        animations[0].duration > 0.0,
        "Animation duration must be positive"
    );
    assert!(
        !animations[0].tracks.is_empty(),
        "Expected at least one track"
    );

    // Write summary to file for inspection
    let mut summary = String::new();
    let anim = &animations[0];
    summary.push_str(&format!(
        "Animation: '{}' duration={:.3}s tracks={}\n",
        anim.name,
        anim.duration,
        anim.tracks.len()
    ));
    let mut with_rot = 0;
    let mut with_pos = 0;
    for track in &anim.tracks {
        if !track.rotation_keys.is_empty() {
            with_rot += 1;
        }
        if !track.position_keys.is_empty() {
            with_pos += 1;
        }
    }
    summary.push_str(&format!(
        "Tracks with rotation keys: {with_rot}\nTracks with position keys: {with_pos}\n"
    ));
    for track in anim.tracks.iter().take(15) {
        summary.push_str(&format!(
            "  '{}': rot={} pos={} scale={}\n",
            track.bone_name,
            track.rotation_keys.len(),
            track.position_keys.len(),
            track.scale_keys.len()
        ));
        if let Some((t, q)) = track.rotation_keys.first() {
            summary.push_str(&format!(
                "    rot[0]: t={t:.4} q=[{:.4},{:.4},{:.4},{:.4}]\n",
                q[0], q[1], q[2], q[3]
            ));
        }
        if let Some((t, p)) = track.position_keys.first() {
            summary.push_str(&format!(
                "    pos[0]: t={t:.4} p=[{:.4},{:.4},{:.4}]\n",
                p[0], p[1], p[2]
            ));
        }
    }
    std::fs::write("../target_test/anim_result.txt", &summary).unwrap();
}

#[test]
#[ignore]
fn test_audit_idle_curve_formats() {
    use app_lib::parsers::gr2::Gr2Parser;
    let data = extract_gr2_from_lod_merged("p_hhm_idle.gr2").expect("P_HHM_idle.gr2 not found");

    let formats = Gr2Parser::audit_curve_formats(&data);
    for (name, pf, of, sf) in &formats {
        println!("{name:<25} pos={pf} orient={of} scale={sf}");
    }
    assert!(!formats.is_empty());
}

#[test]
#[ignore]
fn test_parse_idle_fidget_animation() {
    let data = extract_gr2_from_lod_merged("p_hhm_idlefidgetnervous.gr2")
        .expect("P_HHM_idlefidgetnervous.gr2 not found");

    let result = Gr2Parser::parse_animations(&data);
    match &result {
        Ok(animations) => {
            for anim in animations {
                println!(
                    "Fidget '{}': {:.3}s, {} tracks",
                    anim.name,
                    anim.duration,
                    anim.tracks.len()
                );
            }
        }
        Err(e) => println!("Parse error: {e}"),
    }

    assert!(
        result.is_ok(),
        "Failed to parse fidget animation: {:?}",
        result.err()
    );
}

#[test]
#[ignore]
fn test_compare_bind_pose_vs_animation_positions() {
    let skel_data = extract_file_from_zip(GAME_DATA_PATH, "p_hhm_skel.gr2")
        .expect("p_hhm_skel.gr2 not found in lod-merged.zip");
    let anim_data = extract_file_from_zip(GAME_DATA_PATH, "p_hhm_idle.gr2")
        .expect("p_hhm_idle.gr2 not found in lod-merged.zip");

    let skeleton = Gr2Parser::parse(&skel_data).expect("Failed to parse p_hhm_skel.gr2");
    let animations =
        Gr2Parser::parse_animations(&anim_data).expect("Failed to parse p_hhm_idle.gr2");

    println!(
        "Skeleton '{}': {} bones",
        skeleton.name,
        skeleton.bones.len()
    );
    println!("Animations: {}", animations.len());

    use std::collections::HashMap;
    let bone_map: HashMap<&str, &[f32; 3]> = skeleton
        .bones
        .iter()
        .map(|b| (b.name.as_str(), &b.transform.position))
        .collect();

    for anim in &animations {
        println!(
            "\nAnimation '{}' duration={:.3}s tracks={}",
            anim.name,
            anim.duration,
            anim.tracks.len()
        );

        let tracks_with_pos: Vec<_> = anim
            .tracks
            .iter()
            .filter(|t| !t.position_keys.is_empty())
            .collect();
        println!("  Tracks with position_keys: {}", tracks_with_pos.len());
        println!(
            "  {:<30} {:>10} {:>10} {:>10}   {:>10} {:>10} {:>10}   {}",
            "bone", "bind_x", "bind_y", "bind_z", "anim_x", "anim_y", "anim_z", "keys"
        );

        for track in &tracks_with_pos {
            let bind = bone_map.get(track.bone_name.as_str()).copied();
            let first_pos = track.position_keys.first().map(|(_, p)| p);

            match (bind, first_pos) {
                (Some(b), Some(a)) => {
                    println!(
                        "  {:<30} {:>10.4} {:>10.4} {:>10.4}   {:>10.4} {:>10.4} {:>10.4}   {}",
                        track.bone_name,
                        b[0],
                        b[1],
                        b[2],
                        a[0],
                        a[1],
                        a[2],
                        track.position_keys.len()
                    );
                }
                (None, Some(a)) => {
                    println!(
                        "  {:<30} {:>10} {:>10} {:>10}   {:>10.4} {:>10.4} {:>10.4}   {} [NO BONE MATCH]",
                        track.bone_name,
                        "?",
                        "?",
                        "?",
                        a[0],
                        a[1],
                        a[2],
                        track.position_keys.len()
                    );
                }
                _ => {}
            }
        }

        let tracks_no_pos: Vec<_> = anim
            .tracks
            .iter()
            .filter(|t| t.position_keys.is_empty())
            .collect();
        if !tracks_no_pos.is_empty() {
            println!(
                "\n  Tracks WITHOUT position_keys ({}):",
                tracks_no_pos.len()
            );
            for track in tracks_no_pos.iter().take(10) {
                let bind = bone_map.get(track.bone_name.as_str());
                let bind_str = match bind {
                    Some(b) => format!("[{:.4},{:.4},{:.4}]", b[0], b[1], b[2]),
                    None => "no match".to_string(),
                };
                println!("    '{}' bind={}", track.bone_name, bind_str);
            }
            if tracks_no_pos.len() > 10 {
                println!("    ... and {} more", tracks_no_pos.len() - 10);
            }
        }
    }
}

#[test]
#[ignore]
fn test_compare_bind_pose_vs_animation_rotations() {
    let skel_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_skel.gr2").expect("p_hhm_skel.gr2 not found");
    let anim_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_idle.gr2").expect("p_hhm_idle.gr2 not found");

    let skeleton = Gr2Parser::parse(&skel_data).unwrap();
    let animations = Gr2Parser::parse_animations(&anim_data).unwrap();

    use std::collections::HashMap;
    let bone_map: HashMap<&str, &[f32; 4]> = skeleton
        .bones
        .iter()
        .map(|b| (b.name.as_str(), &b.transform.rotation))
        .collect();

    let anim = &animations[0];
    println!(
        "{:<30} {:>8} {:>8} {:>8} {:>8}   {:>8} {:>8} {:>8} {:>8}   keys",
        "bone", "bx", "by", "bz", "bw", "ax", "ay", "az", "aw"
    );

    let mut out = format!(
        "{:<30} {:>8} {:>8} {:>8} {:>8}   {:>8} {:>8} {:>8} {:>8}   keys\n",
        "bone", "bx", "by", "bz", "bw", "ax", "ay", "az", "aw"
    );

    for track in &anim.tracks {
        if track.rotation_keys.is_empty() {
            continue;
        }
        let bind = bone_map.get(track.bone_name.as_str());
        let (_, first_rot) = &track.rotation_keys[0];

        if let Some(b) = bind {
            out.push_str(&format!(
                "{:<30} {:>8.4} {:>8.4} {:>8.4} {:>8.4}   {:>8.4} {:>8.4} {:>8.4} {:>8.4}   {}\n",
                track.bone_name,
                b[0],
                b[1],
                b[2],
                b[3],
                first_rot[0],
                first_rot[1],
                first_rot[2],
                first_rot[3],
                track.rotation_keys.len()
            ));
        }
    }
    std::fs::write("../target_test/rot_compare.txt", &out).unwrap();
}

#[test]
#[ignore]
fn test_curve_format_audit() {
    use app_lib::parsers::gr2::Gr2Parser;

    let format_name = |f: u8| -> &'static str {
        match f {
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
            255 => "NULL",
            _ => "UNKNOWN",
        }
    };

    for file_name in &["p_hhf_idle.gr2", "p_hhm_idle.gr2"] {
        let data = match extract_gr2_from_lod_merged(file_name) {
            Some(d) => d,
            None => {
                println!("SKIP: {file_name} not found");
                continue;
            }
        };

        println!("\n=== {file_name} — RAW CURVE FORMATS ===");
        let formats = Gr2Parser::audit_curve_formats(&data);
        println!(
            "{:<25} {:<20} {:<20} {:<20}",
            "bone", "pos_fmt", "orient_fmt", "scale_fmt"
        );
        let mut pos_fmt_counts = std::collections::HashMap::new();
        let mut orient_fmt_counts = std::collections::HashMap::new();
        for (name, pf, of, sf) in &formats {
            println!(
                "{:<25} {:<20} {:<20} {:<20}",
                name,
                format!("{}({})", format_name(*pf), pf),
                format!("{}({})", format_name(*of), of),
                format!("{}({})", format_name(*sf), sf),
            );
            *pos_fmt_counts.entry(*pf).or_insert(0) += 1;
            *orient_fmt_counts.entry(*of).or_insert(0) += 1;
        }
        println!("\nPosition format distribution:");
        for (f, c) in &pos_fmt_counts {
            println!("  {}({}): {} bones", format_name(*f), f, c);
        }
        println!("Orientation format distribution:");
        for (f, c) in &orient_fmt_counts {
            println!("  {}({}): {} bones", format_name(*f), f, c);
        }

        // Now parse and check decoded results
        println!("\n--- DECODED RESULTS ---");
        let animations = Gr2Parser::parse_animations(&data).expect("parse failed");
        for anim in &animations {
            let mut pos_empty = 0;
            let mut rot_empty = 0;
            for (idx, track) in anim.tracks.iter().enumerate() {
                let pos_count = track.position_keys.len();
                let rot_count = track.rotation_keys.len();

                if pos_count == 0 {
                    pos_empty += 1;
                }
                if rot_count == 0 {
                    rot_empty += 1;
                }

                let (_, pf, of, _) = if idx < formats.len() {
                    &formats[idx]
                } else {
                    &(String::new(), 255, 255, 255)
                };

                // Flag when format exists but decoded to empty
                if *pf != 2 && *pf != 255 && pos_count == 0 {
                    println!(
                        "  {:<25} pos fmt={}({}) but DECODED EMPTY!",
                        track.bone_name,
                        format_name(*pf),
                        pf
                    );
                }
                if *of != 2 && *of != 255 && rot_count == 0 {
                    println!(
                        "  {:<25} orient fmt={}({}) but DECODED EMPTY!",
                        track.bone_name,
                        format_name(*of),
                        of
                    );
                }

                // Check quaternion quality
                if !track.rotation_keys.is_empty() {
                    let mut bad = 0;
                    for (_, q) in &track.rotation_keys {
                        let len_sq = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
                        if (len_sq - 1.0).abs() > 0.01 {
                            bad += 1;
                        }
                    }
                    if bad > 0 {
                        println!(
                            "  {:<25} {} BAD quats (|q|!=1) out of {}",
                            track.bone_name, bad, rot_count
                        );
                    }
                }
            }
            println!(
                "\n  Total: {} tracks, {} empty pos, {} empty rot",
                anim.tracks.len(),
                pos_empty,
                rot_empty
            );
        }
    }
}

#[test]
#[ignore]
fn test_d4n_k8u_c7u_trace() {
    let anim_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_idle.gr2").expect("p_hhm_idle.gr2 not found");

    let animations = Gr2Parser::parse_animations(&anim_data).expect("Failed to parse animations");
    let anim = &animations[0];

    let mut out = String::new();
    // Show all keyframes for a few interesting bones
    for bone_name in &["BHip1", "Head", "LArm011", "Spine", "P_HHM_skel"] {
        if let Some(track) = anim.tracks.iter().find(|t| t.bone_name == *bone_name) {
            out.push_str(&format!(
                "\n{} — {} rotation keys:\n",
                bone_name,
                track.rotation_keys.len()
            ));
            for (i, (t, q)) in track.rotation_keys.iter().enumerate() {
                let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
                out.push_str(&format!(
                    "  [{i:2}] t={t:6.3} q=[{:8.4},{:8.4},{:8.4},{:8.4}] |q|={len:.4}\n",
                    q[0], q[1], q[2], q[3]
                ));
            }
        }
    }
    std::fs::write("../target_test/d4n_trace.txt", &out).unwrap();
}

#[test]
#[ignore]
fn test_bind_vs_anim_rotation_angles() {
    let skel_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_skel.gr2").expect("p_hhm_skel.gr2 not found");
    let anim_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_idle.gr2").expect("p_hhm_idle.gr2 not found");

    let skeleton = Gr2Parser::parse(&skel_data).expect("Failed to parse skeleton");
    let animations = Gr2Parser::parse_animations(&anim_data).expect("Failed to parse animations");
    let anim = &animations[0];

    let mut out = String::new();
    out.push_str(&format!(
        "{:<30} {:>8} {:>8}  bind_quat                              anim0_quat\n",
        "bone", "angle_d", "keys"
    ));

    let mut large_angle_count = 0;
    for track in &anim.tracks {
        if track.rotation_keys.is_empty() {
            continue;
        }
        let anim_q = &track.rotation_keys[0].1;
        let bind_q = skeleton
            .bones
            .iter()
            .find(|b| b.name == track.bone_name)
            .map(|b| b.transform.rotation);

        let angle_deg = if let Some(bq) = bind_q {
            let mut dot: f32 = bq.iter().zip(anim_q.iter()).map(|(a, b)| a * b).sum();
            dot = dot.abs().min(1.0);
            2.0 * dot.acos().to_degrees()
        } else {
            -1.0
        };

        let flag = if angle_deg > 45.0 { " !!!" } else { "" };
        if angle_deg > 45.0 {
            large_angle_count += 1;
        }
        out.push_str(&format!(
            "{:<30} {:>7.1}° {:>6}  [{:8.4},{:8.4},{:8.4},{:8.4}]  [{:8.4},{:8.4},{:8.4},{:8.4}]{flag}\n",
            track.bone_name,
            angle_deg,
            track.rotation_keys.len(),
            bind_q.map_or(0.0, |q| q[0]),
            bind_q.map_or(0.0, |q| q[1]),
            bind_q.map_or(0.0, |q| q[2]),
            bind_q.map_or(0.0, |q| q[3]),
            anim_q[0], anim_q[1], anim_q[2], anim_q[3],
        ));
    }
    out.push_str(&format!(
        "\nBones with >45° difference: {large_angle_count}\n"
    ));

    std::fs::create_dir_all("../target_test").ok();
    std::fs::write("../target_test/bind_vs_anim_angles.txt", &out).unwrap();
    eprintln!("{out}");
}

#[test]
#[ignore]
fn test_bone_palette_mapping() {
    let skel_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_skel.gr2").expect("p_hhm_skel.gr2 not found");
    let skeleton = Gr2Parser::parse(&skel_data).expect("Failed to parse skeleton");

    eprintln!("=== FULL SKELETON ({} bones) ===", skeleton.bones.len());
    eprintln!("{:<5} {:<30} type", "idx", "name");
    for (i, b) in skeleton.bones.iter().enumerate() {
        let kind = if b.name.starts_with("ap_") {
            "AP (skip)"
        } else if b.name.starts_with("f_") {
            "FACE"
        } else if b.name == "Ribcage" {
            "RIBCAGE (move to end)"
        } else {
            "BODY"
        };
        eprintln!("{:<5} {:<30} {}", i, b.name, kind);
    }

    // Build the filtered body palette like nwn2mdk
    let mut body_bones: Vec<(usize, &str)> = Vec::new();
    let mut ribcage_entry: Option<(usize, &str)> = None;
    for (i, b) in skeleton.bones.iter().enumerate() {
        if b.name.starts_with("ap_") {
            // skip
        } else if b.name.starts_with("f_") {
            // face bones - separate
        } else if b.name == "Ribcage" {
            ribcage_entry = Some((i, &b.name));
        } else {
            body_bones.push((i, &b.name));
        }
    }
    if let Some(rc) = ribcage_entry {
        body_bones.push(rc);
    }

    eprintln!("\n=== BODY PALETTE ({} bones) ===", body_bones.len());
    eprintln!("{:<5} {:<5} {:<30}", "mdb_i", "skel_i", "name");
    for (palette_idx, (skel_idx, name)) in body_bones.iter().enumerate() {
        let mismatch = if palette_idx != *skel_idx {
            " ← MISMATCH"
        } else {
            ""
        };
        eprintln!(
            "{:<5} {:<5} {:<30}{}",
            palette_idx, skel_idx, name, mismatch
        );
    }
}

#[test]
#[ignore]
fn test_extract_gr2_for_dumpgr2() {
    let data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_idle.gr2").expect("p_hhm_idle.gr2 not found");
    let out_path = "../target_test/p_hhm_idle.gr2";
    std::fs::create_dir_all("../target_test").ok();
    std::fs::write(out_path, &data).expect("Failed to write GR2 file");
    eprintln!("Extracted {} bytes to {out_path}", data.len());
}

#[test]
#[ignore]
fn test_dump_curve_comparison() {
    let anim_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_idle.gr2").expect("p_hhm_idle.gr2 not found");

    let bones = &["BHip1", "Head", "LArm010", "LArm011", "Spine", "P_HHM_skel"];
    let diag = Gr2Parser::dump_curve_diagnostics(&anim_data, bones);

    // Also load skeleton to compare bind pose
    let skel_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_skel.gr2").expect("p_hhm_skel.gr2 not found");
    let skeleton = Gr2Parser::parse(&skel_data).expect("Failed to parse skeleton");

    let mut out = diag;
    out.push_str("\n\n=== BIND POSE QUATERNIONS (from skeleton) ===\n");
    for bone_name in &["BHip1", "Head", "LArm010", "LArm011", "Spine", "P_HHM_skel"] {
        if let Some(bone) = skeleton.bones.iter().find(|b| b.name == *bone_name) {
            out.push_str(&format!(
                "{}: [{:.6}, {:.6}, {:.6}, {:.6}]\n",
                bone_name,
                bone.transform.rotation[0],
                bone.transform.rotation[1],
                bone.transform.rotation[2],
                bone.transform.rotation[3],
            ));
        }
    }

    std::fs::create_dir_all("../target_test").ok();
    std::fs::write("../target_test/rust_curve_diag.txt", &out).unwrap();
    eprintln!("{out}");
}

#[test]
#[ignore]
fn test_diagnose_eye_bones() {
    let skel_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_skel.gr2").expect("p_hhm_skel.gr2 not found");
    let skeleton = Gr2Parser::parse(&skel_data).expect("Failed to parse skeleton");

    let eye_names = [
        "eyeL", "eyeR", "eyeLlid", "eyeRlid", "f_rlweye", "f_llweye", "Head", "Neck",
    ];

    eprintln!("\n=== EYE/EYELID BONE TRANSFORMS ===");
    for name in &eye_names {
        if let Some((i, b)) = skeleton
            .bones
            .iter()
            .enumerate()
            .find(|(_, b)| b.name == *name)
        {
            let t = &b.transform;
            eprintln!(
                "\n[{}] skel_idx={}, parent_idx={}",
                b.name, i, b.parent_index
            );
            eprintln!(
                "  position:  [{:.6}, {:.6}, {:.6}]",
                t.position[0], t.position[1], t.position[2]
            );
            eprintln!(
                "  rotation:  [{:.6}, {:.6}, {:.6}, {:.6}]",
                t.rotation[0], t.rotation[1], t.rotation[2], t.rotation[3]
            );
            eprintln!(
                "  scale:     [{:.6}, {:.6}, {:.6}]",
                t.scale[0], t.scale[1], t.scale[2]
            );

            let iw = &b.inverse_world_4x4;
            eprintln!(
                "  inv_world: [{:.4}, {:.4}, {:.4}, {:.4}]",
                iw[0], iw[1], iw[2], iw[3]
            );
            eprintln!(
                "             [{:.4}, {:.4}, {:.4}, {:.4}]",
                iw[4], iw[5], iw[6], iw[7]
            );
            eprintln!(
                "             [{:.4}, {:.4}, {:.4}, {:.4}]",
                iw[8], iw[9], iw[10], iw[11]
            );
            eprintln!(
                "             [{:.4}, {:.4}, {:.4}, {:.4}]",
                iw[12], iw[13], iw[14], iw[15]
            );
        } else {
            eprintln!("\n[{}] NOT FOUND in skeleton", name);
        }
    }

    // Build palettes and show face palette
    let mut face: Vec<(usize, &str)> = Vec::new();
    let mut body: Vec<(usize, &str)> = Vec::new();
    let mut ribcage: Option<(usize, &str)> = None;
    for (i, b) in skeleton.bones.iter().enumerate() {
        if b.name.starts_with("ap_") {
            // skip
        } else if b.name.starts_with("f_") {
            face.push((i, &b.name));
        } else if b.name == "Ribcage" {
            ribcage = Some((i, &b.name));
        } else {
            body.push((i, &b.name));
        }
    }
    if let Some(rc) = ribcage {
        body.push(rc);
    }

    eprintln!("\n=== FACE PALETTE ({} bones) ===", face.len());
    for (pal_idx, (skel_idx, name)) in face.iter().enumerate() {
        eprintln!("  face[{}] = skel[{}] {}", pal_idx, skel_idx, name);
    }

    eprintln!("\n=== BODY PALETTE ({} bones) ===", body.len());
    for (pal_idx, (skel_idx, name)) in body.iter().enumerate() {
        eprintln!("  body[{}] = skel[{}] {}", pal_idx, skel_idx, name);
    }

    eprintln!("\n=== CUTSCENE MESH ACCESSIBILITY ===");
    eprintln!("face.len()={}, body.len()={}", face.len(), body.len());
    eprintln!("Body bones at index < face.len() (replaced by face in cutscene):");
    for (pal_idx, (_, name)) in body.iter().enumerate() {
        if pal_idx < face.len() {
            eprintln!(
                "  body[{}] {} -- INACCESSIBLE from cutscene mesh",
                pal_idx, name
            );
        }
    }

    // Load idle animation and check eye bone tracks
    let anim_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_idle.gr2").expect("p_hhm_idle.gr2 not found");
    let anims = Gr2Parser::parse_animations(&anim_data).expect("Failed to parse animations");

    eprintln!("\n=== ANIMATION TRACKS FOR EYE/EYELID BONES ===");
    for anim in &anims {
        eprintln!("Animation: {} ({:.2}s)", anim.name, anim.duration);
        for track in &anim.tracks {
            let is_eye = eye_names.iter().any(|n| *n == track.bone_name.as_str());
            let has_eye = track.bone_name.to_lowercase().contains("eye")
                || track.bone_name.to_lowercase().contains("lid");
            if is_eye || has_eye {
                eprintln!("  Track: {}", track.bone_name);
                eprintln!("    pos keys: {}", track.position_keys.len());
                eprintln!("    rot keys: {}", track.rotation_keys.len());
                eprintln!("    scale keys: {}", track.scale_keys.len());
                if !track.rotation_keys.is_empty() {
                    let (t, q) = &track.rotation_keys[0];
                    eprintln!(
                        "    rot[0] t={:.4}: [{:.6}, {:.6}, {:.6}, {:.6}]",
                        t, q[0], q[1], q[2], q[3]
                    );
                }
                if track.rotation_keys.len() > 1 {
                    let (t, q) = &track.rotation_keys[track.rotation_keys.len() / 2];
                    eprintln!(
                        "    rot[mid] t={:.4}: [{:.6}, {:.6}, {:.6}, {:.6}]",
                        t, q[0], q[1], q[2], q[3]
                    );
                }
            }
        }
    }

    eprintln!("\n=== CURVE FORMATS ===");
    let formats = Gr2Parser::audit_curve_formats(&anim_data);
    for (name, pos_fmt, orient_fmt, scale_fmt) in &formats {
        let is_eye = eye_names.iter().any(|n| *n == name.as_str());
        let has_eye = name.to_lowercase().contains("eye") || name.to_lowercase().contains("lid");
        if is_eye || has_eye {
            eprintln!(
                "  {} -> pos={}, orient={}, scale={}",
                name, pos_fmt, orient_fmt, scale_fmt
            );
        }
    }

    // Load head MDB and check eye mesh flags + bone indices
    use app_lib::parsers::mdb::MdbParser;
    use app_lib::parsers::mdb::types::material_flags::CUTSCENE_MESH;
    let head_mdb = std::fs::read(
        "C:/Program Files (x86)/Steam/steamapps/common/NWN2 Enhanced Edition/data/nwn2_models/P_HHM_Head01.MDB"
    ).expect("Failed to read head MDB");
    let mdb = MdbParser::parse(&head_mdb).expect("Failed to parse head MDB");
    eprintln!("\n=== HEAD MDB SKIN MESHES ({}) ===", mdb.skin_meshes.len());
    for sm in &mdb.skin_meshes {
        let is_cutscene = sm.material.flags & CUTSCENE_MESH != 0;
        let mut unique_bones = std::collections::BTreeSet::new();
        for v in &sm.vertices {
            for j in 0..4 {
                if v.bone_weights[j] > 0.0 {
                    unique_bones.insert(v.bone_indices[j]);
                }
            }
        }
        eprintln!("  Mesh: {} (skel: {})", sm.name, sm.skeleton_name);
        eprintln!(
            "    flags: 0x{:02x}, CUTSCENE_MESH: {}",
            sm.material.flags, is_cutscene
        );
        eprintln!(
            "    verts: {}, faces: {}",
            sm.vertices.len(),
            sm.faces.len()
        );
        eprintln!("    unique bone indices: {:?}", unique_bones);
        let has_eye_name = sm.name.to_lowercase().contains("eye");
        if has_eye_name {
            eprintln!("    ^^^ EYE MESH ^^^");
        }
    }
}

#[test]
#[ignore]
fn test_diagnose_cape_skeleton() {
    let cape_skel_data = extract_file_from_zip(GAME_DATA_PATH, "p_hhmcapewing_skel.gr2")
        .expect("cape skeleton not found");
    let cape_skel = Gr2Parser::parse(&cape_skel_data).expect("Failed to parse cape skeleton");

    eprintln!("=== CAPE SKELETON: {} ({} bones) ===", cape_skel.name, cape_skel.bones.len());
    for (i, b) in cape_skel.bones.iter().enumerate() {
        eprintln!("  [{}] {} (parent={})", i, b.name, b.parent_index);
    }

    // Check if body idle animation has cape bone tracks
    let idle_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_idle.gr2").expect("idle not found");
    let anims = Gr2Parser::parse_animations(&idle_data).expect("Failed to parse idle");

    let cape_bone_names: Vec<&str> = cape_skel.bones.iter().map(|b| b.name.as_str()).collect();

    for anim in &anims {
        let cape_tracks: Vec<&str> = anim.tracks.iter()
            .filter(|t| cape_bone_names.contains(&t.bone_name.as_str()))
            .map(|t| t.bone_name.as_str())
            .collect();
        let body_tracks: Vec<&str> = anim.tracks.iter()
            .filter(|t| !cape_bone_names.contains(&t.bone_name.as_str()))
            .map(|t| t.bone_name.as_str())
            .collect();
        eprintln!("\nAnimation '{}' ({:.2}s): {} total tracks", anim.name, anim.duration, anim.tracks.len());
        eprintln!("  Cape tracks ({}): {:?}", cape_tracks.len(), cape_tracks);
        eprintln!("  Body tracks: {}", body_tracks.len());
    }

    // Also check fidget
    let fidget_data =
        extract_file_from_zip(GAME_DATA_PATH, "p_hhm_idlefidgetnervous.gr2").expect("fidget not found");
    let fidget_anims = Gr2Parser::parse_animations(&fidget_data).expect("Failed to parse fidget");
    for anim in &fidget_anims {
        let cape_tracks: Vec<&str> = anim.tracks.iter()
            .filter(|t| cape_bone_names.contains(&t.bone_name.as_str()))
            .map(|t| t.bone_name.as_str())
            .collect();
        eprintln!("\nFidget '{}' ({:.2}s): {} total tracks", anim.name, anim.duration, anim.tracks.len());
        eprintln!("  Cape tracks ({}): {:?}", cape_tracks.len(), cape_tracks);
    }
}
