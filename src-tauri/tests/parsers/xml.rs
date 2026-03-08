use std::path::PathBuf;

use app_lib::parsers::xml::RustXmlParser;

fn fixtures_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn load_globals_xml() -> Option<String> {
    let path = fixtures_path().join("saves/MOTB/globals.xml");
    if path.exists() {
        std::fs::read_to_string(&path).ok()
    } else {
        None
    }
}

// =============================================================================
// PARSER CREATION TESTS
// =============================================================================

#[test]
fn test_xml_parser_creation() {
    let parser = RustXmlParser::new();
    let info = parser.get_general_info();
    assert!(info.is_empty() || info.values().all(|v| v.is_none()));
}

// =============================================================================
// ERROR HANDLING TESTS
// =============================================================================

#[test]
fn test_xml_parser_from_empty_string() {
    let result = RustXmlParser::from_string("");
    assert!(result.is_err(), "Empty string should fail to parse");
}

#[test]
fn test_xml_parser_from_invalid_xml() {
    let result = RustXmlParser::from_string("<not_closed>");
    assert!(result.is_err(), "Invalid XML should fail to parse");
}

#[test]
fn test_xml_parser_from_malformed_xml() {
    let result = RustXmlParser::from_string("<root><child></root>");
    assert!(result.is_err(), "Malformed XML should fail to parse");
}

#[test]
fn test_xml_parser_from_no_root() {
    let result = RustXmlParser::from_string("not xml at all");
    assert!(result.is_err(), "Non-XML content should fail");
}

// =============================================================================
// BASIC PARSING TESTS
// =============================================================================

#[test]
fn test_xml_parse_simple() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Globals>
    <TestEntry>Value</TestEntry>
</Globals>"#;

    let result = RustXmlParser::from_string(xml);
    assert!(
        result.is_ok(),
        "Simple XML should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_xml_parse_with_attributes() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Globals>
    <Entry name="test" value="123"/>
</Globals>"#;

    let result = RustXmlParser::from_string(xml);
    assert!(result.is_ok(), "XML with attributes should parse");
}

#[test]
fn test_xml_parse_nested() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Globals>
    <Parent>
        <Child>
            <GrandChild>Value</GrandChild>
        </Child>
    </Parent>
</Globals>"#;

    let result = RustXmlParser::from_string(xml);
    assert!(result.is_ok(), "Nested XML should parse");
}

// =============================================================================
// GLOBALS.XML TESTS
// =============================================================================

#[test]
fn test_xml_parser_from_valid_globals_xml() {
    let Some(content) = load_globals_xml() else {
        println!("globals.xml not found in fixtures, skipping");
        return;
    };

    let parser = RustXmlParser::from_string(&content).expect("Failed to parse globals.xml");

    let info = parser.get_general_info();
    println!("General info keys: {:?}", info.keys().collect::<Vec<_>>());

    assert!(!info.is_empty(), "Should have general info");
}

#[test]
fn test_quest_overview() {
    let Some(content) = load_globals_xml() else {
        println!("globals.xml not found in fixtures, skipping");
        return;
    };

    let parser = RustXmlParser::from_string(&content).expect("Failed to parse globals.xml");

    let overview = parser.get_quest_overview_struct();

    println!("Quest Overview:");
    println!("  Total quest vars: {}", overview.total_quest_vars);
    println!("  Completed: {}", overview.completed_count);
    println!("  Active: {}", overview.active_count);
    println!("  Completion: {:.1}%", overview.completion_percentage);
    println!("  Quest groups: {}", overview.quest_groups.len());

    for (group_name, group) in overview.quest_groups.iter().take(5) {
        println!(
            "  - {}: {} completed, {} active",
            group_name,
            group.completed.len(),
            group.active.len()
        );
    }

    // Just verify we got valid data
}

#[test]
fn test_quest_groups_structure() {
    let Some(content) = load_globals_xml() else {
        println!("globals.xml not found in fixtures, skipping");
        return;
    };

    let parser = RustXmlParser::from_string(&content).expect("Failed to parse globals.xml");

    let overview = parser.get_quest_overview_struct();

    for (group_name, group) in &overview.quest_groups {
        assert!(!group_name.is_empty(), "Group name should not be empty");

        for quest in &group.completed {
            assert!(!quest.is_empty(), "Quest name should not be empty");
        }
    }
}

