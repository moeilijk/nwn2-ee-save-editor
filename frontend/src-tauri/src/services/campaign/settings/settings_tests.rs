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
        
        // We need a valid minimal campaign.cam GFF file for testing.
        // Since constructing binary GFF from scratch in test is hard without a builder that supports writing (which we just added!),
        // let's try to verify the read_campaign_settings fails gracefully or mocked if possible.
        // Alternatively, we can use the GffWriter to create a test file!

        use crate::parsers::gff::{GffWriter, GffValue};
        use indexmap::IndexMap;
        
        let mut fields = IndexMap::new();
        fields.insert("LvlCap".to_string(), GffValue::Dword(30));
        fields.insert("XPCap".to_string(), GffValue::Dword(0));
        fields.insert("CompXPWt".to_string(), GffValue::Float(1.0));
        fields.insert("HenchXPWt".to_string(), GffValue::Float(0.8));
        fields.insert("GUID".to_string(), GffValue::String("test-guid".into())); // Python might use hex bytes, but let's see. 
        // Wait, the read_campaign_settings expects GUID to be Void or String. Strings are easier to write.

        // Simplified LocString construction if possible, or just skip checking LocString details for now if complex.
        // GffWriter supports LocString.
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
        // Function to set game folder source is private or we can just mock the access?
        // We can't easily mock NWN2Paths inner state without using the config setters.
        // But we can manually set the folders if we had access.
        // Since we can't easily set `campaigns` folder directly (it derives from game_folder),
        // we set the game_folder to temp_dir.path()
        let _ = paths.set_game_folder(temp_dir.path().to_str().unwrap());

        (temp_dir, paths, cam_file)
    }

    #[test]
    fn test_read_campaign_settings() {
        let (_tmp, paths, _cam_file) = setup_test_campaign();
        
        // We need to match the GUID.
        // In our setup we wrote "test-guid".
        // read_campaign_settings searches for a campaign matching the ID.
        
        let result = read_campaign_settings("test-guid", &paths);
        assert!(result.is_ok(), "Failed to read campaign settings: {:?}", result.err());
        
        let settings = result.unwrap();
        assert_eq!(settings.level_cap, 30);
        assert_eq!(settings.companion_xp_weight, 1.0);
        assert_eq!(settings.display_name, "My Campaign");
    }
}
