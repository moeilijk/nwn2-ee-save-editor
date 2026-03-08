use serde::{Deserialize, Serialize};
use crate::parsers::gff::{GffParser, GffValue, GffWriter};
use std::path::{Path, PathBuf};
use crate::config::NWN2Paths;
use std::fs;
use tracing::{info, warn};
use indexmap::IndexMap;
use chrono::Local;

// Forward declaration if we can't import it yet, but we will fix content.rs shortly.
// For now, let's assume we can import it.
use super::content::find_campaign_path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CampaignSettings {
    pub campaign_file_path: String,
    pub guid: String,
    pub level_cap: u32,
    pub xp_cap: u32,
    pub companion_xp_weight: f32,
    pub henchman_xp_weight: f32,
    pub attack_neutrals: bool,
    pub auto_xp_award: bool,
    pub journal_sync: bool,
    pub no_char_changing: bool,
    pub use_personal_reputation: bool,
    pub start_module: String,
    pub module_names: Vec<String>,
    pub display_name: String,
    pub description: String,
}

pub fn read_campaign_settings(campaign_id: &str, paths: &NWN2Paths) -> Result<CampaignSettings, String> {
    let campaign_file = find_campaign_path(campaign_id, paths)
        .ok_or_else(|| format!("Campaign file for GUID {campaign_id} not found"))?;

    let parser = GffParser::new(&campaign_file)
        .map_err(|e| format!("Failed to parse campaign file: {e}"))?;

    let root = parser.read_struct_fields(0)
        .map_err(|e| format!("Failed to read root struct: {e}"))?;

    let mut settings = CampaignSettings {
        campaign_file_path: campaign_file.to_string_lossy().to_string(),
        ..Default::default()
    };

    // Helper closure to getting values
    let get_bool = |key: &str| -> bool {
        match root.get(key) {
            Some(GffValue::Byte(v)) => *v != 0,
            Some(GffValue::Int(v)) => *v != 0,
            _ => false,
        }
    };

    let get_u32 = |key: &str| -> u32 {
        match root.get(key) {
            Some(GffValue::Dword(v)) => *v,
            Some(GffValue::Int(v)) => *v as u32,
            Some(GffValue::Byte(v)) => u32::from(*v),
            _ => 0,
        }
    };

    let get_f32 = |key: &str| -> f32 {
        match root.get(key) {
            Some(GffValue::Float(v)) => *v,
            _ => 0.0,
        }
    };
    
    let get_string = |key: &str| -> String {
         match root.get(key) {
            Some(GffValue::String(s) | GffValue::ResRef(s)) => s.to_string(),
             _ => String::new(),
         }
    };

    // GUID
    settings.guid = match root.get("GUID") {
          Some(GffValue::Void(bytes)) => hex::encode(bytes),
          Some(GffValue::String(s)) => s.to_string(),
          _ => String::new(),
    };

    settings.level_cap = get_u32("LvlCap");
    settings.xp_cap = get_u32("XPCap");
    settings.companion_xp_weight = get_f32("CompXPWt");
    settings.henchman_xp_weight = get_f32("HenchXPWt");
    settings.attack_neutrals = get_bool("AttackNeut");
    settings.auto_xp_award = get_bool("AutoXPAwd");
    settings.journal_sync = get_bool("JournalSynch");
    settings.no_char_changing = get_bool("NoCharChanging");
    settings.use_personal_reputation = get_bool("UsePersonalRep");
    settings.start_module = get_string("StartModule");

    // Module Names
    if let Some(GffValue::List(mod_names)) = root.get("ModNames") {
        for item in mod_names {
             let fields = item.force_load();
             if let Some(GffValue::String(s) | GffValue::ResRef(s)) = fields.get("ModuleName") {
                 settings.module_names.push(s.to_string());
             }
        }
    }

    // Display Name
    if let Some(GffValue::LocString(ls)) = root.get("DisplayName") {
         settings.display_name = ls.substrings.first().map(|s| s.string.clone().into_owned()).unwrap_or_default();
    }
    
    // Description
    if let Some(GffValue::LocString(ls)) = root.get("Description") {
         settings.description = ls.substrings.first().map(|s| s.string.clone().into_owned()).unwrap_or_default();
    }
    
    Ok(settings)
}

pub fn update_campaign_settings(settings: &CampaignSettings, paths: &NWN2Paths) -> Result<(), String> {
    let campaign_file = PathBuf::from(&settings.campaign_file_path);
    if !campaign_file.exists() {
        return Err(format!("Campaign file not found: {}", settings.campaign_file_path));
    }

    // Backup
    if let Err(e) = backup_campaign_file(&campaign_file, settings, paths) {
        warn!("Failed to backup campaign file: {}", e);
    }
    
    // Read existing GFF to preserve other fields
    let parser = GffParser::new(&campaign_file)
         .map_err(|e| format!("Failed to parse campaign file for update: {e}"))?;
    
    let root_fields = parser.read_struct_fields(0)
         .map_err(|e| format!("Failed to read root struct: {e}"))?;
         
    // Convert to owned map
    let mut owned_fields: IndexMap<String, GffValue<'static>> = root_fields
        .into_iter()
        .map(|(k, v)| (k, v.into_owned()))
        .collect();
    
    // Update fields
    owned_fields.insert("LvlCap".to_string(), GffValue::Dword(settings.level_cap));
    owned_fields.insert("XPCap".to_string(), GffValue::Dword(settings.xp_cap));
    owned_fields.insert("CompXPWt".to_string(), GffValue::Float(settings.companion_xp_weight));
    owned_fields.insert("HenchXPWt".to_string(), GffValue::Float(settings.henchman_xp_weight));
    owned_fields.insert("AttackNeut".to_string(), GffValue::Byte(u8::from(settings.attack_neutrals)));
    owned_fields.insert("AutoXPAwd".to_string(), GffValue::Byte(u8::from(settings.auto_xp_award)));
    owned_fields.insert("JournalSynch".to_string(), GffValue::Byte(u8::from(settings.journal_sync)));
    owned_fields.insert("NoCharChanging".to_string(), GffValue::Byte(u8::from(settings.no_char_changing)));
    owned_fields.insert("UsePersonalRep".to_string(), GffValue::Byte(u8::from(settings.use_personal_reputation)));
    
    // Write back
    let mut writer = GffWriter::new("CAM ", "V3.2");
    
    let bytes = writer.write(owned_fields)
        .map_err(|e| format!("Failed to serialize campaign settings: {e:?}"))?;

    fs::write(&campaign_file, bytes)
        .map_err(|e| format!("Failed to write campaign file: {e}"))?;

    info!("Updated campaign settings in {:?}", campaign_file);
    Ok(())
}

fn backup_campaign_file(path: &Path, settings: &CampaignSettings, paths: &NWN2Paths) -> Result<PathBuf, String> {
    let backup_dir = paths.saves().ok_or("Saves path not configured")?.join("backups").join("campaign_backups");
    
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let safe_name: String = settings.display_name.chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect();
    let safe_name = safe_name.replace(' ', "_");
    let safe_name = if safe_name.len() > 30 { &safe_name[..30] } else { &safe_name };
    
    let guid_prefix: String = settings.guid.chars().take(8).collect();
    let filename = format!("{safe_name}_{guid_prefix}_{timestamp}.cam");
    let backup_path = backup_dir.join(filename);
    
    fs::copy(path, &backup_path).map_err(|e| e.to_string())?;
    
    info!("Created campaign backup at {:?}", backup_path);
    Ok(backup_path)
}

#[cfg(test)]
mod settings_tests;