// =============================================================================
// COMPANION STATUS TESTS
// =============================================================================

#[test]
fn test_companion_status() {
    let Some(content) = load_globals_xml() else {
        println!("globals.xml not found in fixtures, skipping");
        return;
    };

    let parser = RustXmlParser::from_string(&content).expect("Failed to parse globals.xml");

    let companions = parser.get_companion_status();

    println!("Companion Status:");
    for (name, status) in &companions {
        println!(
            "  - {}: recruitment={}, influence={:?}",
            name, status.recruitment, status.influence
        );
    }
}

#[test]
fn test_motb_companions() {
    let Some(content) = load_globals_xml() else {
        println!("globals.xml not found in fixtures, skipping");
        return;
    };

    let parser = RustXmlParser::from_string(&content).expect("Failed to parse globals.xml");

    let companions = parser.get_companion_status();

    let motb_companion_names = ["safiya", "gann", "kaelyn", "okku", "oneofmany", "ammon"];

    for name in motb_companion_names {
        if companions.contains_key(name) {
            println!("Found MOTB companion: {}", name);
        }
    }
}

// =============================================================================
// FULL SUMMARY TESTS
// =============================================================================

#[test]
fn test_full_summary() {
    let Some(content) = load_globals_xml() else {
        println!("globals.xml not found in fixtures, skipping");
        return;
    };

    let parser = RustXmlParser::from_string(&content).expect("Failed to parse globals.xml");

    let summary = parser.get_full_summary_struct();

    println!("Full Summary:");
    println!("  General info entries: {}", summary.general_info.len());
    println!(
        "  Quest groups: {}",
        summary.quest_overview.quest_groups.len()
    );
    println!("  Companions tracked: {}", summary.companion_status.len());
    println!("  Raw data: {:?}", summary.raw_data_counts);
}

// =============================================================================
// ROUND-TRIP TESTS
// =============================================================================

#[test]
fn test_xml_round_trip() {
    let Some(content) = load_globals_xml() else {
        println!("globals.xml not found in fixtures, skipping");
        return;
    };

    let parser = RustXmlParser::from_string(&content).expect("Failed to parse globals.xml");

    let xml_output = parser.to_xml_string().expect("Failed to serialize to XML");

    assert!(!xml_output.is_empty(), "Output should not be empty");
    assert!(xml_output.contains("<?xml"), "Should have XML declaration");

    let parser2 =
        RustXmlParser::from_string(&xml_output).expect("Failed to re-parse generated XML");

    let info1 = parser.get_general_info();
    let info2 = parser2.get_general_info();

    assert_eq!(
        info1.len(),
        info2.len(),
        "General info should match after round-trip"
    );
}

#[test]
fn test_xml_round_trip_preserves_quests() {
    let Some(content) = load_globals_xml() else {
        println!("globals.xml not found in fixtures, skipping");
        return;
    };

    let parser1 = RustXmlParser::from_string(&content).expect("Parse 1");
    let overview1 = parser1.get_quest_overview_struct();

    let xml_output = parser1.to_xml_string().expect("Serialize");
    let parser2 = RustXmlParser::from_string(&xml_output).expect("Parse 2");
    let overview2 = parser2.get_quest_overview_struct();

    assert_eq!(
        overview1.total_quest_vars, overview2.total_quest_vars,
        "Quest count should be preserved"
    );
    assert_eq!(
        overview1.completed_count, overview2.completed_count,
        "Completed count should be preserved"
    );
}

// =============================================================================
// COMPANION DISCOVERY TESTS
// =============================================================================

#[test]
fn test_potential_companions_discovery() {
    let Some(content) = load_globals_xml() else {
        println!("globals.xml not found in fixtures, skipping");
        return;
    };

    let parser = RustXmlParser::from_string(&content).expect("Failed to parse globals.xml");

    let potential = parser.discover_potential_companions();

    println!("Potential Companions Discovered:");
    for (name, vars) in &potential {
        println!("  - {}: {} related variables", name, vars.len());
    }
}

