use tauri::State;

use crate::character::{
    AbilitiesState, ClassesState, FeatsState, OverviewState, SpellsState,
};
use crate::commands::{CommandError, CommandResult};
use crate::state::AppState;

use super::types::{AbilitiesUpdates, CharacterUpdates};

#[tauri::command]
pub async fn get_overview_state(state: State<'_, AppState>) -> CommandResult<OverviewState> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let game_data = state.game_data.read();

    Ok(character.get_overview_state(&game_data))
}

#[tauri::command]
pub async fn update_character(
    state: State<'_, AppState>,
    updates: CharacterUpdates,
) -> CommandResult<OverviewState> {
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;

    if let Some(first_name) = updates.first_name {
        character.set_first_name(first_name);
    }
    if let Some(last_name) = updates.last_name {
        character.set_last_name(last_name);
    }
    if let Some(age) = updates.age {
        character.set_age(age)?;
    }
    if let Some(deity) = updates.deity {
        character.set_deity(deity);
    }
    if let Some(description) = updates.description {
        character.set_description(description);
    }
    if let Some((law_chaos, good_evil)) = updates.alignment {
        character
            .set_alignment(Some(law_chaos), Some(good_evil))?;
    }
    if let Some(experience) = updates.experience {
        character.set_experience(experience)?;
    }

    let game_data = state.game_data.read();
    Ok(character.get_overview_state(&game_data))
}

#[tauri::command]
pub async fn get_abilities_state(state: State<'_, AppState>) -> CommandResult<AbilitiesState> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let game_data = state.game_data.read();
    let decoder = &session.item_property_decoder;

    Ok(character.get_abilities_state(&game_data, decoder))
}

#[tauri::command]
pub async fn update_abilities(
    state: State<'_, AppState>,
    updates: AbilitiesUpdates,
) -> CommandResult<AbilitiesState> {
    let game_data = state.game_data.read();

    {
        let mut session = state.session.write();
        let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;

        if let Some(str_val) = updates.str_ {
            character.set_ability_with_cascades(
                crate::character::types::AbilityIndex::STR,
                str_val,
                &game_data,
            )?;
        }
        if let Some(dex_val) = updates.dex {
            character.set_ability_with_cascades(
                crate::character::types::AbilityIndex::DEX,
                dex_val,
                &game_data,
            )?;
        }
        if let Some(con_val) = updates.con {
            character.set_ability_with_cascades(
                crate::character::types::AbilityIndex::CON,
                con_val,
                &game_data,
            )?;
        }
        if let Some(int_val) = updates.int {
            character.set_ability_with_cascades(
                crate::character::types::AbilityIndex::INT,
                int_val,
                &game_data,
            )?;
        }
        if let Some(wis_val) = updates.wis {
            character.set_ability_with_cascades(
                crate::character::types::AbilityIndex::WIS,
                wis_val,
                &game_data,
            )?;
        }
        if let Some(cha_val) = updates.cha {
            character.set_ability_with_cascades(
                crate::character::types::AbilityIndex::CHA,
                cha_val,
                &game_data,
            )?;
        }
    }

    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let decoder = &session.item_property_decoder;
    Ok(character.get_abilities_state(&game_data, decoder))
}

#[tauri::command]
pub async fn apply_point_buy(
    state: State<'_, AppState>,
    new_scores: crate::character::types::AbilityScores,
) -> CommandResult<AbilitiesState> {
    use crate::character::abilities::{calculate_point_buy_cost, POINT_BUY_BUDGET, POINT_BUY_MIN, POINT_BUY_MAX};
    use crate::character::types::AbilityIndex;

    let cost = calculate_point_buy_cost(&new_scores);
    if cost > POINT_BUY_BUDGET {
        return Err(CommandError::ValidationError {
            field: "point_buy_cost".to_string(),
            reason: format!("Point buy cost {} exceeds budget {}", cost, POINT_BUY_BUDGET),
        });
    }

    for score in [
        new_scores.str_,
        new_scores.dex,
        new_scores.con,
        new_scores.int,
        new_scores.wis,
        new_scores.cha,
    ] {
        if score < POINT_BUY_MIN || score > POINT_BUY_MAX {
            return Err(CommandError::ValidationError {
                field: "ability_score".to_string(),
                reason: format!("Scores must be between {} and {}", POINT_BUY_MIN, POINT_BUY_MAX),
            });
        }
    }

    let game_data = state.game_data.read();

    {
        let mut session = state.session.write();
        let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;

        character.clear_ability_level_up_history()?;

        character.set_ability_with_cascades(AbilityIndex::STR, new_scores.str_, &game_data)?;
        character.set_ability_with_cascades(AbilityIndex::DEX, new_scores.dex, &game_data)?;
        character.set_ability_with_cascades(AbilityIndex::CON, new_scores.con, &game_data)?;
        character.set_ability_with_cascades(AbilityIndex::INT, new_scores.int, &game_data)?;
        character.set_ability_with_cascades(AbilityIndex::WIS, new_scores.wis, &game_data)?;
        character.set_ability_with_cascades(AbilityIndex::CHA, new_scores.cha, &game_data)?;
    }

    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let decoder = &session.item_property_decoder;
    Ok(character.get_abilities_state(&game_data, decoder))
}

#[tauri::command]
pub async fn get_classes_state(state: State<'_, AppState>) -> CommandResult<ClassesState> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let game_data = state.game_data.read();

    Ok(character.get_classes_state(&game_data))
}

#[tauri::command]
pub async fn get_feats_state(state: State<'_, AppState>) -> CommandResult<FeatsState> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let game_data = state.game_data.read();

    Ok(character.get_feats_state(&game_data))
}

#[tauri::command]
pub async fn get_spells_state(state: State<'_, AppState>) -> CommandResult<SpellsState> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let game_data = state.game_data.read();

    Ok(character.get_spells_state(&game_data))
}
