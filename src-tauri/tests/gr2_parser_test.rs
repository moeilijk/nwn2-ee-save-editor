use std::io::Read;

use app_lib::parsers::gr2::decompress::gr2_decompress;
use app_lib::parsers::gr2::{Gr2Error, Gr2Parser};

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