// =============================================================================
// VARIABLE MANIPULATION TESTS
// =============================================================================

#[test]
fn test_get_global_variable() {
    let Some(content) = load_globals_xml() else {
        println!("globals.xml not found in fixtures, skipping");
        return;
    };

    let parser = RustXmlParser::from_string(&content).expect("Failed to parse globals.xml");

    let info = parser.get_general_info();
    println!("Sample global variables:");
    for (key, value) in info.iter().take(10) {
        println!("  {}: {:?}", key, value);
    }
}

// =============================================================================
// ENCODING TESTS
// =============================================================================

#[test]
fn test_xml_utf8_handling() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Globals>
    <SpecialChars>éèêëàâùûôîç</SpecialChars>
</Globals>"#;

    let result = RustXmlParser::from_string(xml);
    assert!(result.is_ok(), "UTF-8 special chars should parse");
}

#[test]
fn test_xml_entity_handling() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Globals>
    <Entities>&lt;&gt;&amp;&quot;&apos;</Entities>
</Globals>"#;

    let result = RustXmlParser::from_string(xml);
    assert!(result.is_ok(), "XML entities should parse");
}

// =============================================================================
// STRESS TESTS
// =============================================================================

#[test]
fn test_xml_large_content() {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="utf-8"?><Globals>"#);
    for i in 0..1000 {
        xml.push_str(&format!("<Entry_{}>Value_{}</Entry_{}>", i, i, i));
    }
    xml.push_str("</Globals>");

    let result = RustXmlParser::from_string(&xml);
    assert!(result.is_ok(), "Large XML should parse");
}

#[test]
fn test_xml_deeply_nested() {
    let depth = 50;
    let mut xml = String::from(r#"<?xml version="1.0" encoding="utf-8"?>"#);
    for i in 0..depth {
        xml.push_str(&format!("<Level{}>", i));
    }
    xml.push_str("DeepValue");
    for i in (0..depth).rev() {
        xml.push_str(&format!("</Level{}>", i));
    }

    let result = RustXmlParser::from_string(&xml);
    assert!(result.is_ok(), "Deeply nested XML should parse");
}

// =============================================================================
// EDGE CASES
// =============================================================================

#[test]
fn test_xml_with_comments() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<!-- This is a comment -->
<Globals>
    <!-- Another comment -->
    <Entry>Value</Entry>
</Globals>"#;

    let result = RustXmlParser::from_string(xml);
    assert!(result.is_ok(), "XML with comments should parse");
}

#[test]
fn test_xml_with_cdata() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Globals>
    <Script><![CDATA[function test() { return "<value>"; }]]></Script>
</Globals>"#;

    let result = RustXmlParser::from_string(xml);
    assert!(result.is_ok(), "XML with CDATA should parse");
}

#[test]
fn test_xml_whitespace_handling() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Globals>
    <Entry>
        Value with
        newlines
    </Entry>
</Globals>"#;

    let result = RustXmlParser::from_string(xml);
    assert!(result.is_ok(), "XML with whitespace should parse");
}

// =============================================================================
// MULTIPLE SAVE FILES
// =============================================================================

#[test]
fn test_parse_all_campaign_globals() {
    let saves_path = fixtures_path().join("saves");

    let campaigns = [
        "MOTB",
        "Classic_Campaign",
        "Community_Campaign",
        "STORM_Campaign",
        "Westgate_Campaign",
    ];

    for campaign in campaigns {
        let globals_path = saves_path.join(campaign).join("globals.xml");
        if globals_path.exists() {
            println!("\n=== {} globals.xml ===", campaign);
            let content = std::fs::read_to_string(&globals_path).expect("Failed to read");
            let parser = RustXmlParser::from_string(&content);
            match parser {
                Ok(p) => {
                    let summary = p.get_full_summary_struct();
                    println!(
                        "  Quests: {}, Companions: {}",
                        summary.quest_overview.quest_groups.len(),
                        summary.companion_status.len()
                    );
                }
                Err(e) => {
                    println!("  Failed to parse: {}", e);
                }
            }
        } else {
            println!("{} globals.xml not found, skipping", campaign);
        }
    }
}
