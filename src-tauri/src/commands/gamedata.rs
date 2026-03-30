use crate::commands::{CommandError, CommandResult};
use crate::loaders::data_model_loader::DataModelLoader;
use crate::state::AppState;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tauri::State;
use tracing::{debug, info, instrument, warn};

static RE_ALIGNMENT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<b>\s*Alignment:?\s*</b>").expect("Invalid regex"));
static RE_PORTFOLIO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<b>\s*Portfolio:?\s*</b>").expect("Invalid regex"));
static RE_ALIASES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<b>\s*Aliases:?\s*</b>").expect("Invalid regex"));
static RE_WEAPON: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<b>\s*Favored Weapon:?\s*</b>").expect("Invalid regex"));
static RE_STRIP_HTML: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[^>]+>").expect("Invalid regex"));
static RE_NEWLINES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n\s*\n").expect("Invalid regex"));
static RE_TITLE_STRIP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(?:<color=[^>]+>\s*)?<b>\s*[^<]+?\s*</b>(?:</color>)?\s*")
        .expect("Invalid regex")
});

#[tauri::command]
pub async fn get_tlk_string(state: State<'_, AppState>, str_ref: u32) -> CommandResult<String> {
    let game_data = state.game_data.read();
    game_data
        .get_string(str_ref as i32)
        .ok_or_else(|| CommandError::NotFound {
            item: format!("String ref {str_ref}"),
        })
}

#[tauri::command]
pub async fn get_2da_row(
    state: State<'_, AppState>,
    table_name: String,
    row_index: usize,
) -> CommandResult<HashMap<String, String>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table(&table_name)
        .ok_or_else(|| CommandError::NotFound {
            item: format!("Table {table_name}"),
        })?;

    let row = table
        .get_row(row_index)
        .map_err(|_| CommandError::NotFound {
            item: format!("Row {row_index} in table {table_name}"),
        })?;

    let mut result = HashMap::new();
    for (k, v) in &row {
        if let Some(val) = v {
            result.insert(k.clone(), val.clone());
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn list_2da_tables(state: State<'_, AppState>) -> CommandResult<Vec<String>> {
    let game_data = state.game_data.read();
    Ok(game_data
        .table_names()
        .map(std::string::ToString::to_string)
        .collect())
}

#[tauri::command]
pub async fn get_2da_table(
    state: State<'_, AppState>,
    table_name: String,
) -> CommandResult<Vec<HashMap<String, Option<String>>>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table(&table_name.to_lowercase())
        .ok_or_else(|| CommandError::NotFound {
            item: format!("Table: {table_name}"),
        })?;

    let mut rows = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            let standard_map: HashMap<String, Option<String>> = row.into_iter().collect();
            rows.push(standard_map);
        }
    }
    Ok(rows)
}

#[derive(serde::Serialize)]
pub struct AvailableClass {
    pub id: i32,
    pub name: Option<String>,
    pub is_prestige: bool,
    pub hit_die: i32,
}

#[tauri::command]
pub async fn get_available_classes(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AvailableClass>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table("classes")
        .ok_or_else(|| CommandError::NotFound {
            item: "Classes table".to_string(),
        })?;

    let mut classes = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            // Name column contains a TLK string reference — resolve it
            let name = if let Some(Some(strref_str)) = row.get("Name")
                && let Ok(strref) = strref_str.parse::<i32>()
                && let Some(resolved) = game_data.get_string(strref)
            {
                resolved
            } else {
                row.get("Label")
                    .and_then(std::clone::Clone::clone)
                    .unwrap_or_default()
            };

            if name.is_empty() {
                continue;
            }

            let hit_die = row
                .get("HitDie")
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

            if hit_die == 0 {
                continue;
            }

            let is_prestige = row.get("PreReqTable").and_then(|v| v.as_ref()).is_some();
            classes.push(AvailableClass {
                id: i as i32,
                name: Some(name),
                is_prestige,
                hit_die,
            });
        }
    }
    Ok(classes)
}

#[derive(serde::Serialize)]
pub struct AvailableFeat {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<i32>,
}

