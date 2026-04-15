use app_lib::services::resource_manager::ResourceManager;
use std::fmt::Write as FmtWrite;
use std::sync::Arc;
use tokio::sync::RwLock;

const DDS_MAGIC: u32 = 0x20534444;
const DDS_HEADER_SIZE: usize = 128;
const DDS_DX10_HEADER_SIZE: usize = 148;
const DDPF_FOURCC: u32 = 0x4;
const DXGI_FORMAT_BC1_UNORM: u32 = 71;
const DXGI_FORMAT_BC1_UNORM_SRGB: u32 = 72;
const DXGI_FORMAT_BC3_UNORM: u32 = 77;
const DXGI_FORMAT_BC3_UNORM_SRGB: u32 = 78;
const DXGI_FORMAT_BC7_UNORM: u32 = 98;
const DXGI_FORMAT_BC7_UNORM_SRGB: u32 = 99;

fn rgba_from_u32(buf: &[u32]) -> Vec<u8> {
    buf.iter()
        .flat_map(|&pixel| {
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            let a = ((pixel >> 24) & 0xFF) as u8;
            [r, g, b, a]
        })
        .collect()
}

fn decode_dds(dds_bytes: &[u8]) -> Result<(usize, usize, Vec<u8>), String> {
    if dds_bytes.len() < DDS_HEADER_SIZE {
        return Err("DDS file too small".into());
    }
    let magic = u32::from_le_bytes(dds_bytes[0..4].try_into().unwrap());
    if magic != DDS_MAGIC {
        return Err("Invalid DDS magic".into());
    }

    let height = u32::from_le_bytes(dds_bytes[12..16].try_into().unwrap()) as usize;
    let width = u32::from_le_bytes(dds_bytes[16..20].try_into().unwrap()) as usize;
    let pf_flags = u32::from_le_bytes(dds_bytes[80..84].try_into().unwrap());
    let fourcc = &dds_bytes[84..88];
    let has_fourcc = pf_flags & DDPF_FOURCC != 0;

    let mut buf = vec![0u32; width * height];

    if has_fourcc && fourcc == b"DX10" {
        let dxgi = u32::from_le_bytes(dds_bytes[128..132].try_into().unwrap());
        let data = &dds_bytes[DDS_DX10_HEADER_SIZE..];
        match dxgi {
            DXGI_FORMAT_BC7_UNORM | DXGI_FORMAT_BC7_UNORM_SRGB => {
                texture2ddecoder::decode_bc7(data, width, height, &mut buf)
                    .map_err(|e| format!("{e}"))?;
            }
            DXGI_FORMAT_BC1_UNORM | DXGI_FORMAT_BC1_UNORM_SRGB => {
                texture2ddecoder::decode_bc1(data, width, height, &mut buf)
                    .map_err(|e| format!("{e}"))?;
            }
            DXGI_FORMAT_BC3_UNORM | DXGI_FORMAT_BC3_UNORM_SRGB => {
                texture2ddecoder::decode_bc3(data, width, height, &mut buf)
                    .map_err(|e| format!("{e}"))?;
            }
            _ => return Err(format!("Unsupported DXGI: {dxgi}")),
        }
    } else if has_fourcc {
        let data = &dds_bytes[DDS_HEADER_SIZE..];
        match fourcc {
            b"DXT1" => {
                texture2ddecoder::decode_bc1(data, width, height, &mut buf)
                    .map_err(|e| format!("{e}"))?;
            }
            b"DXT5" => {
                texture2ddecoder::decode_bc3(data, width, height, &mut buf)
                    .map_err(|e| format!("{e}"))?;
            }
            b"DXT3" => {
                texture2ddecoder::decode_bc2(data, width, height, &mut buf)
                    .map_err(|e| format!("{e}"))?;
            }
            _ => {
                return Err(format!(
                    "Unsupported FourCC: {}",
                    String::from_utf8_lossy(fourcc)
                ));
            }
        }
    } else {
        return Err("No FourCC".into());
    }

    Ok((width, height, rgba_from_u32(&buf)))
}

