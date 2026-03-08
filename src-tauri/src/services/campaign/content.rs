use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::{self};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;
use xz2::stream::{LzmaOptions, Stream};
use serde::{Deserialize, Serialize};
use indexmap::IndexMap;

use crate::parsers::erf::ErfParser;
use crate::parsers::gff::{GffParser, GffValue, GffWriter};

use crate::services::savegame_handler::SaveGameHandler;
use crate::services::campaign::journal::{parse_journal_gff, QuestDefinition};
use crate::config::NWN2Paths;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleInfo {
    pub module_name: String,
    pub area_name: String,
    pub campaign: String,
    pub entry_area: String,
    pub module_description: String,
    pub campaign_id: String,
    pub current_module: String,
    pub hak_list: Vec<String>,
    pub custom_tlk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleVariables {
    pub integers: HashMap<String, i32>,
    pub floats: HashMap<String, f32>,
    pub strings: HashMap<String, String>,
}

pub fn extract_module_info(handler: &SaveGameHandler, paths: &NWN2Paths) -> Result<(ModuleInfo, ModuleVariables), String> {
    let current_module_id = handler.extract_current_module().unwrap_or_default();
    let save_dir = handler.save_dir();

    info!("ContentManager: Current module is '{}'", current_module_id);

    // Find all .z files
    let mut z_files = Vec::new();
    if let Ok(entries) = fs::read_dir(save_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "z") {
                z_files.push(path);
            }
        }
    }

    if z_files.is_empty() {
        warn!("ContentManager: No .z files found in save directory");
        return Ok((ModuleInfo::default(), ModuleVariables::default()));
    }

    let mut found_module_info = None;
    let mut found_module_vars = None;

    // Try to find the current module first
    for z_file in &z_files {
        let file_stem = z_file.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        if file_stem == current_module_id {
            info!("ContentManager: Parsing current module .z file: {}", file_stem);
            if let Ok(res) = parse_module_z_file(z_file, &current_module_id, paths) {
                found_module_info = Some(res.0);
                found_module_vars = Some(res.1);
                break;
            }
        }
    }

    // If not found, fallback to first .z file
    if found_module_info.is_none() && !z_files.is_empty() {
        let z_file = &z_files[0];
        let file_stem = z_file.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        info!("ContentManager: Fallback to parsing: {}", file_stem);
        if let Ok(res) = parse_module_z_file(z_file, &file_stem, paths) {
            found_module_info = Some(res.0);
            found_module_vars = Some(res.1);
        }
    }

    Ok((found_module_info.unwrap_or_default(), found_module_vars.unwrap_or_default()))
}

pub fn extract_journal(handler: &SaveGameHandler) -> Result<HashMap<String, QuestDefinition>, String> {
    let current_module_id = handler.extract_current_module().unwrap_or_default();
    let save_dir = handler.save_dir();

    // Find all .z files
    let mut z_files = Vec::new();
    if let Ok(entries) = fs::read_dir(save_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "z") {
                z_files.push(path);
            }
        }
    }

    if z_files.is_empty() {
        return Ok(HashMap::new());
    }

    // Try to find the current module first
    for z_file in &z_files {
        let file_stem = z_file.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        if file_stem == current_module_id
             && let Ok(journal) = parse_journal_from_z(z_file) {
                 return Ok(journal);
             }
    }

    // Fallback to first .z file
    if !z_files.is_empty()
        && let Ok(journal) = parse_journal_from_z(&z_files[0]) {
            return Ok(journal);
        }

    Ok(HashMap::new())
}

fn parse_journal_from_z(path: &Path) -> Result<HashMap<String, QuestDefinition>, String> {
    let file = fs::File::open(path).map_err(|e| format!("Failed to open .z file: {e}"))?;
    let stream = Stream::new_lzma_decoder(u64::MAX).map_err(|e| format!("Failed to create LZMA decoder: {e}"))?;
    let mut decoder = XzDecoder::new_stream(file, stream);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).map_err(|e| format!("Failed to decompress .z file: {e}"))?;

    let mut erf_parser = ErfParser::new();
    erf_parser.parse_from_bytes(&decompressed).map_err(|e| format!("Failed to parse ERF: {e}"))?;

    let journal_bytes = erf_parser.extract_resource("module.jrl").map_err(|e| format!("module.jrl not found: {e}"))?;
    
    parse_journal_gff(&journal_bytes, &path.file_stem().unwrap().to_string_lossy())
}

