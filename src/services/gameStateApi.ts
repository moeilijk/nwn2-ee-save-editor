import { invoke } from '@tauri-apps/api/core';

export interface CompanionInfluenceData {
  name: string;
  influence: number | null;
  recruitment: string;
  source: string;
}

export interface CompanionInfluenceResponse {
  companions: Record<string, CompanionInfluenceData>;
}

export interface UpdateCompanionInfluenceRequest {
  companion_id: string;
  new_influence: number;
}

export interface UpdateCompanionInfluenceResponse {
  success: boolean;
  companion_id: string;
  old_influence: number;
  new_influence: number;
  message: string;
  has_unsaved_changes: boolean;
}

export interface QuestVariable {
  name: string;
  value: number | string;
  type: string;
  category?: string;
}

export interface QuestGroup {
  prefix: string;
  name: string;
  variables: QuestVariable[];
  completed_count: number;
  active_count: number;
  total_count: number;
}

export interface QuestDetailsResponse {
  groups: QuestGroup[];
  total_quests: number;
  completed_quests: number;
  active_quests: number;
  unknown_quests: number;
  completion_rate: number;
}

export interface UpdateQuestVariableRequest {
  variable_name: string;
  value: number | string;
  variable_type: string;
}

export interface UpdateQuestVariableResponse {
  success: boolean;
  variable_name: string;
  old_value: number | string;
  new_value: number | string;
  message: string;
  has_unsaved_changes: boolean;
}

export interface BatchQuestUpdate {
  variable_name: string;
  value: number | string;
  variable_type: string;
}

export interface BatchUpdateQuestsResponse {
  success: boolean;
  updated_count: number;
  failed_count: number;
  updates: Array<{
    variable_name: string;
    success: boolean;
    error?: string;
  }>;
  message: string;
  has_unsaved_changes: boolean;
}

export interface CampaignVariable {
  variable_name: string;
  value: number | string;
  variable_type: string;
}

export interface CampaignVariablesResponse {
  integers: Record<string, number>;
  strings: Record<string, string>;
  floats: Record<string, number>;
  total_count: number;
}

export interface UpdateCampaignVariableRequest {
  variable_name: string;
  value: number | string;
  variable_type: string;
}

export interface UpdateCampaignVariableResponse {
  success: boolean;
  variable_name: string;
  old_value: number | string;
  new_value: number | string;
  message: string;
  has_unsaved_changes: boolean;
}

export interface CampaignSettingsResponse {
  campaign_file_path: string;
  guid: string;
  display_name: string;
  description: string;
  level_cap: number;
  xp_cap: number;
  companion_xp_weight: number;
  henchman_xp_weight: number;
  attack_neutrals: number;
  auto_xp_award: number;
  journal_sync: number;
  no_char_changing: number;
  use_personal_reputation: number;
  start_module: string;
  module_names: string[];
}

export interface UpdateCampaignSettingsRequest {
  level_cap?: number;
  xp_cap?: number;
  companion_xp_weight?: number;
  henchman_xp_weight?: number;
  attack_neutrals?: number;
  auto_xp_award?: number;
  journal_sync?: number;
  no_char_changing?: number;
  use_personal_reputation?: number;
}

export interface UpdateCampaignSettingsResponse {
  success: boolean;
  updated_fields: string[];
  warning: string;
}

export interface CampaignBackupInfo {
  filename: string;
  path: string;
  size_bytes: number;
  created: string;
}

export interface CampaignBackupsResponse {
  backups: CampaignBackupInfo[];
  campaign_name: string | null;
  campaign_guid: string | null;
}

export interface RestoreCampaignResponse {
  success: boolean;
  restored_from: string;
}

export interface QuestProgressData {
  variable: string;
  category: string;
  name: string;
  description?: string;
  current_stage: number;
  is_completed: boolean;
  xp: number;
  source: string;
  type_hint?: string;
}

export interface QuestProgressResponse {
  quests: QuestProgressData[];
  total_count: number;
}