#[tauri::command]
pub async fn get_available_feats(state: State<'_, AppState>) -> CommandResult<Vec<AvailableFeat>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table("feat")
        .ok_or_else(|| CommandError::NotFound {
            item: "Feat table".to_string(),
        })?;

    let mut feats = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            let name = row.get("label").and_then(std::clone::Clone::clone);
            let description = row
                .get("description")
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok());
            feats.push(AvailableFeat {
                id: i as i32,
                name,
                description,
            });
        }
    }
    Ok(feats)
}

pub use crate::state::app_state::InitStatus;

#[tauri::command]
pub async fn get_initialization_status(state: State<'_, AppState>) -> CommandResult<InitStatus> {
    Ok(state.init_status.read().clone())
}

#[tauri::command]
#[instrument(name = "initialize_game_data_command", skip(state))]
pub async fn initialize_game_data(state: State<'_, AppState>) -> CommandResult<bool> {
    info!("Initialize game data command invoked");

    {
        let game_data = state.game_data.read();
        let table_count = game_data.table_count();
        debug!("Current table count: {}", table_count);

        if table_count > 0 {
            info!(
                "Game data already initialized ({} tables loaded)",
                table_count
            );
            return Ok(true);
        }
    }

    info!("Starting game data initialization");
    update_init_status(&state, "initializing", 0.0, "Starting initialization...");

    update_init_status(
        &state,
        "initializing",
        1.0,
        "Initializing ResourceManager...",
    );
    {
        let mut rm = state.resource_manager.write().await;
        rm.initialize().await.map_err(|e| {
            warn!("ResourceManager initialization failed: {}", e);
            CommandError::OperationFailed {
                operation: "ResourceManager initialization".to_string(),
                reason: e.to_string(),
            }
        })?;
    }
    info!("ResourceManager initialized");

    update_init_status(&state, "initializing", 2.0, "Loading TLK strings...");
    let tlk_parser = {
        let rm = state.resource_manager.read().await;
        rm.get_tlk_parser().ok_or_else(|| CommandError::NotFound {
            item: "TLK parser".to_string(),
        })?
    };

    update_init_status(&state, "initializing", 5.0, "Loading 2DA tables...");
    let mut loader =
        DataModelLoader::with_options(Arc::clone(&state.resource_manager), true, false);

    let status_handle = Arc::clone(&state.init_status);
    loader.set_progress_callback(Box::new(move |msg, progress| {
        *status_handle.write() = InitStatus {
            step: "initializing".to_string(),
            progress,
            message: msg.to_string(),
        };
    }));

    let loaded_data = loader
        .load_game_data(Arc::clone(&tlk_parser))
        .await
        .map_err(|e| {
            warn!("Failed to load game data: {}", e);
            CommandError::OperationFailed {
                operation: "Load game data".to_string(),
                reason: e.to_string(),
            }
        })?;

    update_init_status(&state, "finalizing", 95.0, "Finalizing...");
    {
        let mut game_data = state.game_data.write();
        game_data.tables = loaded_data.tables;
        game_data.strings = tlk_parser;
        game_data.rule_detector = loaded_data.rule_detector;
        game_data.relationships = loaded_data.relationships;
        game_data.priority_tables = loaded_data.priority_tables;
    }

    let table_count = state.game_data.read().table_count();
    info!(
        "Game data initialization completed: {} tables loaded",
        table_count
    );

    update_init_status(&state, "ready", 100.0, "Ready");
    Ok(true)
}

fn update_init_status(state: &AppState, step: &str, progress: f32, message: &str) {
    let mut status = state.init_status.write();
    *status = InitStatus {
        step: step.to_string(),
        progress,
        message: message.to_string(),
    };
}

#[derive(serde::Serialize)]
pub struct AvailableSkill {
    pub id: i32,
    pub name: Option<String>,
    pub key_ability: Option<String>,
}