fn parse_module_z_file(path: &Path, module_id: &str, nwn2_paths: &NWN2Paths) -> Result<(ModuleInfo, ModuleVariables), String> {
    // Open file
    let file = fs::File::open(path).map_err(|e| format!("Failed to open .z file: {e}"))?;
    
    // Decompress LZMA
    let stream = Stream::new_lzma_decoder(u64::MAX).map_err(|e| format!("Failed to create LZMA decoder: {e}"))?;
    let mut decoder = XzDecoder::new_stream(file, stream);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).map_err(|e| format!("Failed to decompress .z file: {e}"))?;

    // Parse ERF
    let mut erf_parser = ErfParser::new();
    erf_parser.parse_from_bytes(&decompressed).map_err(|e| format!("Failed to parse ERF from .z file: {e}"))?;

    // Extract module.ifo
    let module_ifo_bytes = erf_parser.extract_resource("module.ifo").map_err(|e| format!("module.ifo not found in .z file: {e}"))?;

    // Parse GFF
    let gff = GffParser::from_bytes(module_ifo_bytes).map_err(|e| format!("Failed to parse module.ifo GFF: {e}"))?;
    let root = gff.read_struct_fields(0).map_err(|e| format!("Failed to read root struct: {e}"))?;

    // helper functions
    let get_string = |key: &str| -> String {
        match root.get(key) {
            Some(GffValue::String(s)) => s.to_string(),
            Some(GffValue::ResRef(s)) => s.to_string(),
            Some(GffValue::LocString(ls)) => ls.substrings.first().map(|sub| sub.string.to_string()).unwrap_or_default(),
            _ => String::new(),
        }
    };
    
    // Process Module Info
    let module_name = get_string("Mod_Name");
    let module_description = get_string("Mod_Description");
    let entry_area = get_string("Mod_Entry_Area");
    let campaign_id = match root.get("Campaign_ID") {
        Some(GffValue::Void(bytes)) => hex::encode(bytes),
        Some(GffValue::String(s) | GffValue::ResRef(s)) => s.to_string(),
        _ => String::new(),
    };
    
    let campaign = if campaign_id.is_empty() {
        String::new()
    } else {
        find_campaign_name(&campaign_id, nwn2_paths).unwrap_or_else(|| {
             format!("Campaign ({})", &campaign_id[..std::cmp::min(8, campaign_id.len())])
        })
    };
    
    let custom_tlk = get_string("Mod_CustomTlk");

    let mut hak_list = Vec::new();
    if let Some(GffValue::List(haks)) = root.get("Mod_HakList") {
        for hak_lazy in haks {
            let fields = hak_lazy.force_load();
            if let Some(GffValue::String(s)) = fields.get("Mod_Hak") {
                hak_list.push(s.to_string());
            }
        }
    }

    let info = ModuleInfo {
        module_name: if module_name.is_empty() { module_id.to_string() } else { module_name },
        area_name: entry_area.clone(), // Rough approx
        campaign,
        entry_area,
        module_description,
        campaign_id,
        current_module: module_id.to_string(),
        hak_list,
        custom_tlk,
    };

    // Process Variables (VarTable)
    let mut vars = ModuleVariables::default();
    if let Some(GffValue::List(var_entries)) = root.get("VarTable") {
        for var_lazy in var_entries {
            let fields = var_lazy.force_load();
            let name = match fields.get("Name") {
                Some(GffValue::String(s)) => s.to_string(),
                _ => continue,
            };
            let type_id = match fields.get("Type") {
                Some(GffValue::Dword(v)) => *v,
                Some(GffValue::Int(v)) => *v as u32,
                _ => continue,
            };
            
            if let Some(val_field) = fields.get("Value") {
                match (type_id, val_field) {
                    (1, GffValue::Int(v)) => { vars.integers.insert(name, *v); },
                    (1, GffValue::Dword(v)) => { vars.integers.insert(name, *v as i32); },
                    (2, GffValue::Float(v)) => { vars.floats.insert(name, *v); },
                    (3, GffValue::String(s)) => { vars.strings.insert(name, s.to_string()); },
                    _ => {},
                }
            }
        }
    }

    Ok((info, vars))
}