export interface PlotVariableData {
  name: string;
  display_name?: string;
  description?: string;
  value: number | string;
  type: string;
  has_definition: boolean;
  category?: string;
  quest_text?: string;
  type_hint?: string;
}

export interface PlotVariablesResponse {
  quest_variables: PlotVariableData[];
  unknown_variables: PlotVariableData[];
  total_count: number;
}

export interface KnownQuestValue {
  value: number;
  description: string;
  is_completed: boolean;
}

export interface QuestInfoData {
  category: string;
  category_name: string;
  entry_id: number;
  quest_name: string;
  current_stage_text: string;
  xp: number;
}

export interface EnrichedQuestData {
  variable_name: string;
  current_value: number;
  variable_type: string;
  quest_info: QuestInfoData | null;
  known_values: KnownQuestValue[];
  confidence: 'high' | 'medium' | 'low';
  source: string;
  is_completed: boolean;
  is_active: boolean;
}

export interface UnmappedVariableData {
  variable_name: string;
  display_name: string;
  current_value: number | string;
  variable_type: string;
  category: string;
}

export interface QuestStats {
  total: number;
  completed: number;
  active: number;
  unmapped: number;
}

export interface DialogueCacheInfo {
  cached: boolean;
  version?: string;
  generated_at?: string;
  dialogue_count: number;
  mapping_count: number;
  campaign_name: string;
}

export interface EnrichedQuestsResponse {
  quests: EnrichedQuestData[];
  unmapped_variables: UnmappedVariableData[];
  stats: QuestStats;
  cache_info: DialogueCacheInfo;
}

export interface ModuleInfo {
  module_name: string;
  area_name: string;
  campaign: string;
  entry_area: string;
  module_description: string;
  campaign_id: string;
  current_module?: string;
}

export interface ModuleVariablesResponse {
  integers: Record<string, number>;
  strings: Record<string, string>;
  floats: Record<string, number>;
  total_count: number;
}

export interface UpdateModuleVariableRequest {
  variable_name: string;
  value: number | string;
  variable_type: string;
}

export interface UpdateModuleVariableResponse {
  success: boolean;
  variable_name: string;
  old_value: number | string;
  new_value: number | string;
  message: string;
  has_unsaved_changes: boolean;
}

export class GameStateAPI {
  async getCompanionInfluence(characterId: number): Promise<CompanionInfluenceResponse> {
    const companions = await invoke<Record<string, CompanionInfluenceData>>('get_companion_influence');
    return { companions };
  }

  async updateCompanionInfluence(
    characterId: number,
    companionId: string,
    newInfluence: number
  ): Promise<UpdateCompanionInfluenceResponse> {
    await invoke('update_companion_influence', { companionId, newInfluence });
    return {
        success: true,
        companion_id: companionId,
        old_influence: 0,
        new_influence: newInfluence,
        message: "Updated",
        has_unsaved_changes: true
    };
  }

  async getQuestDetails(characterId: number): Promise<QuestDetailsResponse> {
    console.warn("getQuestDetails not implemented in Rust backend");
    return {
        groups: [],
        total_quests: 0,
        completed_quests: 0,
        active_quests: 0,
        unknown_quests: 0,
        completion_rate: 0
    };
  }

  async updateQuestVariable(
    characterId: number,
    variableName: string,
    value: number | string,
    variableType: string
  ): Promise<UpdateQuestVariableResponse> {
    console.warn("updateQuestVariable not implemented");
    return {
         success: false,
         variable_name: variableName,
         old_value: value,
         new_value: value,
         message: "Not implemented",
         has_unsaved_changes: false
    };
  }

  async batchUpdateQuests(
    characterId: number,
    updates: BatchQuestUpdate[]
  ): Promise<BatchUpdateQuestsResponse> {
    console.warn("batchUpdateQuests not implemented");
    return {
         success: false,
         updated_count: 0,
         failed_count: updates.length,
         updates: [],
         message: "Not implemented",
         has_unsaved_changes: false
    };
  }

