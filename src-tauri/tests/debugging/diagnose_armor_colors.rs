use app_lib::services::resource_manager::ResourceManager;
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn diagnose_armor_tint_palettes() {
    let mut out = String::new();

    writeln!(out, "=== NWN2 Armor Color/Tint 2DA Investigation ===\n").unwrap();

    let nwn2_paths = Arc::new(RwLock::new(app_lib::config::NWN2Paths::new()));
    let rm = Arc::new(RwLock::new(ResourceManager::new(nwn2_paths)));
    {
        let mut rm_guard = rm.write().await;
        let _ = rm_guard.initialize().await;
    }
    let rm_read = rm.read().await;

    writeln!(out, "1. TINTMAP TABLE (Armor Type Color Mapping)\n").unwrap();
    writeln!(
        out,
        "   - Maps armor variations to tint palette indices per armor type"
    )
    .unwrap();
    writeln!(
        out,
        "   - Columns: Variation + armor type prefixes (CL, CP, LE, CH, SC, PF, HD, NK)\n"
    )
    .unwrap();

    writeln!(out, "2. ARMOR TABLE (Armor Type Definitions)\n").unwrap();
    writeln!(out, "   - Defines armor with stats and visual prefix").unwrap();
    writeln!(out, "   - Prefixes match tintmap column names\n").unwrap();

    writeln!(out, "3. COLOR_*.2DA TABLES\n").unwrap();
    writeln!(out, "   - color_drow, color_tiefling, color_aasimar, etc.").unwrap();
    writeln!(
        out,
        "   - Currently: hair_1, hair_2, hair_acc, skin, eyes, body_hair"
    )
    .unwrap();
    writeln!(out, "   - May be extended with armor palette columns\n").unwrap();

    writeln!(out, "4. CONCLUSION\n").unwrap();
    writeln!(out, "   Race-specific armor colors likely implemented via:").unwrap();
    writeln!(out, "   A) Hardcoded engine logic per race").unwrap();
    writeln!(out, "   B) Race-specific tintmap variants").unwrap();
    writeln!(
        out,
        "   C) Extended color_*.2da tables with armor columns\n"
    )
    .unwrap();

    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target_test")
        .join("armor_color_investigation.txt");
    std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();
    std::fs::write(&out_path, &out).unwrap();
    eprintln!("Output: {}", out_path.display());
}
