use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use hex;
use indexmap::IndexMap;
use tracing::{debug, warn};

use crate::parsers::erf::ErfParser;
use crate::parsers::gff::{GffParser, GffValue};
use crate::parsers::tda::TDAParser;
use crate::parsers::tlk::TLKParser;

use super::error::{ResourceManagerError, ResourceManagerResult};
use super::override_chain::{CampaignInfo, ModuleInfo};

const ERF_TYPE_2DA: u16 = 2017;
// const ERF_TYPE_TLK: u16 = 2018; // Unused
const ERF_TYPE_IFO: u16 = 2014;

pub fn extract_module_info(module_path: &Path) -> ResourceManagerResult<ModuleInfo> {
    let is_directory = module_path.is_dir();

    let ifo_data = if is_directory {
        let ifo_path = module_path.join("module.ifo");
        if !ifo_path.exists() {
            return Err(ResourceManagerError::FileNotFound(ifo_path));
        }
        std::fs::read(&ifo_path)?
    } else {
        let mut erf = ErfParser::new();
        erf.read(module_path).map_err(|e| {
            ResourceManagerError::InvalidErfFormat(format!(
                "Failed to parse module {}: {}",
                module_path.display(),
                e
            ))
        })?;

        let ifo_resource = erf.resources.iter().find(|(name, res)| {
            res.key.resource_type == ERF_TYPE_IFO
                || name.to_lowercase() == "module.ifo"
                || name.to_lowercase().ends_with(".ifo")
        });

        ifo_resource
            .and_then(|(_, res)| res.data.clone())
            .ok_or_else(|| {
                ResourceManagerError::InvalidGffFormat("module.ifo not found in module".to_string())
            })?
    };

    let gff = GffParser::from_bytes(ifo_data).map_err(|e| {
        ResourceManagerError::InvalidGffFormat(format!("Failed to parse module.ifo: {e}"))
    })?;

    let root_fields = gff.read_struct_fields(0).map_err(|e| {
        ResourceManagerError::InvalidGffFormat(format!("Failed to read module.ifo fields: {e}"))
    })?;

    let mut info = ModuleInfo {
        path: module_path.to_path_buf(),
        is_directory,
        ..Default::default()
    };

    info.name = extract_locstring_or_string(&root_fields, "Mod_Name").unwrap_or_default();
    info.mod_id = extract_string(&root_fields, "Mod_ID").unwrap_or_default();
    info.entry_area = extract_string(&root_fields, "Mod_Entry_Area").unwrap_or_default();
    info.custom_tlk = extract_string(&root_fields, "Mod_CustomTlk").unwrap_or_default();
    info.campaign_id = extract_string(&root_fields, "Campaign_ID");

    if let Some(GffValue::ListOwned(hak_entries)) = root_fields.get("Mod_HakList") {
        for entry in hak_entries {
            if let Some(hak_name) = extract_string(entry, "Mod_Hak")
                && !hak_name.is_empty()
            {
                info.hak_list.push(hak_name);
            }
        }
    }

    debug!(
        "Extracted module info: name={}, haks={:?}, custom_tlk={}",
        info.name, info.hak_list, info.custom_tlk
    );

    Ok(info)
}

pub fn load_module_2das(
    module_path: &Path,
    is_directory: bool,
) -> ResourceManagerResult<HashMap<String, Arc<TDAParser>>> {
    let mut overrides = HashMap::new();

    if is_directory {
        for entry in std::fs::read_dir(module_path)? {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("2da"))
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
            {
                match parse_2da_file(&path) {
                    Ok(parser) => {
                        overrides.insert(name.to_lowercase(), Arc::new(parser));
                    }
                    Err(e) => {
                        warn!("Failed to parse module 2DA {}: {}", path.display(), e);
                    }
                }
            }
        }
    } else {
        let mut erf = ErfParser::new();
        erf.read(module_path).map_err(|e| {
            ResourceManagerError::InvalidErfFormat(format!("Failed to parse module: {e}"))
        })?;

        for (name, resource) in &erf.resources {
            if resource.key.resource_type == ERF_TYPE_2DA {
                let tda_name = name.to_lowercase().replace(".2da", "");
                if let Some(data) = &resource.data {
                    match parse_2da_from_bytes(data) {
                        Ok(parser) => {
                            overrides.insert(tda_name, Arc::new(parser));
                        }
                        Err(e) => {
                            warn!("Failed to parse module 2DA {}: {}", name, e);
                        }
                    }
                }
            }
        }
    }

    debug!("Loaded {} 2DA overrides from module", overrides.len());
    Ok(overrides)
}