  async getCampaignVariables(characterId: number): Promise<CampaignVariablesResponse> {
    const data = await invoke<{
      integers: Record<string, number>;
      floats: Record<string, number>;
      strings: Record<string, string>;
    }>('get_campaign_variables');
    const total_count = Object.keys(data.integers).length
      + Object.keys(data.floats).length
      + Object.keys(data.strings).length;
    return {
      integers: data.integers,
      floats: data.floats,
      strings: data.strings,
      total_count,
    };
  }

  async updateCampaignVariable(
    characterId: number,
    variableName: string,
    value: number | string,
    variableType: string
  ): Promise<UpdateCampaignVariableResponse> {
    await invoke('update_campaign_variable', {
      variableName,
      value: String(value),
      variableType,
    });
    return {
      success: true,
      variable_name: variableName,
      old_value: value,
      new_value: value,
      message: 'Updated',
      has_unsaved_changes: true,
    };
  }

  async getCampaignSettings(characterId: number): Promise<CampaignSettingsResponse> {
    const [info] = await invoke<[ModuleInfo, ModuleVariablesResponse]>('get_module_info');
    if (!info.campaign_id) {
      throw new Error('No campaign associated with this save');
    }
    const settings = await invoke<CampaignSettingsResponse>('get_campaign_settings', {
      campaignId: info.campaign_id,
    });
    return settings;
  }

  async updateCampaignSettings(
    characterId: number,
    settingsUpdates: UpdateCampaignSettingsRequest
  ): Promise<UpdateCampaignSettingsResponse> {
    const currentSettings = await this.getCampaignSettings(characterId);
    const merged = {
      ...currentSettings,
      attack_neutrals: Boolean(currentSettings.attack_neutrals),
      auto_xp_award: Boolean(currentSettings.auto_xp_award),
      journal_sync: Boolean(currentSettings.journal_sync),
      no_char_changing: Boolean(currentSettings.no_char_changing),
      use_personal_reputation: Boolean(currentSettings.use_personal_reputation),
    };
    const updatedFields: string[] = [];
    if (settingsUpdates.level_cap !== undefined) { merged.level_cap = settingsUpdates.level_cap; updatedFields.push('level_cap'); }
    if (settingsUpdates.xp_cap !== undefined) { merged.xp_cap = settingsUpdates.xp_cap; updatedFields.push('xp_cap'); }
    if (settingsUpdates.companion_xp_weight !== undefined) { merged.companion_xp_weight = settingsUpdates.companion_xp_weight; updatedFields.push('companion_xp_weight'); }
    if (settingsUpdates.henchman_xp_weight !== undefined) { merged.henchman_xp_weight = settingsUpdates.henchman_xp_weight; updatedFields.push('henchman_xp_weight'); }
    if (settingsUpdates.attack_neutrals !== undefined) { merged.attack_neutrals = Boolean(settingsUpdates.attack_neutrals); updatedFields.push('attack_neutrals'); }
    if (settingsUpdates.auto_xp_award !== undefined) { merged.auto_xp_award = Boolean(settingsUpdates.auto_xp_award); updatedFields.push('auto_xp_award'); }
    if (settingsUpdates.journal_sync !== undefined) { merged.journal_sync = Boolean(settingsUpdates.journal_sync); updatedFields.push('journal_sync'); }
    if (settingsUpdates.no_char_changing !== undefined) { merged.no_char_changing = Boolean(settingsUpdates.no_char_changing); updatedFields.push('no_char_changing'); }
    if (settingsUpdates.use_personal_reputation !== undefined) { merged.use_personal_reputation = Boolean(settingsUpdates.use_personal_reputation); updatedFields.push('use_personal_reputation'); }
    await invoke('update_campaign_settings', { settings: merged });
    return { success: true, updated_fields: updatedFields, warning: '' };
  }