pub fn update_module_variable(
    handler: &SaveGameHandler,
    var_name: &str,
    value: &str,
    var_type: &str,
    module_id: Option<&str>,
) -> Result<(), String> {
    let result = update_module_variable_inner(handler, var_name, value, var_type, module_id);
    if let Err(ref e) = result {
        error!("Failed to update module variable '{}': {}", var_name, e);
    }
    result
}

fn update_module_variable_inner(
    handler: &SaveGameHandler,
    var_name: &str,
    value: &str,
    var_type: &str,
    module_id: Option<&str>,
) -> Result<(), String> {
    let current_module_id = handler.extract_current_module().unwrap_or_default();
    let target_module = module_id.unwrap_or(&current_module_id);
    let save_dir = handler.save_dir();

    let z_path = find_module_z_file(save_dir, target_module)?;

    info!("Updating module variable '{}' (type={}) in {:?}", var_name, var_type, z_path);

    let file = fs::File::open(&z_path)
        .map_err(|e| format!("Failed to open .z file: {e}"))?;
    let stream = Stream::new_lzma_decoder(u64::MAX)
        .map_err(|e| format!("Failed to create LZMA decoder: {e}"))?;
    let mut decoder = XzDecoder::new_stream(file, stream);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)
        .map_err(|e| format!("Failed to decompress .z file: {e}"))?;

    info!("Decompressed {} bytes from .z file", decompressed.len());

    let mut erf_parser = ErfParser::new();
    erf_parser.parse_from_bytes(&decompressed)
        .map_err(|e| format!("Failed to parse ERF: {e}"))?;
    erf_parser.load_all_resources()
        .map_err(|e| format!("Failed to load ERF resources: {e}"))?;

    let module_ifo_bytes = erf_parser.extract_resource("module.ifo")
        .map_err(|e| format!("module.ifo not found: {e}"))?;

    info!("Extracted module.ifo ({} bytes)", module_ifo_bytes.len());

    let gff = GffParser::from_bytes(module_ifo_bytes)
        .map_err(|e| format!("Failed to parse module.ifo GFF: {e}"))?;
    let root = gff.read_struct_fields(0)
        .map_err(|e| format!("Failed to read root struct: {e}"))?;

    let mut owned_fields: IndexMap<String, GffValue<'static>> = root
        .into_iter()
        .map(|(k, v)| (k, v.force_owned()))
        .collect();

    let (type_id, gff_value): (u32, GffValue<'static>) = match var_type {
        "int" => {
            let v: i32 = value.parse().map_err(|e| format!("Invalid int value: {e}"))?;
            (1, GffValue::Int(v))
        }
        "float" => {
            let v: f32 = value.parse().map_err(|e| format!("Invalid float value: {e}"))?;
            (2, GffValue::Float(v))
        }
        "string" => (3, GffValue::String(Cow::Owned(value.to_string()))),
        _ => return Err(format!("Unknown variable type: {var_type}")),
    };

    let var_table = owned_fields
        .entry("VarTable".to_string())
        .or_insert_with(|| GffValue::ListOwned(Vec::new()));

    if let GffValue::ListOwned(entries) = var_table {
        let mut found = false;
        for entry in entries.iter_mut() {
            let name_matches = matches!(
                entry.get("Name"),
                Some(GffValue::String(s)) if s.as_ref() == var_name
            );
            if name_matches {
                entry.insert("Type".to_string(), GffValue::Dword(type_id));
                entry.insert("Value".to_string(), gff_value.clone());
                found = true;
                info!("Updated existing VarTable entry '{}'", var_name);
                break;
            }
        }
        if !found {
            let mut new_entry = IndexMap::new();
            new_entry.insert("Name".to_string(), GffValue::String(Cow::Owned(var_name.to_string())));
            new_entry.insert("Type".to_string(), GffValue::Dword(type_id));
            new_entry.insert("Value".to_string(), gff_value);
            entries.push(new_entry);
            info!("Added new VarTable entry '{}'", var_name);
        }
    } else {
        return Err(format!("VarTable is not a ListOwned, got: {:?}", std::mem::discriminant(var_table)));
    }

    let new_ifo_bytes = GffWriter::new("IFO ", "V3.2").write(owned_fields)
        .map_err(|e| format!("Failed to serialize module.ifo: {e:?}"))?;

    info!("Serialized module.ifo ({} bytes)", new_ifo_bytes.len());

    erf_parser.update_resource("module.ifo", new_ifo_bytes)
        .map_err(|e| format!("Failed to update module.ifo in ERF: {e}"))?;

    let erf_bytes = erf_parser.to_bytes()
        .map_err(|e| format!("Failed to serialize ERF: {e}"))?;

    info!("Serialized ERF ({} bytes), compressing with LZMA", erf_bytes.len());

    let lzma_opts = LzmaOptions::new_preset(6)
        .map_err(|e| format!("Failed to create LZMA options: {e}"))?;
    let lzma_stream = Stream::new_lzma_encoder(&lzma_opts)
        .map_err(|e| format!("Failed to create LZMA encoder stream: {e}"))?;
    let mut encoder = XzEncoder::new_stream(Vec::new(), lzma_stream);
    encoder.write_all(&erf_bytes)
        .map_err(|e| format!("Failed to compress ERF: {e}"))?;
    let compressed = encoder.finish()
        .map_err(|e| format!("Failed to finish LZMA compression: {e}"))?;

    info!("Compressed to {} bytes, writing to {:?}", compressed.len(), z_path);

    fs::write(&z_path, compressed)
        .map_err(|e| format!("Failed to write .z file: {e}"))?;

    info!("Successfully updated module variable '{}' in {:?}", var_name, z_path);
    Ok(())
}

