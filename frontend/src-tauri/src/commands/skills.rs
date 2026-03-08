use crate::character::{SkillSummaryEntry, SkillPointsSummary, ABLE_LEARNER_FEAT_ID};
use crate::character::types::{SkillId, FeatId};
use crate::commands::{CommandError, CommandResult};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillChangeResult {
    pub skill_id: SkillId,
    pub old_ranks: i32,
    pub new_ranks: i32,
    pub points_spent: i32,
    pub points_remaining: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsStateResponse {
    pub class_skills: Vec<SkillSummaryEntry>,
    pub cross_class_skills: Vec<SkillSummaryEntry>,
    pub total_available: i32,
    pub spent_points: i32,
}

#[tauri::command]
pub async fn get_all_skills(state: State<'_, AppState>) -> CommandResult<Vec<SkillSummaryEntry>> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let decoder = &session.item_property_decoder;
    Ok(character.get_skill_summary(&game_data, Some(decoder)))
}

#[tauri::command]
pub async fn get_skill_ranks(state: State<'_, AppState>, skill_id: i32) -> CommandResult<i32> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.skill_rank(SkillId(skill_id)))
}

#[tauri::command]
pub async fn is_class_skill(state: State<'_, AppState>, skill_id: i32) -> CommandResult<bool> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    Ok(character.is_class_skill(SkillId(skill_id), &game_data))
}

#[tauri::command]
pub async fn calculate_skill_cost(
    state: State<'_, AppState>,
    skill_id: i32,
    ranks: i32,
) -> CommandResult<i32> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;

    let has_able_learner = character.has_feat(FeatId(ABLE_LEARNER_FEAT_ID));
    Ok(character.calculate_skill_cost(SkillId(skill_id), ranks, has_able_learner, &game_data))
}

#[tauri::command]
pub async fn get_skill_summary(state: State<'_, AppState>) -> CommandResult<Vec<SkillSummaryEntry>> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let decoder = &session.item_property_decoder;
    Ok(character.get_skill_summary(&game_data, Some(decoder)))
}

#[tauri::command]
pub async fn get_skills_state(state: State<'_, AppState>) -> CommandResult<SkillsStateResponse> {
    let session = state.session.read();
    let game_data = state.game_data.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
    let decoder = &session.item_property_decoder;

    let all_skills = character.get_skill_summary(&game_data, Some(decoder));
    let (class_skills, cross_class_skills): (Vec<_>, Vec<_>) =
        all_skills.into_iter().partition(|s| s.is_class_skill);

    let available_points = character.get_available_skill_points();
    let total_spent = character.total_skill_points_spent();

    Ok(SkillsStateResponse {
        class_skills,
        cross_class_skills,
        total_available: available_points + total_spent,
        spent_points: total_spent,
    })
}

#[tauri::command]
pub async fn set_skill_rank(
    state: State<'_, AppState>,
    skill_id: i32,
    ranks: i32,
) -> CommandResult<SkillChangeResult> {
    let game_data = state.game_data.read();
    let mut session = state.session.write();

    let (old_ranks, old_cost, new_cost) = {
        let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;
        let old_ranks = character.skill_rank(SkillId(skill_id));
        let has_able_learner = character.has_feat(FeatId(ABLE_LEARNER_FEAT_ID));
        let old_cost = character.calculate_skill_cost(SkillId(skill_id), old_ranks, has_able_learner, &game_data);
        let new_cost = character.calculate_skill_cost(SkillId(skill_id), ranks, has_able_learner, &game_data);
        (old_ranks, old_cost, new_cost)
    };

    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;
    character.set_skill_rank(SkillId(skill_id), ranks)?;

    let points_spent = new_cost - old_cost;

    Ok(SkillChangeResult {
        skill_id: SkillId(skill_id),
        old_ranks,
        new_ranks: ranks,
        points_spent,
        points_remaining: 0,
    })
}

#[tauri::command]
pub async fn reset_all_skills(state: State<'_, AppState>) -> CommandResult<i32> {
    let mut session = state.session.write();
    let character = session.character.as_mut().ok_or(CommandError::NoCharacterLoaded)?;

    let skill_ranks = character.skill_ranks();
    let mut total_refunded = 0;

    for entry in skill_ranks {
        character.set_skill_rank(entry.skill_id, 0)?;
        total_refunded += entry.ranks;
    }

    Ok(total_refunded)
}

#[tauri::command]
pub async fn get_skill_points_remaining(state: State<'_, AppState>) -> CommandResult<SkillPointsSummary> {
    let session = state.session.read();
    let character = session.character.as_ref().ok_or(CommandError::NoCharacterLoaded)?;

    let available_points = character.get_available_skill_points();
    let total_spent = character.total_skill_points_spent();

    Ok(SkillPointsSummary {
        available_points,
        total_points: total_spent + available_points,
    })
}
