use crate::character::{
    AbilityIndex, AbilityPointsSummary, AbilityScores, Alignment, HitPoints, RaceId,
};
use crate::commands::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceLimits {
    pub light: i32,
    pub medium: i32,
    pub heavy: i32,
}

impl EncumbranceLimits {
    pub fn new(light: i32, medium: i32, heavy: i32) -> Self {
        Self {
            light,
            medium,
            heavy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Biography {
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub age: i32,
    pub description: String,
    pub background: Option<String>,
    pub experience: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceChangedEvent {
    pub old_race_id: Option<i32>,
    pub new_race_id: i32,
    pub old_subrace: Option<String>,
    pub new_subrace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// Identity Commands
#[tauri::command]
pub async fn get_character_name(state: State<'_, AppState>) -> CommandResult<String> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.full_name())
}

#[tauri::command]
pub async fn get_first_name(state: State<'_, AppState>) -> CommandResult<String> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.first_name())
}

#[tauri::command]
pub async fn get_last_name(state: State<'_, AppState>) -> CommandResult<String> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.last_name())
}

#[tauri::command]
pub async fn set_first_name(state: State<'_, AppState>, name: String) -> CommandResult<String> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character.set_first_name(name);
    Ok(character.first_name())
}

#[tauri::command]
pub async fn set_last_name(state: State<'_, AppState>, name: String) -> CommandResult<String> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character.set_last_name(name);
    Ok(character.last_name())
}

#[tauri::command]
pub async fn get_character_age(state: State<'_, AppState>) -> CommandResult<i32> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.age())
}

#[tauri::command]
pub async fn set_character_age(state: State<'_, AppState>, age: i32) -> CommandResult<i32> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character
        .set_age(age)
        .map_err(|e| CommandError::ValidationError {
            field: "age".to_string(),
            reason: e.to_string(),
        })?;
    Ok(character.age())
}

#[tauri::command]
pub async fn get_experience_points(state: State<'_, AppState>) -> CommandResult<i32> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.experience())
}

#[tauri::command]
pub async fn set_experience_points(state: State<'_, AppState>, xp: i32) -> CommandResult<i32> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character
        .set_experience(xp)
        .map_err(|e| CommandError::ValidationError {
            field: "experience".to_string(),
            reason: e.to_string(),
        })?;
    Ok(character.experience())
}

#[tauri::command]
pub async fn get_alignment(state: State<'_, AppState>) -> CommandResult<Alignment> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.alignment())
}

#[tauri::command]
pub async fn set_alignment(
    state: State<'_, AppState>,
    law_chaos: Option<i32>,
    good_evil: Option<i32>,
) -> CommandResult<Alignment> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character
        .set_alignment(law_chaos, good_evil)
        .map_err(|e| CommandError::ValidationError {
            field: "alignment".to_string(),
            reason: e.to_string(),
        })?;
    Ok(character.alignment())
}

#[tauri::command]
pub async fn get_deity(state: State<'_, AppState>) -> CommandResult<String> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.deity())
}

#[tauri::command]
pub async fn set_deity(state: State<'_, AppState>, deity: String) -> CommandResult<String> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character.set_deity(deity);
    Ok(character.deity())
}

#[tauri::command]
pub async fn get_biography(state: State<'_, AppState>) -> CommandResult<Biography> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(Biography {
        first_name: character.first_name(),
        last_name: character.last_name(),
        full_name: character.full_name(),
        age: character.age(),
        description: character.description(),
        background: character.background(&game_data),
        experience: character.experience(),
    })
}

#[tauri::command]
pub async fn set_biography(
    state: State<'_, AppState>,
    description: String,
) -> CommandResult<String> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character.set_description(description);
    Ok(character.description())
}

#[tauri::command]
pub async fn get_background(state: State<'_, AppState>) -> CommandResult<Option<String>> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.background(&game_data))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[tauri::command]
pub async fn get_domains(state: State<'_, AppState>) -> CommandResult<Vec<DomainInfo>> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    let domain_ids = character.domains();

    let mut domains = Vec::new();
    if let Some(domains_table) = game_data.get_table("domains") {
        for domain_id in domain_ids {
            if let Some(row) = domains_table.get_by_id(domain_id.0) {
                let name_strref = row
                    .get("Name")
                    .and_then(|v| v.as_ref())
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(-1);
                let name = game_data.get_string(name_strref).unwrap_or_default();

                let description_strref = row
                    .get("Description")
                    .and_then(|v| v.as_ref())
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(-1);
                let description = game_data.get_string(description_strref);

                let icon = row.get("Icon").and_then(|v| v.as_ref()).cloned();

                domains.push(DomainInfo {
                    id: domain_id.0,
                    name,
                    description,
                    icon,
                });
            }
        }
    }

    Ok(domains)
}

// Ability Commands
#[tauri::command]
pub async fn get_ability_scores(
    state: State<'_, AppState>,
    _include_racial: Option<bool>,
    _include_equipment: Option<bool>,
) -> CommandResult<AbilityScores> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_effective_abilities(&game_data))
}

#[tauri::command]
pub async fn get_base_ability_scores(state: State<'_, AppState>) -> CommandResult<AbilityScores> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.base_scores())
}

#[tauri::command]
pub async fn get_full_name(state: State<'_, AppState>) -> CommandResult<String> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.full_name())
}