  async getCampaignBackups(characterId: number): Promise<CampaignBackupsResponse> {
    try {
        const backups = await invoke<any[]>('list_backups');
        return {
            backups: backups.map(b => ({
                filename: b.filename || b.path.split(/[\\/]/).pop(),
                path: b.path,
                size_bytes: b.size_bytes || 0,
                created: b.created_at || new Date().toISOString()
            })),
            campaign_name: "Current Session",
            campaign_guid: null
        };
    } catch (e) {
        console.warn("list_backups failed", e);
        return { backups: [], campaign_name: null, campaign_guid: null };
    }
  }

  async createBackup(): Promise<void> {
    await invoke('create_backup');
  }

  async restoreModulesFromBackup(backupPath: string): Promise<RestoreCampaignResponse> {
    try {
      await invoke('restore_modules_from_backup', { backupPath });
      return { success: true, restored_from: backupPath };
    } catch (e) {
      throw new Error(String(e));
    }
  }

  async restoreCampaignFromBackup(
    characterId: number,
    backupPath: string
  ): Promise<RestoreCampaignResponse> {
      try {
          await invoke('restore_backup', { 
              backupPath: backupPath, 
              createPreRestoreBackup: true 
          });
          return { success: true, restored_from: backupPath };
      } catch (e) {
          throw new Error(String(e));
      }
  }

  async getModuleInfo(characterId: number): Promise<ModuleInfo> {
    const [info] = await invoke<[ModuleInfo, ModuleVariablesResponse]>('get_module_info');
    return info;
  }

  async getAllModules(characterId: number): Promise<{modules: Array<{id: string, name: string, campaign: string, variable_count: number, is_current: boolean}>, current_module: string}> {
    const [info, variables] = await invoke<[ModuleInfo, ModuleVariablesResponse]>('get_module_info');
    return {
      modules: [{
        id: info.current_module || info.module_name,
        name: info.module_name,
        campaign: info.campaign,
        variable_count: variables.total_count || (Object.keys(variables.integers).length + Object.keys(variables.strings).length + Object.keys(variables.floats).length),
        is_current: true
      }],
      current_module: info.current_module || info.module_name
    };
  }

  async getModuleById(characterId: number, moduleId: string): Promise<ModuleInfo & {variables: ModuleVariablesResponse}> {
    const [info, variables] = await invoke<[ModuleInfo, ModuleVariablesResponse]>('get_module_info');
    const totalCount = Object.keys(variables.integers).length + Object.keys(variables.strings).length + Object.keys(variables.floats).length;
    return {
      ...info,
      variables: {
        ...variables,
        total_count: totalCount
      }
    };
  }

  async getModuleVariables(characterId: number): Promise<ModuleVariablesResponse> {
    const [, variables] = await invoke<[ModuleInfo, ModuleVariablesResponse]>('get_module_info');
    const totalCount = Object.keys(variables.integers).length + Object.keys(variables.strings).length + Object.keys(variables.floats).length;
    return {
      ...variables,
      total_count: totalCount
    };
  }

  async updateModuleVariable(
    characterId: number,
    variableName: string,
    value: number | string,
    variableType: string,
    moduleId?: string
  ): Promise<UpdateModuleVariableResponse> {
    await invoke('update_module_variable', {
      variableName,
      value: String(value),
      variableType,
      moduleId: moduleId ?? null,
    });
    return {
      success: true,
      variable_name: variableName,
      old_value: value,
      new_value: value,
      message: 'Updated',
      has_unsaved_changes: true,
    };
  }

  async getQuestProgress(characterId: number): Promise<QuestProgressResponse> {
      return { quests: [], total_count: 0 };
  }

  async getPlotVariables(characterId: number): Promise<PlotVariablesResponse> {
      return { quest_variables: [], unknown_variables: [], total_count: 0 };
  }

  async getEnrichedQuests(characterId: number): Promise<EnrichedQuestsResponse> {
      return { 
          quests: [], 
          unmapped_variables: [], 
          stats: { total: 0, completed: 0, active: 0, unmapped: 0 }, 
          cache_info: { cached: false, dialogue_count: 0, mapping_count: 0, campaign_name: "" } 
      };
  }
}

export const gameStateAPI = new GameStateAPI();