#[tauri::command]
pub async fn get_available_skills(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AvailableSkill>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table("skills")
        .ok_or_else(|| CommandError::NotFound {
            item: "Skills table".to_string(),
        })?;

    let mut skills = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            let name = row.get("name").and_then(std::clone::Clone::clone);
            let key_ability = row.get("KeyAbility").and_then(std::clone::Clone::clone);
            skills.push(AvailableSkill {
                id: i as i32,
                name,
                key_ability,
            });
        }
    }
    Ok(skills)
}

#[derive(serde::Serialize)]
pub struct AvailableSpell {
    pub id: i32,
    pub name: Option<String>,
    pub school: Option<String>,
}

#[tauri::command]
pub async fn get_available_spells(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AvailableSpell>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table("spells")
        .ok_or_else(|| CommandError::NotFound {
            item: "Spells table".to_string(),
        })?;

    let mut spells = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            let name = row.get("label").and_then(std::clone::Clone::clone);
            let school = row.get("School").and_then(std::clone::Clone::clone);
            spells.push(AvailableSpell {
                id: i as i32,
                name,
                school,
            });
        }
    }
    Ok(spells)
}

#[derive(serde::Serialize)]
pub struct AvailableRace {
    pub id: i32,
    pub name: String,
    pub is_playable: bool,
}

#[tauri::command]
pub async fn get_available_races(state: State<'_, AppState>) -> CommandResult<Vec<AvailableRace>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table("racialtypes")
        .ok_or_else(|| CommandError::NotFound {
            item: "Racialtypes table".to_string(),
        })?;

    let mut races = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            // Name column contains a TLK string reference (integer) — resolve it
            let name = if let Some(Some(strref_str)) = row.get("Name")
                && let Ok(strref) = strref_str.parse::<i32>()
                && let Some(resolved) = game_data.get_string(strref)
            {
                resolved
            } else {
                // Fall back to Label column
                row.get("Label")
                    .and_then(std::clone::Clone::clone)
                    .unwrap_or_default()
            };

            if name.is_empty() {
                continue;
            }

            let is_playable = row
                .get("PlayerRace")
                .and_then(|v| v.as_ref())
                .is_some_and(|s| s == "1");
            races.push(AvailableRace {
                id: i as i32,
                name,
                is_playable,
            });
        }
    }
    Ok(races)
}

#[derive(serde::Serialize)]
pub struct AvailableGender {
    pub id: i32,
    pub name: String,
}

#[tauri::command]
pub async fn get_available_genders(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AvailableGender>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table("gender")
        .ok_or_else(|| CommandError::NotFound {
            item: "Gender table".to_string(),
        })?;

    let mut genders = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            let name_ref = row
                .get("name")
                .and_then(std::clone::Clone::clone)
                .and_then(|s| s.parse::<i32>().ok());
            let name = name_ref
                .and_then(|r| game_data.get_string(r))
                .unwrap_or_else(|| format!("Gender {i}"));
            genders.push(AvailableGender { id: i as i32, name });
        }
    }
    Ok(genders)
}

#[derive(serde::Serialize)]
pub struct AvailableAlignment {
    pub id: i32,
    pub name: String,
    pub law_chaos: i32,
    pub good_evil: i32,
}

#[tauri::command]
pub async fn get_available_alignments(
    _state: State<'_, AppState>,
) -> CommandResult<Vec<AvailableAlignment>> {
    Ok(vec![
        AvailableAlignment {
            id: 0,
            name: "Lawful Good".to_string(),
            law_chaos: 0,
            good_evil: 0,
        },
        AvailableAlignment {
            id: 1,
            name: "Neutral Good".to_string(),
            law_chaos: 50,
            good_evil: 0,
        },
        AvailableAlignment {
            id: 2,
            name: "Chaotic Good".to_string(),
            law_chaos: 100,
            good_evil: 0,
        },
        AvailableAlignment {
            id: 3,
            name: "Lawful Neutral".to_string(),
            law_chaos: 0,
            good_evil: 50,
        },
        AvailableAlignment {
            id: 4,
            name: "True Neutral".to_string(),
            law_chaos: 50,
            good_evil: 50,
        },
        AvailableAlignment {
            id: 5,
            name: "Chaotic Neutral".to_string(),
            law_chaos: 100,
            good_evil: 50,
        },
        AvailableAlignment {
            id: 6,
            name: "Lawful Evil".to_string(),
            law_chaos: 0,
            good_evil: 100,
        },
        AvailableAlignment {
            id: 7,
            name: "Neutral Evil".to_string(),
            law_chaos: 50,
            good_evil: 100,
        },
        AvailableAlignment {
            id: 8,
            name: "Chaotic Evil".to_string(),
            law_chaos: 100,
            good_evil: 100,
        },
    ])
}

