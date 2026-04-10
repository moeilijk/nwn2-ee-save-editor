use image::{ImageBuffer, RgbaImage};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, error, info};

use crate::services::model_loader::{self, ModelData};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub filename: String,
    pub resref: String,
    pub zip_source: String,
}

fn strip_extension(name: &str) -> &str {
    if let Some(dot) = name.rfind('.') {
        &name[..dot]
    } else {
        name
    }
}

#[tauri::command]
pub fn load_model(state: State<'_, AppState>, resref: String) -> Result<ModelData, String> {
    info!("Loading model: {}", resref);
    let rm = state.resource_manager.blocking_read();
    match model_loader::load_model(&rm, &resref, "none", "none") {
        Ok(data) => {
            info!(
                "Model loaded: {} meshes, {} hooks, skeleton={}",
                data.meshes.len(),
                data.hooks.len(),
                data.skeleton.is_some()
            );
            for mesh in &data.meshes {
                debug!(
                    "  Mesh '{}' ({}): {} verts, {} indices, diffuse='{}'",
                    mesh.name,
                    mesh.mesh_type,
                    mesh.positions.len() / 3,
                    mesh.indices.len(),
                    mesh.material.diffuse_map
                );
            }
            Ok(data)
        }
        Err(e) => {
            error!("Failed to load model '{}': {}", resref, e);
            Err(e)
        }
    }
}

#[tauri::command]
pub fn get_texture_bytes(state: State<'_, AppState>, name: String) -> Result<Vec<u8>, String> {
    debug!("Loading texture: {}", name);
    let rm = state.resource_manager.blocking_read();
    match rm.get_resource_bytes(&name, "dds") {
        Ok(bytes) => {
            debug!("Texture loaded: {} ({} bytes)", name, bytes.len());
            Ok(bytes)
        }
        Err(e) => {
            error!("Texture not found '{}': {}", name, e);
            Err(format!("Texture not found {name}: {e}"))
        }
    }
}

#[tauri::command]
pub fn get_icon_png(state: State<'_, AppState>, name: String) -> Result<String, String> {
    let rm = state.resource_manager.blocking_read();

    // 1. Check indexed icon files (upscaled DDS, workshop overrides)
    if let Some(icon_path) = rm.get_icon_path(&name) {
        let path: &std::path::Path = &icon_path;
        let bytes = std::fs::read(path).map_err(|e| format!("Failed to read icon {name}: {e}"))?;

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("dds")
            .to_lowercase();

        let png_bytes = if ext == "tga" {
            decode_tga_to_png(&bytes)
                .map_err(|e| format!("Failed to decode TGA icon {name}: {e}"))?
        } else {
            decode_dds_to_png(&bytes)
                .map_err(|e| format!("Failed to decode DDS icon {name}: {e}"))?
        };
        return encode_png_data_url(&png_bytes);
    }

    // 2. Fallback to get_resource_bytes (HAKs, override, zips)
    if let Ok(dds_bytes) = rm.get_resource_bytes(&name, "dds") {
        let png_bytes = decode_dds_to_png(&dds_bytes)
            .map_err(|e| format!("Failed to decode icon {name}: {e}"))?;
        return encode_png_data_url(&png_bytes);
    }

    if let Ok(tga_bytes) = rm.get_resource_bytes(&name, "tga") {
        let png_bytes = decode_tga_to_png(&tga_bytes)
            .map_err(|e| format!("Failed to decode TGA icon {name}: {e}"))?;
        return encode_png_data_url(&png_bytes);
    }

    Err(format!("Icon not found: {name}"))
}

fn encode_png_data_url(png_bytes: &[u8]) -> Result<String, String> {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(png_bytes);
    Ok(format!("data:image/png;base64,{b64}"))
}

fn decode_tga_to_png(tga_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory_with_format(tga_bytes, image::ImageFormat::Tga)
        .map_err(|e| format!("TGA decode failed: {e}"))?;
    let mut png_buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut png_buf, image::ImageFormat::Png)
        .map_err(|e| format!("PNG encode failed: {e}"))?;
    Ok(png_buf.into_inner())
}

const DDS_MAGIC: u32 = 0x2053_4444;
const DDS_HEADER_SIZE: usize = 128;
const DDS_DX10_HEADER_SIZE: usize = 148;
const DDPF_FOURCC: u32 = 0x4;