pub fn load_hak_2das(hak_path: &Path) -> ResourceManagerResult<HashMap<String, Arc<TDAParser>>> {
    let mut overrides = HashMap::new();

    let mut erf = ErfParser::new();
    erf.read(hak_path).map_err(|e| {
        ResourceManagerError::InvalidErfFormat(format!(
            "Failed to parse HAK {}: {}",
            hak_path.display(),
            e
        ))
    })?;

    for (name, resource) in &erf.resources {
        if resource.key.resource_type == ERF_TYPE_2DA {
            let tda_name = name.to_lowercase().replace(".2da", "");
            if let Some(data) = &resource.data {
                match parse_2da_from_bytes(data) {
                    Ok(parser) => {
                        overrides.insert(tda_name, Arc::new(parser));
                    }
                    Err(e) => {
                        warn!("Failed to parse HAK 2DA {}: {}", name, e);
                    }
                }
            }
        }
    }

    debug!(
        "Loaded {} 2DA overrides from HAK {}",
        overrides.len(),
        hak_path.display()
    );
    Ok(overrides)
}

pub fn check_hak_for_tlk(hak_path: &Path) -> Option<PathBuf> {
    let hak_stem = hak_path.file_stem()?.to_str()?;
    let hak_dir = hak_path.parent()?;

    let tlk_name = format!("{hak_stem}.tlk");
    let tlk_path = hak_dir.join(&tlk_name);
    if tlk_path.exists() {
        return Some(tlk_path);
    }

    let parent_dir = hak_dir.parent()?;
    let tlk_in_parent = parent_dir.join(&tlk_name);
    if tlk_in_parent.exists() {
        return Some(tlk_in_parent);
    }

    let tlk_subdir = parent_dir.join("tlk").join(&tlk_name);
    if tlk_subdir.exists() {
        return Some(tlk_subdir);
    }

    None
}

pub fn load_tlk(tlk_path: &Path) -> ResourceManagerResult<TLKParser> {
    let mut parser = TLKParser::new();
    parser.parse_from_file(tlk_path).map_err(|e| {
        ResourceManagerError::InvalidTlkFormat(format!(
            "Failed to parse TLK {}: {}",
            tlk_path.display(),
            e
        ))
    })?;
    Ok(parser)
}

pub fn extract_campaign_info(campaign_path: &Path) -> ResourceManagerResult<CampaignInfo> {
    let cam_file = if campaign_path.is_dir() {
        campaign_path.join("campaign.cam")
    } else {
        campaign_path.to_path_buf()
    };

    if !cam_file.exists() {
        return Err(ResourceManagerError::FileNotFound(cam_file));
    }

    let data = std::fs::read(&cam_file)?;
    let gff = GffParser::from_bytes(data).map_err(|e| {
        ResourceManagerError::InvalidGffFormat(format!("Failed to parse campaign.cam: {e}"))
    })?;

    let root_fields = gff.read_struct_fields(0).map_err(|e| {
        ResourceManagerError::InvalidGffFormat(format!("Failed to read campaign.cam fields: {e}"))
    })?;

    let mut info = CampaignInfo {
        path: campaign_path.to_path_buf(),
        name: String::new(),
        guid: String::new(),
        description: String::new(),
        module_names: Vec::new(),
    };

    info.name = extract_locstring_or_string(&root_fields, "DisplayName").unwrap_or_default();
    info.guid = extract_string(&root_fields, "GUID").unwrap_or_default();
    info.description = extract_locstring_or_string(&root_fields, "Description").unwrap_or_default();

    if let Some(GffValue::ListOwned(mod_entries)) = root_fields.get("ModNames") {
        for entry in mod_entries {
            if let Some(mod_name) = extract_string(entry, "ModuleName")
                && !mod_name.is_empty()
            {
                info.module_names.push(mod_name);
            }
        }
    }

    Ok(info)
}

