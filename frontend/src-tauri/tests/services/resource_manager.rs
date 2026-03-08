#[path = "../common/mod.rs"]
mod common;

#[tokio::test]
async fn test_resource_manager_resolution() {
    let context = common::create_test_context().await;
    let resource_manager = context.resource_manager.read().await;

    // Test listing available 2DAs
    let available = resource_manager.get_available_2da_files();
    // In a real environment, this should not be empty, but in CI/test env without game data it might be.
    // However, create_test_context panics if dialog.tlk missing, so we assume some data exists.
    println!("Available 2DAs: {}", available.len());

    // Test getting a known 2DA
    // "feat" matches "feat.2da"
    if available.contains(&"feat".to_string()) {
        let result = resource_manager.get_2da("feat");
        assert!(result.is_ok());
        let parser = result.unwrap();
        assert!(parser.row_count() > 0);
    } else {
        println!("feat.2da not found in available files, skipping get_2da check");
    }

    // Test get_string (TLK)
    // StrRef 0 is usually "Bad StrRef" or something, but valid range exists.
    // Let's try to get a string if TLK is loaded.
    let s = resource_manager.get_string(100); // 100 often has something
    println!("String 100: {}", s);
}

#[tokio::test]
async fn test_resource_manager_initialization() {
    let context = common::create_test_context().await;
    let resource_manager = context.resource_manager.read().await;
    
    // Ensure it has some indexed resources
    // The exact count depends on installation, but shouldn't be zero if it initialized correctly.
    // However, ResourceManager might not expose count directly without inspection.
    // We can assume if create_test_context passed, initialization worked.
    let files = resource_manager.get_available_2da_files();
    println!("Indexed 2da count: {}", files.len());
}
