use axum::{
    Router,
    extract::{Json, State},
    routing::post,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::loaders::types::GameData;
use crate::state::session_state::SessionState;

#[derive(Clone)]
pub struct McpState {
    pub game_data: Arc<RwLock<GameData>>,
    pub session: Arc<RwLock<SessionState>>,
}

#[derive(Debug, Deserialize)]
pub struct McpToolRequest {
    pub tool_name: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct McpToolResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub async fn start(state: McpState, port: u16) {
    let app = Router::new()
        .route("/mcp", post(handle_mcp_request))
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind MCP local port");

    tracing::info!("Started Local MCP Bridge Server on {}", addr);
    axum::serve(listener, app)
        .await
        .expect("Failed to serve local MCP Bridge");
}

async fn handle_mcp_request(
    State(state): State<McpState>,
    Json(payload): Json<McpToolRequest>,
) -> Json<McpToolResponse> {
    let result = match payload.tool_name.as_str() {
        "query_2da" => query_2da(state, payload.params),
        "get_tlk_string" => query_tlk(state, payload.params),
        "query_character" => query_character(state, payload.params),
        "get_overview" => get_overview(state, payload.params),
        "get_classes" => get_classes(state, payload.params),
        "get_feats" => get_feats(state, payload.params),
        "get_spells" => get_spells(state, payload.params),
        "get_skills" => get_skills(state, payload.params),
        "get_abilities" => get_abilities(state, payload.params),
        "get_saves" => get_saves(state, payload.params),
        "get_inventory" => get_inventory(state, payload.params),
        "get_campaign_info" => get_campaign_info(state, payload.params),
        "search_2da" => search_2da(state, payload.params),
        _ => Err(anyhow::anyhow!("Unknown tool: {}", payload.tool_name)),
    };

    match result {
        Ok(res) => Json(McpToolResponse {
            success: true,
            result: Some(res),
            error: None,
        }),
        Err(e) => Json(McpToolResponse {
            success: false,
            result: None,
            error: Some(e.to_string()),
        }),
    }
}

fn query_2da(state: McpState, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let table = params
        .get("table")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: table"))?;

    let row_id = params
        .get("row_id")
        .and_then(|v| v.as_i64())
        .map(|v| v as usize)
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: row_id"))?;

    let game_data_lock = state.game_data.read();
    if game_data_lock.table_count() == 0 {
        return Err(anyhow::anyhow!(
            "Game Data not initialized. Run the editor and load a character first."
        ));
    }

    let table_ref = game_data_lock
        .get_table(table)
        .ok_or_else(|| anyhow::anyhow!("Table {} not found", table))?;

    match table_ref.get_row(row_id) {
        Ok(row) => {
            let mut tree = std::collections::BTreeMap::new();
            for (k, v) in row {
                tree.insert(k, v);
            }
            Ok(serde_json::to_value(tree)?)
        }
        Err(e) => Err(anyhow::anyhow!("Failed to retrieve 2DA row: {}", e)),
    }
}

fn query_tlk(state: McpState, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let strref = params
        .get("strref")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: strref"))?;

    let game_data_lock = state.game_data.read();
    if game_data_lock.table_count() == 0 {
        return Err(anyhow::anyhow!("Game Data not initialized."));
    }

    if let Some(text) = game_data_lock.get_string(strref) {
        Ok(serde_json::json!({ "text": text }))
    } else {
        Err(anyhow::anyhow!("StrRef {} not found", strref))
    }
}

fn query_character(
    state: McpState,
    _params: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    let session_lock = state.session.read();

    if let Some(char_ref) = session_lock.character() {
        // We will output the basic character dump to start with
        Ok(serde_json::json!({
            "name": char_ref.full_name(),
            "level": char_ref.total_level(),
            "experience": char_ref.experience(),
            "classes": char_ref.class_entries().iter().map(|c| c.class_id.0).collect::<Vec<_>>(),
        }))
    } else {
        Err(anyhow::anyhow!(
            "No character is currently loaded in the session. Please open a character in the editor UI first."
        ))
    }
}

fn get_overview(state: McpState, _params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let session_lock = state.session.read();
    let game_data_lock = state.game_data.read();
    if let Some(char_ref) = session_lock.character() {
        let summary = char_ref.get_overview_state(&game_data_lock);
        Ok(serde_json::to_value(summary)?)
    } else {
        Err(anyhow::anyhow!("No character is currently loaded."))
    }
}

fn get_classes(state: McpState, _params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let session_lock = state.session.read();
    let game_data_lock = state.game_data.read();
    if let Some(char_ref) = session_lock.character() {
        let summary = char_ref.get_classes_state(&game_data_lock);
        Ok(serde_json::to_value(summary)?)
    } else {
        Err(anyhow::anyhow!("No character is currently loaded."))
    }
}

fn get_feats(state: McpState, _params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let session_lock = state.session.read();
    let game_data_lock = state.game_data.read();
    if let Some(char_ref) = session_lock.character() {
        let summary = char_ref.get_feats_state(&game_data_lock);
        Ok(serde_json::to_value(summary)?)
    } else {
        Err(anyhow::anyhow!("No character is currently loaded."))
    }
}

fn get_spells(state: McpState, _params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let session_lock = state.session.read();
    let game_data_lock = state.game_data.read();
    if let Some(char_ref) = session_lock.character() {
        let summary = char_ref.get_spells_state(&game_data_lock);
        Ok(serde_json::to_value(summary)?)
    } else {
        Err(anyhow::anyhow!("No character is currently loaded."))
    }
}

fn get_skills(state: McpState, _params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let session_lock = state.session.read();
    let game_data_lock = state.game_data.read();
    if let Some(char_ref) = session_lock.character() {
        // Option 1: decoder could be extracted
        let decoder = &session_lock.item_property_decoder;
        let summary = char_ref.get_skill_summary(&game_data_lock, Some(decoder));
        Ok(serde_json::to_value(summary)?)
    } else {
        Err(anyhow::anyhow!("No character is currently loaded."))
    }
}

fn get_abilities(state: McpState, _params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let session_lock = state.session.read();
    let game_data_lock = state.game_data.read();
    if let Some(char_ref) = session_lock.character() {
        let decoder = &session_lock.item_property_decoder;
        let summary = char_ref.get_abilities_state(&game_data_lock, decoder);
        Ok(serde_json::to_value(summary)?)
    } else {
        Err(anyhow::anyhow!("No character is currently loaded."))
    }
}

fn get_saves(state: McpState, _params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let session_lock = state.session.read();
    let game_data_lock = state.game_data.read();
    if let Some(char_ref) = session_lock.character() {
        let decoder = &session_lock.item_property_decoder;
        let summary = char_ref.get_save_summary(&game_data_lock, decoder);
        Ok(serde_json::to_value(summary)?)
    } else {
        Err(anyhow::anyhow!("No character is currently loaded."))
    }
}

fn get_inventory(state: McpState, _params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let session_lock = state.session.read();
    let game_data_lock = state.game_data.read();
    if let Some(char_ref) = session_lock.character() {
        let decoder = &session_lock.item_property_decoder;
        let summary = char_ref.get_full_inventory_summary(&game_data_lock, decoder);
        let mut json_val = serde_json::to_value(summary)?;
        if let Some(obj) = json_val.as_object_mut() {
            if let Some(inv) = obj.get_mut("inventory").and_then(|v| v.as_array_mut()) {
                for item in inv {
                    if let Some(item_obj) = item.as_object_mut() {
                        item_obj.remove("item");
                    }
                }
            }
            if let Some(equipped) = obj.get_mut("equipped").and_then(|v| v.as_array_mut()) {
                for item in equipped {
                    if let Some(item_obj) = item.as_object_mut() {
                        item_obj.remove("item_data");
                    }
                }
            }
        }
        Ok(json_val)
    } else {
        Err(anyhow::anyhow!("No character is currently loaded."))
    }
}

fn get_campaign_info(
    state: McpState,
    _params: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    let session_lock = state.session.read();
    if let Some(handler) = session_lock.savegame_handler.as_ref() {
        let summary = crate::services::campaign::CampaignManager::get_summary(handler)
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(serde_json::to_value(summary)?)
    } else {
        Err(anyhow::anyhow!("No savegame is currently loaded."))
    }
}

fn search_2da(state: McpState, params: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let table = params
        .get("table")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: table"))?;

    let query = params
        .get("query")
        .and_then(|v| v.as_str())
        .map(|s| s.to_lowercase())
        .ok_or_else(|| anyhow::anyhow!("Missing parameter: query"))?;

    let game_data_lock = state.game_data.read();
    if game_data_lock.table_count() == 0 {
        return Err(anyhow::anyhow!(
            "Game Data not initialized. Run the editor and load a character first."
        ));
    }

    let table_ref = game_data_lock
        .get_table(table)
        .ok_or_else(|| anyhow::anyhow!("Table {} not found", table))?;

    let mut results = Vec::new();
    for row_id in 0..table_ref.row_count() {
        if let Ok(row) = table_ref.get_row(row_id) {
            let mut matches = false;
            for val in row.values() {
                if let Some(s) = val {
                    if s.to_lowercase().contains(&query) {
                        matches = true;
                        break;
                    }
                }
            }
            if matches {
                let mut tree = std::collections::BTreeMap::new();
                for (k, v) in row {
                    if let Some(s) = v {
                        tree.insert(k.clone(), s.clone());
                    } else {
                        tree.insert(k.clone(), "".to_string());
                    }
                }
                tree.insert("id".to_string(), row_id.to_string());
                results.push(tree);
                if results.len() >= 50 {
                    break; // Cap at 50 results
                }
            }
        }
    }

    Ok(serde_json::to_value(results)?)
}
