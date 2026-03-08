pub mod globals;
pub mod content;
pub mod journal;
pub mod settings;


// use tracing::error;

use crate::services::savegame_handler::SaveGameHandler;
use self::globals::GlobalsParser;
use crate::parsers::xml::{FullSummary, QuestOverview, CompanionStatus, get_companion_definitions, XmlData};
use self::content::{ModuleInfo, ModuleVariables, extract_module_info, extract_journal, update_module_variable as update_mod_var};
use crate::config::NWN2Paths;
use self::journal::QuestDefinition;
use self::settings::{CampaignSettings, read_campaign_settings, update_campaign_settings as update_settings};
use std::collections::HashMap;

pub struct CampaignManager;

impl CampaignManager {
    pub fn get_summary(handler: &SaveGameHandler) -> Result<FullSummary, String> {
        let xml_content = handler.extract_globals_xml().map_err(|e| e.to_string())?;
        let parser = GlobalsParser::from_string(&xml_content)?;
        Ok(parser.get_full_summary_struct())
    }

    pub fn get_campaign_variables(handler: &SaveGameHandler) -> Result<XmlData, String> {
        let xml_content = handler.extract_globals_xml().map_err(|e| e.to_string())?;
        let parser = GlobalsParser::from_string(&xml_content)?;
        Ok(parser.data)
    }

    pub fn get_module_info(handler: &SaveGameHandler, paths: &NWN2Paths) -> Result<(ModuleInfo, ModuleVariables), String> {
        extract_module_info(handler, paths)
    }

    pub fn get_journal(handler: &SaveGameHandler) -> Result<HashMap<String, QuestDefinition>, String> {
        extract_journal(handler)
    }

    pub fn analyze_quest_progress(handler: &SaveGameHandler) -> Result<QuestOverview, String> {
        let xml_content = handler.extract_globals_xml().map_err(|e| e.to_string())?;
        let parser = GlobalsParser::from_string(&xml_content)?;
        Ok(parser.get_quest_overview_struct())
    }

    pub fn get_campaign_settings(campaign_id: &str, paths: &NWN2Paths) -> Result<CampaignSettings, String> {
        read_campaign_settings(campaign_id, paths)
    }

    pub fn update_campaign_settings(settings: &CampaignSettings, paths: &NWN2Paths) -> Result<(), String> {
        update_settings(settings, paths)
    }

    pub fn update_global_int(handler: &mut SaveGameHandler, name: &str, value: i32) -> Result<(), String> {
        let xml_content = handler.extract_globals_xml().map_err(|e| e.to_string())?;
        let mut parser = GlobalsParser::from_string(&xml_content)?;
        
        parser.data.integers.insert(name.to_string(), value);
        
        let new_xml = parser.to_xml_string()?;
        handler.update_file("globals.xml", new_xml.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_global_float(handler: &mut SaveGameHandler, name: &str, value: f32) -> Result<(), String> {
        let xml_content = handler.extract_globals_xml().map_err(|e| e.to_string())?;
        let mut parser = GlobalsParser::from_string(&xml_content)?;
        
        parser.data.floats.insert(name.to_string(), value);
        
        let new_xml = parser.to_xml_string()?;
        handler.update_file("globals.xml", new_xml.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_global_string(handler: &mut SaveGameHandler, name: &str, value: &str) -> Result<(), String> {
        let xml_content = handler.extract_globals_xml().map_err(|e| e.to_string())?;
        let mut parser = GlobalsParser::from_string(&xml_content)?;

        parser.data.strings.insert(name.to_string(), value.to_string());

        let new_xml = parser.to_xml_string()?;
        handler.update_file("globals.xml", new_xml.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_companion_influence(handler: &SaveGameHandler) -> Result<HashMap<String, CompanionStatus>, String> {
        let xml_content = handler.extract_globals_xml().map_err(|e| e.to_string())?;
        let parser = GlobalsParser::from_string(&xml_content)?;
        Ok(parser.get_companion_status())
    }

    pub fn update_companion_influence(handler: &mut SaveGameHandler, companion_id: &str, new_influence: i32) -> Result<(), String> {
        let defs = get_companion_definitions();
        let def = defs.get(companion_id).ok_or_else(|| format!("Unknown companion: {companion_id}"))?;
        Self::update_global_int(handler, def.influence_var, new_influence)
    }

    pub fn update_module_variable(
        handler: &SaveGameHandler,
        var_name: &str,
        value: &str,
        var_type: &str,
        module_id: Option<&str>,
    ) -> Result<(), String> {
        update_mod_var(handler, var_name, value, var_type, module_id)
    }

    pub fn update_campaign_variable(
        handler: &mut SaveGameHandler,
        var_name: &str,
        value: &str,
        var_type: &str,
    ) -> Result<(), String> {
        match var_type {
            "int" => {
                let v: i32 = value.parse().map_err(|e| format!("Invalid int value: {e}"))?;
                Self::update_global_int(handler, var_name, v)
            }
            "float" => {
                let v: f32 = value.parse().map_err(|e| format!("Invalid float value: {e}"))?;
                Self::update_global_float(handler, var_name, v)
            }
            "string" => Self::update_global_string(handler, var_name, value),
            _ => Err(format!("Unknown variable type: {var_type}")),
        }
    }
}