fn find_module_z_file(save_dir: &Path, module_id: &str) -> Result<PathBuf, String> {
    if let Ok(entries) = fs::read_dir(save_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path.extension().is_some_and(|e| e == "z")
                && path.file_stem().is_some_and(|s| s.to_string_lossy() == module_id)
            {
                return Ok(path);
            }
        }
    }
    Err(format!("Module .z file not found for '{module_id}' in {}", save_dir.display()))
}

pub fn find_campaign_path(campaign_id: &str, paths: &NWN2Paths) -> Option<PathBuf> {
    let mut campaign_dirs = Vec::new();
    if let Some(p) = paths.user_campaigns() { campaign_dirs.push(p); }
    if let Some(p) = paths.campaigns() { campaign_dirs.push(p); }

    for dir in campaign_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let cam_file = path.join("campaign.cam");
                    if cam_file.exists()
                         && let Ok(parser) = GffParser::new(&cam_file) {
                             // Check GUID
                             let file_guid = match parser.read_field_by_label(0, "GUID") {
                                 Ok(GffValue::Void(bytes)) => hex::encode(bytes),
                                 Ok(GffValue::String(s) | GffValue::ResRef(s)) => s.to_string(),
                                 _ => continue,
                             };

                             if file_guid.to_lowercase() == campaign_id.to_lowercase() {
                                 return Some(cam_file);
                             }
                         }
                }
            }
        }
    }
    None
}

pub fn find_campaign_name(campaign_id: &str, paths: &NWN2Paths) -> Option<String> {
    if let Some(cam_file) = find_campaign_path(campaign_id, paths)
        && let Ok(parser) = GffParser::new(&cam_file) {
            return match parser.read_field_by_label(0, "DisplayName") {
                Ok(GffValue::LocString(ls)) => {
                    ls.substrings.first().map(|s| s.string.to_string())
                },
                _ => None,
            };
        }
    None
}
