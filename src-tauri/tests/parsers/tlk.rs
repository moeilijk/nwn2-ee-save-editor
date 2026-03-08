use super::super::common::create_test_context;

// =============================================================================
// BASIC TLK LOOKUP TESTS
// =============================================================================

#[tokio::test]
async fn test_tlk_lookup_strref_0() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    let str_0 = rm.get_string(0);
    println!("StrRef 0: {}", str_0);
    assert!(!str_0.is_empty(), "StrRef 0 should not be empty");
}

#[tokio::test]
async fn test_tlk_invalid_strref() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    let bad_str = rm.get_string(16777216);
    assert!(
        bad_str.contains("StrRef:") || bad_str.is_empty(),
        "Should handle invalid StrRef gracefully"
    );
}

// =============================================================================
// DYNAMIC STRING LOOKUPS (using actual game data)
// =============================================================================

#[tokio::test]
async fn test_tlk_strings_exist() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    let mut found_count = 0;
    for i in 0..100i32 {
        let s = rm.get_string(i);
        if !s.is_empty() && !s.contains("StrRef:") {
            found_count += 1;
        }
    }

    println!("Found {} valid strings in first 100 StrRefs", found_count);
    assert!(found_count > 0, "Should find some valid strings");
}

#[tokio::test]
async fn test_tlk_range_lookup() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    println!("\n=== First 20 StrRefs ===");
    for i in 0..20i32 {
        let s = rm.get_string(i);
        if !s.is_empty() && !s.contains("StrRef:") {
            println!("  {}: {}", i, &s.chars().take(50).collect::<String>());
        }
    }
}

#[tokio::test]
async fn test_tlk_high_strrefs() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    println!("\n=== High StrRefs (100000+) ===");
    for i in [100000i32, 110000, 111000, 111100, 111111] {
        let s = rm.get_string(i);
        let display_str = if s.contains("StrRef:") {
            "(not found)".to_string()
        } else {
            s.chars().take(50).collect::<String>()
        };
        println!("  {}: {}", i, display_str);
    }
}

// =============================================================================
// STRING CONSISTENCY
// =============================================================================

#[tokio::test]
async fn test_tlk_consistency() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    let s1 = rm.get_string(1);
    let s2 = rm.get_string(1);
    assert_eq!(s1, s2, "Same strref should return same string");
}

#[tokio::test]
async fn test_tlk_boundary_strrefs() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    let s1 = rm.get_string(1);
    println!("StrRef 1: {}", &s1.chars().take(100).collect::<String>());

    let large = rm.get_string(i32::MAX);
    assert!(
        large.contains("StrRef:") || large.is_empty(),
        "Max i32 strref should be handled"
    );
}

// =============================================================================
// STRING LENGTH TESTS
// =============================================================================

#[tokio::test]
async fn test_tlk_varied_string_lengths() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    let mut short_count = 0;
    let mut medium_count = 0;
    let mut long_count = 0;

    for i in 0..500i32 {
        let s = rm.get_string(i);
        if !s.is_empty() && !s.contains("StrRef:") {
            let len = s.len();
            if len < 20 {
                short_count += 1;
            } else if len < 100 {
                medium_count += 1;
            } else {
                long_count += 1;
            }
        }
    }

    println!("String lengths in first 500 StrRefs:");
    println!("  Short (<20 chars): {}", short_count);
    println!("  Medium (20-99 chars): {}", medium_count);
    println!("  Long (100+ chars): {}", long_count);

    assert!(short_count > 0 || medium_count > 0 || long_count > 0, "Should find some strings");
}

// =============================================================================
// BATCH ACCESS TESTS
// =============================================================================

#[tokio::test]
async fn test_tlk_sequential_access() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    let start = std::time::Instant::now();
    let mut count = 0;

    for i in 0..1000i32 {
        let s = rm.get_string(i);
        if !s.is_empty() && !s.contains("StrRef:") {
            count += 1;
        }
    }

    let elapsed = start.elapsed();
    println!("Read {} strings in {:?}", count, elapsed);
    assert!(elapsed.as_millis() < 5000, "Sequential access should be fast");
}

#[tokio::test]
async fn test_tlk_random_access() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    let strrefs = [1, 100, 500, 1000, 5000, 10000, 50000];

    for strref in strrefs {
        let s = rm.get_string(strref);
        let display_str = if s.contains("StrRef:") {
            "(not found)".to_string()
        } else {
            s.chars().take(30).collect::<String>()
        };
        println!("StrRef {}: {}", strref, display_str);
    }
}

// =============================================================================
// EDGE CASES
// =============================================================================

#[tokio::test]
async fn test_tlk_negative_strref() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    let s = rm.get_string(-1);
    println!("StrRef -1: {}", s);
}

#[tokio::test]
async fn test_tlk_zero_strref() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;

    let s = rm.get_string(0);
    println!("StrRef 0: {}", &s.chars().take(100).collect::<String>());
    assert!(!s.is_empty(), "StrRef 0 should have content");
}