#[tauri::command]
pub async fn set_attribute(
    state: State<'_, AppState>,
    ability: AbilityIndex,
    value: i32,
) -> CommandResult<()> {
    let game_data = state.game_data.read();
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character
        .set_ability_with_cascades(ability, value, &game_data)
        .map_err(|e| CommandError::ValidationError {
            field: format!("{ability:?}"),
            reason: e.to_string(),
        })?;
    Ok(())
}

#[tauri::command]
pub async fn update_hit_points(
    state: State<'_, AppState>,
    current: Option<i32>,
    max: Option<i32>,
) -> CommandResult<HitPoints> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    if let Some(hp) = current {
        character.set_current_hp(hp);
    }
    if let Some(hp) = max {
        character.set_max_hp(hp);
    }
    Ok(character.hit_points())
}

#[tauri::command]
pub async fn set_all_ability_scores(
    state: State<'_, AppState>,
    scores: AbilityScores,
) -> CommandResult<()> {
    let game_data = state.game_data.read();
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character
        .set_ability_with_cascades(AbilityIndex::STR, scores.str_, &game_data)
        .map_err(|e| CommandError::ValidationError {
            field: "STR".to_string(),
            reason: e.to_string(),
        })?;
    character
        .set_ability_with_cascades(AbilityIndex::DEX, scores.dex, &game_data)
        .map_err(|e| CommandError::ValidationError {
            field: "DEX".to_string(),
            reason: e.to_string(),
        })?;
    character
        .set_ability_with_cascades(AbilityIndex::CON, scores.con, &game_data)
        .map_err(|e| CommandError::ValidationError {
            field: "CON".to_string(),
            reason: e.to_string(),
        })?;
    character
        .set_ability_with_cascades(AbilityIndex::INT, scores.int, &game_data)
        .map_err(|e| CommandError::ValidationError {
            field: "INT".to_string(),
            reason: e.to_string(),
        })?;
    character
        .set_ability_with_cascades(AbilityIndex::WIS, scores.wis, &game_data)
        .map_err(|e| CommandError::ValidationError {
            field: "WIS".to_string(),
            reason: e.to_string(),
        })?;
    character
        .set_ability_with_cascades(AbilityIndex::CHA, scores.cha, &game_data)
        .map_err(|e| CommandError::ValidationError {
            field: "CHA".to_string(),
            reason: e.to_string(),
        })?;
    Ok(())
}

#[tauri::command]
pub async fn get_hit_points(state: State<'_, AppState>) -> CommandResult<HitPoints> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.hit_points())
}

#[tauri::command]
pub async fn get_encumbrance_limits(
    state: State<'_, AppState>,
) -> CommandResult<EncumbranceLimits> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    let info = character.calculate_encumbrance(&game_data);
    Ok(EncumbranceLimits::new(
        info.light_limit as i32,
        info.medium_limit as i32,
        info.heavy_limit as i32,
    ))
}

#[tauri::command]
pub async fn get_ability_points_summary(
    state: State<'_, AppState>,
) -> CommandResult<AbilityPointsSummary> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_ability_points_summary())
}

// Race Commands
#[tauri::command]
pub async fn get_race_id(state: State<'_, AppState>) -> CommandResult<i32> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.race_id().0)
}

#[tauri::command]
pub async fn get_race_name(state: State<'_, AppState>) -> CommandResult<String> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.race_name(&game_data))
}

#[tauri::command]
pub async fn get_subrace(state: State<'_, AppState>) -> CommandResult<Option<String>> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.subrace())
}

#[tauri::command]
pub async fn set_race(state: State<'_, AppState>, race_id: i32) -> CommandResult<()> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character.set_race(RaceId(race_id));
    Ok(())
}

#[tauri::command]
pub async fn set_subrace(state: State<'_, AppState>, subrace: Option<String>) -> CommandResult<()> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;
    character.set_subrace(subrace);
    Ok(())
}

#[tauri::command]
pub async fn get_available_subraces(
    state: State<'_, AppState>,
    _race_id: i32,
) -> CommandResult<Vec<crate::character::SubraceInfo>> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.available_subraces(&game_data))
}

#[tauri::command]
pub async fn get_ability_modifiers(
    state: State<'_, AppState>,
) -> CommandResult<crate::character::AbilityModifiers> {
    let session = state.session.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.ability_modifiers())
}

#[tauri::command]
pub async fn get_racial_modifiers(
    state: State<'_, AppState>,
) -> CommandResult<crate::character::AbilityModifiers> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.get_racial_ability_modifiers(&game_data))
}

#[tauri::command]
pub async fn change_race(
    state: State<'_, AppState>,
    race_id: i32,
    subrace: Option<String>,
) -> CommandResult<RaceChangedEvent> {
    let mut session = state.session.write();
    let character = session
        .character
        .as_mut()
        .ok_or(CommandError::NoCharacterLoaded)?;

    let old_race_id = character.race_id();
    let old_subrace = character.subrace();

    character.set_race(RaceId(race_id));
    character.set_subrace(subrace.clone());

    Ok(RaceChangedEvent {
        old_race_id: Some(old_race_id.0),
        new_race_id: race_id,
        old_subrace,
        new_subrace: subrace,
    })
}

#[tauri::command]
pub async fn validate_character(state: State<'_, AppState>) -> CommandResult<CharacterValidation> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session
        .character
        .as_ref()
        .ok_or(CommandError::NoCharacterLoaded)?;
    let result = character.validate(&game_data);

    Ok(CharacterValidation {
        valid: result.valid,
        errors: result.errors.clone(),
        warnings: result.warnings.clone(),
    })
}
