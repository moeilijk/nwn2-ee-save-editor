use crate::state::AppState;
use std::fmt::Write;
use tauri::State;

#[tauri::command]
pub async fn debug_deities(state: State<'_, AppState>) -> Result<String, String> {
    let mut output = String::new();
    let game_data = state.game_data.read();

    output.push_str("=== Tables containing 'deit' ===\n");
    for name in game_data.table_names() {
        if name.to_lowercase().contains("deit") {
            let _ = writeln!(output, "  Found: {name}");
        }
    }

    output.push_str("\n=== Trying to get nwn2_deities table ===\n");
    if let Some(table) = game_data.get_table("nwn2_deities") {
        let _ = writeln!(output, "  Table found! Row count: {}", table.row_count());

        output.push_str("\n=== First 3 rows ===\n");
        for i in 0..3.min(table.row_count()) {
            if let Some(row) = table.get_by_id(i as i32) {
                let _ = writeln!(output, "\nRow {i}:");
                for (key, value) in &row {
                    let _ = writeln!(output, "  {key}: {value:?}");
                }
            }
        }
    } else {
        output.push_str("  Table NOT found!\n");

        output.push_str("\n=== Trying variations ===\n");
        for variant in ["nwn2_deities", "NWN2_Deities", "nwn2_DEITIES"] {
            let found = game_data.get_table(variant).is_some();
            let _ = writeln!(output, "  '{variant}': {}", if found { "FOUND" } else { "not found" });
        }
    }

    let _ = writeln!(output, "\n=== Total tables loaded: {} ===", game_data.table_count());

    Ok(output)
}