const DXGI_FORMAT_BC1_UNORM: u32 = 71;
const DXGI_FORMAT_BC1_UNORM_SRGB: u32 = 72;
const DXGI_FORMAT_BC3_UNORM: u32 = 77;
const DXGI_FORMAT_BC3_UNORM_SRGB: u32 = 78;
const DXGI_FORMAT_BC7_UNORM: u32 = 98;
const DXGI_FORMAT_BC7_UNORM_SRGB: u32 = 99;

macro_rules! decode_bc {
    ($pixel_data:expr, $w:expr, $h:expr, $decoder:path, $name:literal) => {{
        let mut buf = vec![0u32; $w * $h];
        $decoder($pixel_data, $w, $h, &mut buf)
            .map_err(|e| format!(concat!($name, " decode failed: {}"), e))?;
        rgba_from_u32(&buf)
    }};
}

fn decode_dds_to_png(dds_bytes: &[u8]) -> Result<Vec<u8>, String> {
    if dds_bytes.len() < DDS_DX10_HEADER_SIZE {
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

    let rgba = if has_fourcc && fourcc == b"DX10" {
        let dxgi_format = u32::from_le_bytes(dds_bytes[128..132].try_into().unwrap());
        let pixel_data = &dds_bytes[DDS_DX10_HEADER_SIZE..];

        match dxgi_format {
            DXGI_FORMAT_BC7_UNORM | DXGI_FORMAT_BC7_UNORM_SRGB => {
                decode_bc!(
                    pixel_data,
                    width,
                    height,
                    texture2ddecoder::decode_bc7,
                    "BC7"
                )
            }
            DXGI_FORMAT_BC1_UNORM | DXGI_FORMAT_BC1_UNORM_SRGB => {
                decode_bc!(
                    pixel_data,
                    width,
                    height,
                    texture2ddecoder::decode_bc1,
                    "BC1"
                )
            }
            DXGI_FORMAT_BC3_UNORM | DXGI_FORMAT_BC3_UNORM_SRGB => {
                decode_bc!(
                    pixel_data,
                    width,
                    height,
                    texture2ddecoder::decode_bc3,
                    "BC3"
                )
            }
            _ => return Err(format!("Unsupported DXGI format: {dxgi_format}")),
        }
    } else if has_fourcc {
        let pixel_data = &dds_bytes[DDS_HEADER_SIZE..];

        match fourcc {
            b"DXT1" => decode_bc!(
                pixel_data,
                width,
                height,
                texture2ddecoder::decode_bc1,
                "DXT1"
            ),
            b"DXT5" => decode_bc!(
                pixel_data,
                width,
                height,
                texture2ddecoder::decode_bc3,
                "DXT5"
            ),
            b"DXT3" => decode_bc!(
                pixel_data,
                width,
                height,
                texture2ddecoder::decode_bc2,
                "DXT3"
            ),
            _ => {
                let cc = String::from_utf8_lossy(fourcc);
                return Err(format!("Unsupported FourCC: {cc}"));
            }
        }
    } else {
        return Err("Unsupported DDS format: no FourCC".into());
    };

    let img: RgbaImage = ImageBuffer::from_raw(width as u32, height as u32, rgba)
        .ok_or("Failed to create image buffer")?;

    let mut png_buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut png_buf, image::ImageFormat::Png)
        .map_err(|e| format!("PNG encode failed: {e}"))?;

    Ok(png_buf.into_inner())
}

/// texture2ddecoder outputs BGRA packed in u32; convert to RGBA byte array.
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

#[tauri::command]
pub fn list_available_models(state: State<'_, AppState>) -> Result<Vec<ModelEntry>, String> {
    let mut cache = state.model_list_cache.lock();
    if let Some(cached) = cache.as_ref() {
        info!("Returning {} cached MDB models", cached.len());
        return Ok(cached.clone());
    }

    info!("Scanning for available models...");
    let rm = state.resource_manager.blocking_read();
    let files = rm.list_resources_by_extension("mdb");
    let count = files.len();
    let result: Vec<ModelEntry> = files
        .into_iter()
        .map(|(filename, zip_source)| {
            let basename = filename.rsplit('/').next().unwrap_or(&filename);
            let resref = strip_extension(basename).to_string();
            ModelEntry {
                filename,
                resref,
                zip_source,
            }
        })
        .collect();
    info!("Found {} MDB models", count);
    *cache = Some(result.clone());
    Ok(result)
}
