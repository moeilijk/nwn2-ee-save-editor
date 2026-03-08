use app_lib::services::rule_detector::{ColumnPurpose, RuleDetector, TableType};

#[test]
fn test_rule_detector_basic_detection() {
    let detector = RuleDetector::new();

    // Check detection of standard 2DA column names
    assert_eq!(
        detector.detect_column_purpose("MinLevel"),
        ColumnPurpose::MinLevel
    );
    assert_eq!(
        detector.detect_column_purpose("MININT"),
        ColumnPurpose::MinInt
    );
    assert_eq!(
        detector.detect_column_purpose("ReqFeat1"),
        ColumnPurpose::PrereqFeat
    );
    assert_eq!(
        detector.detect_column_purpose("OrFeat1"),
        ColumnPurpose::OrPrereqFeat
    );
    assert_eq!(
        detector.detect_column_purpose("FeatIndex"),
        ColumnPurpose::FeatIndex
    );
    assert_eq!(
        detector.detect_column_purpose("SpellID"),
        ColumnPurpose::SpellId
    );

    // Check unknown
    assert_eq!(
        detector.detect_column_purpose("RandomColumn"),
        ColumnPurpose::Unknown
    );
}

#[test]
fn test_rule_detector_table_relationships() {
    let mut detector = RuleDetector::new();

    // Simulate analyzing a table
    let table = "cls_feat_fighter";
    let columns = vec![
        "FeatIndex".to_string(),
        "GrantedOnLevel".to_string(),
        "UnknownCol".to_string(),
        "BonusFeatTable".to_string(), // Should detect dynamic relationship
    ];

    detector.analyze_columns(table, &columns);

    // Verify analysis result in cache
    assert_eq!(
        detector.get_column_purpose(table, "FeatIndex"),
        Some(ColumnPurpose::FeatIndex)
    );
    assert_eq!(
        detector.get_column_purpose(table, "GrantedOnLevel"),
        Some(ColumnPurpose::GrantedLevel)
    );

    // Verify relationship detection
    let rels = detector.detect_relationships(table, &columns);

    // Should find connection to 'feat' table via FeatIndex
    assert!(
        rels.iter()
            .any(|(t, c, target)| t == table && c == "FeatIndex" && target == "feat")
    );

    // Should find dynamic connection via BonusFeatTable
    assert!(
        rels.iter()
            .any(|(t, c, target)| t == table && c == "BonusFeatTable" && target == "dynamic")
    );
}

#[test]
fn test_rule_detector_table_types() {
    let detector = RuleDetector::new();

    assert_eq!(
        detector.detect_table_type("cls_feat_barb"),
        TableType::ClassFeatProgression
    );
    assert_eq!(
        detector.detect_table_type("cls_skill_rogue"),
        TableType::ClassSkillList
    );
    assert_eq!(
        detector.detect_table_type("race_feat_dwarf"),
        TableType::RaceFeat
    );
    assert_eq!(
        detector.detect_table_type("iprp_spells"),
        TableType::ItemProperty
    );
}