#[derive(serde::Serialize)]
pub struct AvailableDeity {
    pub id: i32,
    pub name: String,
    pub alignment: Option<String>,
    pub portfolio: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub aliases: Option<String>,
    pub favored_weapon: Option<String>,
}

#[tauri::command]
pub async fn get_available_deities(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AvailableDeity>> {
    let game_data = state.game_data.read();
    let Some(table) = game_data.get_table("nwn2_deities") else {
        return Ok(Vec::new());
    };

    // Helper for case-insensitive lookup
    let get_str_ci = |row: &ahash::AHashMap<String, Option<String>>, key: &str| -> Option<String> {
        row.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .and_then(|(_, v)| v.clone())
    };

    let mut deities = Vec::new();

    for i in 0..table.row_count() {
        // Use get_row which returns LoaderResult (Result<AHashMap...>)
        // We handle errors gracefully
        let Ok(row) = table.get_row(i) else {
            continue;
        };

        let removed = get_str_ci(&row, "Removed").unwrap_or_else(|| "0".to_string());
        let first = get_str_ci(&row, "FirstName");
        let last = get_str_ci(&row, "LastName");
        let icon = get_str_ci(&row, "IconID").unwrap_or_default();
        let desc_id_str = get_str_ci(&row, "DescID").unwrap_or_default();
        let desc_ref = desc_id_str.parse::<u32>().unwrap_or(0);

        let first_str = first.as_deref().unwrap_or("").trim();
        let last_str = last.as_deref().unwrap_or("").trim();

        if removed == "1" || first_str.eq_ignore_ascii_case("padding") {
            continue;
        }

        let name = if first_str.is_empty() && last_str.is_empty() {
            String::new()
        } else if first_str.is_empty() {
            last_str.to_string()
        } else if last_str.is_empty() {
            first_str.to_string()
        } else {
            format!("{first_str} {last_str}")
        };

        if name.is_empty() {
            continue;
        }

        // Get description from DescID or empty
        let description = if desc_ref > 0 {
            game_data.get_string(desc_ref as i32).unwrap_or_default()
        } else {
            String::new()
        };

        // ... logging ...
        if description.is_empty() && desc_ref > 0 {
            warn!("Deity {name}: Description empty for DescID {desc_ref}");
        }

        // Parse description
        let parsed = parse_deity_description(&description, &name);

        deities.push(AvailableDeity {
            id: i as i32, // Use row index as ID
            name,
            alignment: if parsed.alignment.is_empty() {
                None
            } else {
                Some(parsed.alignment)
            },
            portfolio: if parsed.portfolio.is_empty() {
                None
            } else {
                Some(parsed.portfolio)
            },
            description: if parsed.description.is_empty() {
                None
            } else {
                Some(parsed.description)
            },
            icon: if icon.is_empty() { None } else { Some(icon) },
            aliases: if parsed.aliases.is_empty() {
                None
            } else {
                Some(parsed.aliases)
            },
            favored_weapon: if parsed.favored_weapon.is_empty() {
                None
            } else {
                Some(parsed.favored_weapon)
            },
        });
    }

    info!("Loaded {} deities", deities.len());
    if let Some(first) = deities.first() {
        debug!(
            "First deity: {} (Desc len: {}, Align: {:?})",
            first.name,
            first.description.as_ref().map(|s| s.len()).unwrap_or(0),
            first.alignment
        );
    }

    deities.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(deities)
}

struct ParsedDeityDesc {
    description: String,
    alignment: String,
    portfolio: String,
    aliases: String,
    favored_weapon: String,
}

fn parse_deity_description(raw_desc: &str, _deity_name: &str) -> ParsedDeityDesc {
    let mut data = ParsedDeityDesc {
        description: String::new(),
        alignment: String::new(),
        portfolio: String::new(),
        aliases: String::new(),
        favored_weapon: String::new(),
    };

    if raw_desc.is_empty() {
        return data;
    }

    let mut cleaned = raw_desc.to_string();

    data.alignment = extract_static(&mut cleaned, &RE_ALIGNMENT);
    data.portfolio = extract_static(&mut cleaned, &RE_PORTFOLIO);
    data.aliases = extract_static(&mut cleaned, &RE_ALIASES);
    data.favored_weapon = extract_static(&mut cleaned, &RE_WEAPON);

    // Remove the deity name/title if it appears at the start (often in bold/caps)
    // We use a static regex to strip ANY bolded first line, avoiding dynamic regex compilation overhead.
    // This assumes the first bold element is the title (which holds true for NWN2 descriptions).
    cleaned = RE_TITLE_STRIP.replace(&cleaned, "").to_string();

    // Also strip generic "Name: " prefix if present (rare but possible)
    if cleaned.trim_start().to_lowercase().starts_with("name:")
        && let Some(idx) = cleaned.find(':')
    {
        cleaned.replace_range(0..=idx, "");
    }

    // Remaining is description - strip HTML
    let desc = RE_STRIP_HTML.replace_all(&cleaned, "");
    // Collapse multiple newlines
    let desc = RE_NEWLINES.replace_all(&desc, "\n\n");
    data.description = desc.trim().to_string();

    data
}

// Helper for static regex extraction
fn extract_static(text: &mut String, re: &Regex) -> String {
    if let Some(mat) = re.find(text) {
        let start = mat.end();
        let remaining = &text[start..];

        // Find next <b> tag start, double newline (description separator), or end of string
        let end_by_tag = remaining.find("<b>").unwrap_or(remaining.len());
        let end_by_newline = remaining.find("\n\n").unwrap_or(remaining.len());
        let end = end_by_tag.min(end_by_newline);
        let content = remaining[..end].trim().to_string();

        let range_end = start + end;
        if range_end <= text.len() {
            text.replace_range(mat.start()..range_end, "");
            // Also strip HTML from the extracted content
            return RE_STRIP_HTML.replace_all(&content, "").trim().to_string();
        }
    }
    String::new()
}

#[derive(serde::Serialize)]
pub struct AvailableDomain {
    pub id: i32,
    pub name: String,
    pub granted_feat: Option<i32>,
}

#[tauri::command]
pub async fn get_all_domains(state: State<'_, AppState>) -> CommandResult<Vec<AvailableDomain>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table("domains")
        .ok_or_else(|| CommandError::NotFound {
            item: "Domains table".to_string(),
        })?;

    let mut domains = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            let name_ref = row
                .get("name")
                .or_else(|| row.get("Name"))
                .and_then(std::clone::Clone::clone)
                .and_then(|s| s.parse::<i32>().ok());
            let name = name_ref
                .and_then(|r| game_data.get_string(r))
                .or_else(|| row.get("label").and_then(std::clone::Clone::clone))
                .or_else(|| row.get("Label").and_then(std::clone::Clone::clone))
                .unwrap_or_else(|| format!("Domain {i}"));
            let granted_feat = row
                .get("GrantedFeat")
                .or_else(|| row.get("grantedfeat"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok());
            domains.push(AvailableDomain {
                id: i as i32,
                name,
                granted_feat,
            });
        }
    }
    Ok(domains)
}

