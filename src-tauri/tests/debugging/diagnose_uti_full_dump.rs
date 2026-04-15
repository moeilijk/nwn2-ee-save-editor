use app_lib::services::resource_manager::ResourceManager;
use std::fmt::Write;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn dump_palette_tables() {
    let nwn2_paths = Arc::new(RwLock::new(app_lib::config::NWN2Paths::new()));
    let rm = Arc::new(RwLock::new(ResourceManager::new(nwn2_paths)));
    {
        let mut rm_guard = rm.write().await;
        let _ = rm_guard.initialize().await;
    }
    let rm = rm.read().await;

    let mut out = String::new();

    // Search for 2DA files with color/tint/palette in name
    writeln!(out, "=== 2DA resources matching color/tint/palette ===").unwrap();
    for prefix in [
        "color",
        "tint",
        "pal_",
        "palette",
        "iprp_color",
        "armorvisual",
    ] {
        let found = rm.list_resources_by_prefix(prefix, "2da");
        if !found.is_empty() {
            writeln!(out, "  {prefix}*: {:?}", found).unwrap();
        }
    }

    // Dump raw content of key tables (first 20 lines)
    let tables_to_dump = [
        "tintmap",
        "iprp_color",
        "color_drow",
        "color_human",
        "armorvisualdata",
        "racialsubtypes",
    ];

    for table in &tables_to_dump {
        writeln!(out, "\n\n=== {table}.2da ===").unwrap();
        match rm.get_resource_bytes(table, "2da") {
            Ok(bytes) => {
                let content = String::from_utf8_lossy(&bytes);
                for (i, line) in content.lines().take(25).enumerate() {
                    writeln!(out, "  {i:3}: {line}").unwrap();
                }
                let total = content.lines().count();
                if total > 25 {
                    writeln!(out, "  ... ({total} total lines)").unwrap();
                }
            }
            Err(e) => writeln!(out, "  NOT FOUND: {e}").unwrap(),
        }
    }

    // Specifically look at tintmap row for Variation=4 (Body05, chain armor)
    writeln!(out, "\n\n=== tintmap.2da - full dump ===").unwrap();
    if let Ok(bytes) = rm.get_resource_bytes("tintmap", "2da") {
        let content = String::from_utf8_lossy(&bytes);
        for (i, line) in content.lines().enumerate() {
            writeln!(out, "  {i:3}: {line}").unwrap();
        }
    }

    let out_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target_test")
        .join("palette_tables.txt");
    std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();
    std::fs::write(&out_path, &out).unwrap();
    eprintln!("Output: {}", out_path.display());
}