pub fn find_hak_path(
    hak_name: &str,
    custom_hak_folders: &[PathBuf],
    user_hak_dir: Option<&PathBuf>,
    install_hak_dir: Option<&PathBuf>,
) -> Option<PathBuf> {
    let hak_filename = if hak_name.to_lowercase().ends_with(".hak") {
        hak_name.to_string()
    } else {
        format!("{hak_name}.hak")
    };

    for folder in custom_hak_folders {
        let path = folder.join(&hak_filename);
        if path.exists() {
            return Some(path);
        }
    }

    if let Some(user_dir) = user_hak_dir {
        let path = user_dir.join(&hak_filename);
        if path.exists() {
            return Some(path);
        }
    }

    if let Some(install_dir) = install_hak_dir {
        let path = install_dir.join(&hak_filename);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

pub fn find_module_path(
    module_name: &str,
    custom_module_folders: &[PathBuf],
    user_modules_dir: Option<&PathBuf>,
    install_modules_dir: Option<&PathBuf>,
    campaigns_dir: Option<&PathBuf>,
) -> Option<PathBuf> {
    let mod_filename = if module_name.to_lowercase().ends_with(".mod") {
        module_name.to_string()
    } else {
        format!("{module_name}.mod")
    };

    for folder in custom_module_folders {
        let path = folder.join(&mod_filename);
        if path.exists() {
            return Some(path);
        }
        let dir_path = folder.join(module_name);
        if dir_path.is_dir() {
            return Some(dir_path);
        }
    }

    if let Some(user_dir) = user_modules_dir {
        let path = user_dir.join(&mod_filename);
        if path.exists() {
            return Some(path);
        }
    }

    if let Some(install_dir) = install_modules_dir {
        let path = install_dir.join(&mod_filename);
        if path.exists() {
            return Some(path);
        }
    }

    if let Some(campaigns) = campaigns_dir {
        for entry in std::fs::read_dir(campaigns).ok()?.flatten() {
            let campaign_path = entry.path();
            if campaign_path.is_dir() {
                let mod_path = campaign_path.join(&mod_filename);
                if mod_path.exists() {
                    return Some(mod_path);
                }
            }
        }
    }

    None
}

pub fn find_campaign_by_guid(
    guid: &str,
    install_campaigns_dir: Option<&PathBuf>,
    user_campaigns_dir: Option<&PathBuf>,
) -> Option<PathBuf> {
    let search_dirs: Vec<&PathBuf> = [user_campaigns_dir, install_campaigns_dir]
        .into_iter()
        .flatten()
        .collect();

    for campaigns_dir in search_dirs {
        if let Ok(entries) = std::fs::read_dir(campaigns_dir) {
            for entry in entries.flatten() {
                let campaign_path = entry.path();
                if campaign_path.is_dir() {
                    let cam_file = campaign_path.join("campaign.cam");
                    if cam_file.exists()
                        && let Ok(info) = extract_campaign_info(&campaign_path)
                        && info.guid.eq_ignore_ascii_case(guid)
                    {
                        return Some(campaign_path);
                    }
                }
            }
        }
    }

    None
}

fn parse_2da_file(path: &Path) -> ResourceManagerResult<TDAParser> {
    let mut parser = TDAParser::new();
    parser.parse_from_file(path).map_err(|e| {
        ResourceManagerError::InvalidTdaFormat(format!("Failed to parse {}: {}", path.display(), e))
    })?;
    Ok(parser)
}

fn parse_2da_from_bytes(data: &[u8]) -> ResourceManagerResult<TDAParser> {
    let mut parser = TDAParser::new();
    parser.parse_from_bytes(data).map_err(|e| {
        ResourceManagerError::InvalidTdaFormat(format!("Failed to parse 2DA from bytes: {e}"))
    })?;
    Ok(parser)
}

fn extract_string(fields: &IndexMap<String, GffValue<'_>>, key: &str) -> Option<String> {
    match fields.get(key)? {
        GffValue::String(s) | GffValue::ResRef(s) => Some(s.to_string()),
        GffValue::Void(bytes) => Some(hex::encode(bytes)),
        _ => None,
    }
}

fn extract_locstring_or_string(
    fields: &IndexMap<String, GffValue<'_>>,
    key: &str,
) -> Option<String> {
    match fields.get(key)? {
        GffValue::String(s) => Some(s.to_string()),
        GffValue::LocString(ls) => ls.substrings.first().map(|sub| sub.string.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_hak_path_with_extension() {
        let custom_folders = vec![];
        let result = find_hak_path("test.hak", &custom_folders, None, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_hak_path_without_extension() {
        let custom_folders = vec![];
        let result = find_hak_path("test", &custom_folders, None, None);
        assert!(result.is_none());
    }
}