fn analyze_texture(out: &mut String, name: &str, rgba: &[u8], width: usize, height: usize) {
    let pixel_count = width * height;
    let mut sum_r: u64 = 0;
    let mut sum_g: u64 = 0;
    let mut sum_b: u64 = 0;
    let mut sum_a: u64 = 0;
    let mut grey_count: u64 = 0;
    let mut histograms = [[0u32; 4]; 4];

    for i in 0..pixel_count {
        let r = rgba[i * 4] as u64;
        let g = rgba[i * 4 + 1] as u64;
        let b = rgba[i * 4 + 2] as u64;
        let a = rgba[i * 4 + 3] as u64;

        sum_r += r;
        sum_g += g;
        sum_b += b;
        sum_a += a;

        let max_c = r.max(g).max(b);
        let min_c = r.min(g).min(b);
        if max_c - min_c <= 10 {
            grey_count += 1;
        }

        for (ch, val) in [(0, r), (1, g), (2, b), (3, a)] {
            let bucket = (val / 64).min(3) as usize;
            histograms[ch][bucket] += 1;
        }
    }

    let pc = pixel_count as f64;
    let greyscale_pct = grey_count as f64 / pc * 100.0;
    let is_greyscale = greyscale_pct > 90.0;

    writeln!(out, "  {name} ({width}x{height})").unwrap();
    writeln!(
        out,
        "    Avg RGBA: ({:.1}, {:.1}, {:.1}, {:.1})",
        sum_r as f64 / pc,
        sum_g as f64 / pc,
        sum_b as f64 / pc,
        sum_a as f64 / pc
    )
    .unwrap();
    writeln!(
        out,
        "    Greyscale: {} ({:.1}%)",
        if is_greyscale { "YES" } else { "NO" },
        greyscale_pct
    )
    .unwrap();

    let labels = ["R", "G", "B", "A"];
    for (ch, label) in labels.iter().enumerate() {
        let h = &histograms[ch];
        writeln!(
            out,
            "    {label}: [0-63]={}, [64-127]={}, [128-191]={}, [192-255]={}",
            h[0], h[1], h[2], h[3]
        )
        .unwrap();
    }
}