#[derive(serde::Serialize)]
pub struct AvailableBackground {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn get_available_backgrounds(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AvailableBackground>> {
    let game_data = state.game_data.read();
    let table = match game_data.get_table("backgrounds") {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    let mut backgrounds = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            let name_ref = row
                .get("name")
                .or_else(|| row.get("Name"))
                .and_then(std::clone::Clone::clone)
                .and_then(|s| s.parse::<i32>().ok());
            let name = name_ref
                .and_then(|r| game_data.get_string(r))
                .or_else(|| row.get("label").and_then(std::clone::Clone::clone))
                .unwrap_or_else(|| format!("Background {i}"));
            let desc_ref = row
                .get("description")
                .or_else(|| row.get("Description"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok());
            let description = desc_ref.and_then(|r| game_data.get_string(r));
            backgrounds.push(AvailableBackground {
                id: i as i32,
                name,
                description,
            });
        }
    }
    Ok(backgrounds)
}

#[derive(serde::Serialize)]
pub struct AvailableAbility {
    pub id: i32,
    pub name: String,
    pub short_name: String,
}

#[tauri::command]
pub async fn get_available_abilities(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AvailableAbility>> {
    let game_data = state.game_data.read();
    let abilities = vec![
        (0, 135, "Strength", "STR"),
        (1, 133, "Dexterity", "DEX"),
        (2, 132, "Constitution", "CON"),
        (3, 134, "Intelligence", "INT"),
        (4, 136, "Wisdom", "WIS"),
        (5, 131, "Charisma", "CHA"),
    ];

    let result = abilities
        .into_iter()
        .map(|(id, strref, default_name, short_name)| {
            let name = game_data
                .get_string(strref)
                .unwrap_or_else(|| default_name.to_string());
            AvailableAbility {
                id,
                name,
                short_name: short_name.to_string(),
            }
        })
        .collect();

    Ok(result)
}

#[derive(serde::Serialize)]
pub struct AvailableBaseItem {
    pub id: i32,
    pub name: String,
    pub item_class: Option<String>,
    pub store_panel: i32,
    pub sub_category: String,
    pub weight: Option<f32>,
    pub base_cost: Option<i32>,
}

#[tauri::command]
pub async fn get_available_base_items(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AvailableBaseItem>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table("baseitems")
        .ok_or_else(|| CommandError::NotFound {
            item: "Baseitems table".to_string(),
        })?;

    let mut items = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            let name_ref = row
                .get("name")
                .or_else(|| row.get("Name"))
                .and_then(std::clone::Clone::clone)
                .and_then(|s| s.parse::<i32>().ok());
            let name = name_ref
                .and_then(|r| game_data.get_string(r))
                .or_else(|| row.get("label").and_then(std::clone::Clone::clone))
                .or_else(|| row.get("Label").and_then(std::clone::Clone::clone))
                .unwrap_or_else(|| format!("Item {i}"));

            let name_lower = name.to_lowercase();
            if name.is_empty()
                || name_lower.contains("bad index")
                || name_lower.contains("****")
                || name_lower.contains("deleted")
                || name_lower.contains("padding")
            {
                continue;
            }

            let item_class = row
                .get("itemclass")
                .or_else(|| row.get("ItemClass"))
                .and_then(std::clone::Clone::clone);
            let store_panel = row
                .get("StorePanel")
                .or_else(|| row.get("storepanel"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(4);
            let label = row
                .get("label")
                .or_else(|| row.get("Label"))
                .and_then(|v| v.as_ref())
                .cloned()
                .unwrap_or_default();
            let sub_category = compute_sub_category(store_panel, &label, item_class.as_deref());
            let weight = row
                .get("weight")
                .or_else(|| row.get("Weight"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<f32>().ok());
            let base_cost = row
                .get("basecost")
                .or_else(|| row.get("BaseCost"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok());
            items.push(AvailableBaseItem {
                id: i as i32,
                name,
                item_class,
                store_panel,
                sub_category,
                weight,
                base_cost,
            });
        }
    }
    Ok(items)
}

#[derive(serde::Serialize)]
pub struct AvailableSpellSchool {
    pub id: i32,
    pub name: String,
    pub opposition_school: Option<i32>,
}

#[tauri::command]
pub async fn get_available_spell_schools(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AvailableSpellSchool>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table("spellschools")
        .ok_or_else(|| CommandError::NotFound {
            item: "Spellschools table".to_string(),
        })?;

    let mut schools = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            let name_ref = row
                .get("stringref")
                .or_else(|| row.get("StringRef"))
                .and_then(std::clone::Clone::clone)
                .and_then(|s| s.parse::<i32>().ok());
            let name = name_ref
                .and_then(|r| game_data.get_string(r))
                .or_else(|| row.get("label").and_then(std::clone::Clone::clone))
                .or_else(|| row.get("Label").and_then(std::clone::Clone::clone))
                .unwrap_or_else(|| format!("School {i}"));
            let opposition_school = row
                .get("oppositionschool")
                .or_else(|| row.get("OppositionSchool"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok());
            schools.push(AvailableSpellSchool {
                id: i as i32,
                name,
                opposition_school,
            });
        }
    }
    Ok(schools)
}

#[derive(serde::Serialize)]
pub struct AvailableItemProperty {
    pub id: i32,
    pub name: String,
    pub cost_table: Option<i32>,
}

#[tauri::command]
pub async fn get_available_item_properties(
    state: State<'_, AppState>,
) -> CommandResult<Vec<AvailableItemProperty>> {
    let game_data = state.game_data.read();
    let table = game_data
        .get_table("itempropdef")
        .ok_or_else(|| CommandError::NotFound {
            item: "Itempropdef table".to_string(),
        })?;

    let mut props = Vec::new();
    for i in 0..table.row_count() {
        if let Some(row) = table.get_by_id(i as i32) {
            let name_ref = row
                .get("name")
                .or_else(|| row.get("Name"))
                .and_then(std::clone::Clone::clone)
                .and_then(|s| s.parse::<i32>().ok());
            let name = name_ref
                .and_then(|r| game_data.get_string(r))
                .or_else(|| row.get("label").and_then(std::clone::Clone::clone))
                .or_else(|| row.get("Label").and_then(std::clone::Clone::clone))
                .unwrap_or_else(|| format!("Property {i}"));
            let cost_table = row
                .get("costtableresref")
                .or_else(|| row.get("CostTableResRef"))
                .and_then(|v| v.as_ref())
                .and_then(|s| s.parse::<i32>().ok());
            props.push(AvailableItemProperty {
                id: i as i32,
                name,
                cost_table,
            });
        }
    }
    Ok(props)
}

pub(super) fn compute_sub_category(
    store_panel: i32,
    label: &str,
    item_class: Option<&str>,
) -> String {
    let label_lower = label.to_lowercase();
    let cls_lower = item_class.map(str::to_lowercase).unwrap_or_default();

    match store_panel {
        0 => {
            if label_lower.contains("helm") {
                "helmets"
            } else if label_lower.contains("shield") || label_lower.contains("tower") {
                "shields"
            } else if label_lower.contains("boot") {
                "boots"
            } else if label_lower.contains("glove")
                || label_lower.contains("gauntlet")
                || label_lower.contains("bracer")
            {
                "gloves"
            } else if label_lower.contains("cloak") {
                "cloaks"
            } else if label_lower.contains("belt") {
                "belts"
            } else if label_lower.contains("robe") || label_lower.contains("cloth") {
                "robes"
            } else {
                "bodyArmor"
            }
        }
        1 => {
            if cls_lower.contains("sword")
                || label_lower.contains("sword")
                || label_lower.contains("rapier")
                || label_lower.contains("kukri")
                || label_lower.contains("katana")
                || label_lower.contains("scimitar")
                || label_lower.contains("falchion")
                || label_lower.contains("kama")
                || label_lower.contains("sickle")
            {
                "swords"
            } else if cls_lower.contains("axe") || label_lower.contains("axe") {
                "axes"
            } else if (cls_lower.contains("bow") || label_lower.contains("bow"))
                && !cls_lower.contains("cross")
                && !label_lower.contains("cross")
            {
                "bows"
            } else if cls_lower.contains("xbow") || label_lower.contains("crossbow") {
                "crossbows"
            } else if cls_lower.contains("dag") || label_lower.contains("dagger") {
                "daggers"
            } else if cls_lower.contains("mace")
                || cls_lower.contains("ham")
                || label_lower.contains("mace")
                || label_lower.contains("hammer")
                || label_lower.contains("morning")
                || label_lower.contains("club")
            {
                "macesAndHammers"
            } else if label_lower.contains("spear")
                || label_lower.contains("halberd")
                || label_lower.contains("trident")
                || label_lower.contains("pike")
            {
                "polearms"
            } else if cls_lower.contains("staf")
                || label_lower.contains("staff")
                || label_lower.contains("quarter")
            {
                "staves"
            } else if label_lower.contains("flail") || label_lower.contains("whip") {
                "flails"
            } else if label_lower.contains("thrown")
                || label_lower.contains("sling")
                || label_lower.contains("dart")
                || label_lower.contains("shuriken")
            {
                "thrown"
            } else if label_lower.contains("arrow")
                || label_lower.contains("bolt")
                || label_lower.contains("bullet")
            {
                "ammunition"
            } else {
                "otherWeapons"
            }
        }
        2 => {
            if label_lower.contains("potion") {
                "potions"
            } else if label_lower.contains("scroll") {
                "scrolls"
            } else if label_lower.contains("wand") || label_lower.contains("rod") {
                "wandsAndRods"
            } else {
                "otherMagic"
            }
        }
        3 => {
            if label_lower.contains("wand") || label_lower.contains("rod") {
                "wandsAndRods"
            } else if label_lower.contains("ring") {
                "rings"
            } else if label_lower.contains("amulet") || label_lower.contains("neck") {
                "amulets"
            } else if label_lower.contains("potion") {
                "potions"
            } else if label_lower.contains("scroll") {
                "scrolls"
            } else {
                "otherMagic"
            }
        }
        4 => {
            if label_lower.contains("ring") {
                "rings"
            } else if label_lower.contains("amulet") || label_lower.contains("neck") {
                "amulets"
            } else if label_lower.contains("gem") || label_lower.contains("jewel") {
                "gems"
            } else if label_lower.contains("trap") || label_lower.contains("kit") {
                "trapsAndKits"
            } else if label_lower.contains("book")
                || label_lower.contains("recipe")
                || label_lower.contains("enchant")
            {
                "books"
            } else if label_lower.contains("container") || label_lower.contains("bag") {
                "containers"
            } else {
                "otherMisc"
            }
        }
        _ => "other",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_deity_description() {
        let name = "Mystra";
        let raw_desc = "<b>Mystra</b>
<b>Alignment:</b> Neutral Good
<b>Portfolio:</b> Magic, spells, the Weave
<b>Aliases:</b> The Lady of Mysteries, The Mother of All Magic
<b>Favored Weapon:</b> Shuriken

Mystra (MISS-trah) is the goddess of magic.
";

        let parsed = parse_deity_description(raw_desc, name);

        assert_eq!(parsed.alignment, "Neutral Good");
        assert_eq!(parsed.portfolio, "Magic, spells, the Weave");
        assert_eq!(
            parsed.aliases,
            "The Lady of Mysteries, The Mother of All Magic"
        );
        assert_eq!(parsed.favored_weapon, "Shuriken");
        assert_eq!(
            parsed.description,
            "Mystra (MISS-trah) is the goddess of magic."
        );
    }

    #[test]
    fn test_parse_deity_description_html_cleanup() {
        let name = "Bane";
        let raw_desc = "<color=red><b>Bane</b></color>
<b>Alignment:</b> Lawful Evil
<b>Portfolio:</b> Hatred, tyranny, fear
<b>Aliases:</b> The Black Hand
<b>Favored Weapon:</b> Morningstar

Bane (BAIN) is the ultimate tyrant.
";

        let parsed = parse_deity_description(raw_desc, name);

        assert_eq!(parsed.alignment, "Lawful Evil");
        assert_eq!(parsed.portfolio, "Hatred, tyranny, fear");
        assert_eq!(parsed.aliases, "The Black Hand");
        assert_eq!(parsed.favored_weapon, "Morningstar");
        assert_eq!(parsed.description, "Bane (BAIN) is the ultimate tyrant.");
    }
}
