#[cfg(test)]
mod tests {
    use crate::services::campaign::settings::*;
    use crate::config::NWN2Paths;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_test_campaign() -> (TempDir, NWN2Paths, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let campaigns_dir = temp_dir.path().join("Campaigns");
        let my_campaign_dir = campaigns_dir.join("MyCampaign");
        fs::create_dir_all(&my_campaign_dir).unwrap();

        let cam_file = my_campaign_dir.join("campaign.cam");

        use crate::parsers::gff::{GffWriter, GffValue};
        use indexmap::IndexMap;
        
        let mut fields = IndexMap::new();
        fields.insert("LvlCap".to_string(), GffValue::Dword(30));
        fields.insert("XPCap".to_string(), GffValue::Dword(0));
        fields.insert("CompXPWt".to_string(), GffValue::Float(1.0));
        fields.insert("HenchXPWt".to_string(), GffValue::Float(0.8));
        fields.insert("GUID".to_string(), GffValue::String("test-guid".into()));

        use crate::parsers::gff::{LocalizedString, LocalizedSubstring};
        let loc_string = LocalizedString {
            string_ref: -1,
            substrings: vec![LocalizedSubstring {
                language: 0,
                gender: 0,
                string: "My Campaign".into(),
            }],
        };
        fields.insert("DisplayName".to_string(), GffValue::LocString(loc_string));
        
        
        let mut writer = GffWriter::new("CAM ", "V3.2");
        let bytes = writer.write(fields).unwrap();
        fs::write(&cam_file, bytes).unwrap();

        let mut paths = NWN2Paths::new();
        let _ = paths.set_game_folder(temp_dir.path().to_str().unwrap());

        (temp_dir, paths, cam_file)
    }

    #[test]
    fn test_read_campaign_settings() {
        let (_tmp, paths, _cam_file) = setup_test_campaign();
        let result = read_campaign_settings("test-guid", &paths);
        assert!(result.is_ok(), "Failed to read campaign settings: {:?}", result.err());
        
        let settings = result.unwrap();
        assert_eq!(settings.level_cap, 30);
        assert_eq!(settings.companion_xp_weight, 1.0);
        assert_eq!(settings.display_name, "My Campaign");
    }
}