#[tokio::test]
async fn diagnose_tint_textures() {
    let nwn2_paths = Arc::new(RwLock::new(app_lib::config::NWN2Paths::new()));
    let rm = Arc::new(RwLock::new(ResourceManager::new(nwn2_paths)));
    {
        let mut rm_guard = rm.write().await;
        let _ = rm_guard.initialize().await;
    }
    let rm = rm.read().await;

    let mut out = String::new();

    // First find actual texture names from mesh files
    writeln!(out, "=== Mesh texture references ===").unwrap();
    let meshes = ["p_hhm_ch_body05", "p_hhm_cl_body01", "p_hhm_nk_body01"];
    for mesh_name in &meshes {
        if let Ok(mdb_bytes) = rm.get_resource_bytes(mesh_name, "mdb") {
            let mdb = app_lib::parsers::mdb::parser::MdbParser::parse(&mdb_bytes).expect("parse");
            for mesh in &mdb.skin_meshes {
                if mesh.name.contains("_L0") {
                    continue;
                }
                writeln!(
                    out,
                    "  {}: diffuse='{}', tint='{}', normal='{}'",
                    mesh.name,
                    mesh.material.diffuse_map_name,
                    mesh.material.tint_map_name,
                    mesh.material.normal_map_name,
                )
                .unwrap();
            }
        }
    }

    // Analyze textures using ACTUAL names from mesh references
    let texture_pairs = [
        // Chain body05 (the problem case)
        ("p_hhm_ch_body05", "diffuse"),
        ("p_hhm_ch_body05_t", "tint_map"),
        // Cloth body01 (references body02 textures)
        ("p_hhm_cl_body02", "diffuse"),
        ("p_hhm_cl_body02_t", "tint_map"),
        // Naked body
        ("p_hhm_nk_body01", "diffuse"),
        ("p_hhm_nk_body01_t", "tint_map"),
    ];

    writeln!(out, "\n=== Texture Analysis ===").unwrap();
    for (tex_name, tex_type) in &texture_pairs {
        match rm.get_resource_bytes(tex_name, "dds") {
            Ok(bytes) => match decode_dds(&bytes) {
                Ok((w, h, rgba)) => {
                    writeln!(out, "\n  [{tex_type}]").unwrap();
                    analyze_texture(&mut out, tex_name, &rgba, w, h);

                    if *tex_type == "tint_map" {
                        let pixel_count = w * h;
                        let mut r_active = 0u64;
                        let mut g_active = 0u64;
                        let mut b_active = 0u64;
                        let mut a_zero = 0u64;
                        for i in 0..pixel_count {
                            if rgba[i * 4] > 10 {
                                r_active += 1;
                            }
                            if rgba[i * 4 + 1] > 10 {
                                g_active += 1;
                            }
                            if rgba[i * 4 + 2] > 10 {
                                b_active += 1;
                            }
                            if rgba[i * 4 + 3] == 0 {
                                a_zero += 1;
                            }
                        }
                        let pc = pixel_count as f64;
                        writeln!(
                            out,
                            "    Mask coverage: R={:.1}%, G={:.1}%, B={:.1}%",
                            r_active as f64 / pc * 100.0,
                            g_active as f64 / pc * 100.0,
                            b_active as f64 / pc * 100.0
                        )
                        .unwrap();
                        writeln!(
                            out,
                            "    Alpha=0 pixels: {:.1}%",
                            a_zero as f64 / pc * 100.0
                        )
                        .unwrap();
                    }
                }
                Err(e) => writeln!(out, "\n  {tex_name}: decode error: {e}").unwrap(),
            },
            Err(e) => writeln!(out, "\n  {tex_name}: not found: {e}").unwrap(),
        }
    }

    // Sample pixel values from the chain body diffuse
    writeln!(out, "\n\n=== Sample pixels from chain body05 diffuse ===").unwrap();
    if let Ok(bytes) = rm.get_resource_bytes("p_hhm_ch_body05", "dds") {
        if let Ok((w, h, rgba)) = decode_dds(&bytes) {
            let step_x = w / 8;
            let step_y = h / 8;
            for row in 0..8 {
                let y = row * step_y;
                let mut line = format!("  y={y:4}: ");
                for col in 0..8 {
                    let x = col * step_x;
                    let idx = (y * w + x) * 4;
                    write!(
                        line,
                        "({:3},{:3},{:3}) ",
                        rgba[idx],
                        rgba[idx + 1],
                        rgba[idx + 2]
                    )
                    .unwrap();
                }
                writeln!(out, "{line}").unwrap();
            }
        }
    }

    // Sample pixel values from the chain body tint_map
    writeln!(
        out,
        "\n=== Sample pixels from chain body05 tint_map (RGBA) ==="
    )
    .unwrap();
    if let Ok(bytes) = rm.get_resource_bytes("p_hhm_ch_body05_t", "dds") {
        if let Ok((w, h, rgba)) = decode_dds(&bytes) {
            let step_x = w / 8;
            let step_y = h / 8;
            for row in 0..8 {
                let y = row * step_y;
                let mut line = format!("  y={y:4}: ");
                for col in 0..8 {
                    let x = col * step_x;
                    let idx = (y * w + x) * 4;
                    write!(
                        line,
                        "({:3},{:3},{:3},{:3}) ",
                        rgba[idx],
                        rgba[idx + 1],
                        rgba[idx + 2],
                        rgba[idx + 3]
                    )
                    .unwrap();
                }
                writeln!(out, "{line}").unwrap();
            }
        }
    }

    // Write output to file
    let out_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target_test")
        .join("tint_texture_analysis.txt");
    std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();
    std::fs::write(&out_path, &out).unwrap();
    eprintln!("Output written to: {}", out_path.display());
}
