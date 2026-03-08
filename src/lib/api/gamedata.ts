import { invoke } from '@tauri-apps/api/core';
import type {
  AvailableRace,
  AvailableClass,
  AvailableFeat,
  AvailableSkill,
  AvailableSpell,
  AvailableDeity,
  AvailableDomain,
  AvailableGender,
  AvailableAlignment,
  AvailableBackground,
  AvailableAbility,
  AvailableBaseItem,
  AvailableSpellSchool,
  AvailableItemProperty,
} from '../bindings';

export interface PagedResponse<T> {
  count: number;
  next: string | null;
  previous: string | null;
  results: T[];
  page?: number;
  page_size?: number;
}

export interface GameDataItem {
  id: number;
  label: string;
  name: string;
  [key: string]: unknown;
}

export class GameDataService {
  private wrapPaged<T>(results: T[]): PagedResponse<T> {
    return {
      count: results.length,
      next: null,
      previous: null,
      results,
      page: 1,
      page_size: results.length
    };
  }

  appearance = {
    get: async () => ({}),
    getPortraits: async () => ({}),
    getSoundsets: async () => ({}),
    getAll: async () => ({
      appearance: {},
      portraits: {},
      soundsets: {},
      gender: {}
    }),
  };

  races = async (): Promise<PagedResponse<AvailableRace>> => {
    const races = await invoke<AvailableRace[]>('get_available_races');
    return this.wrapPaged(races);
  };

  subraces = async (): Promise<PagedResponse<GameDataItem>> => this.wrapPaged([]);

  classes = async (): Promise<PagedResponse<AvailableClass>> => {
    const classes = await invoke<AvailableClass[]>('get_available_classes');
    return this.wrapPaged(classes);
  };

  genders = async (): Promise<PagedResponse<AvailableGender>> => {
    const genders = await invoke<AvailableGender[]>('get_available_genders');
    return this.wrapPaged(genders);
  };

  alignments = async (): Promise<PagedResponse<AvailableAlignment>> => {
    const alignments = await invoke<AvailableAlignment[]>('get_available_alignments');
    return this.wrapPaged(alignments);
  };

  deities = async (): Promise<PagedResponse<AvailableDeity>> => {
    const deities = await invoke<AvailableDeity[]>('get_available_deities');
    return this.wrapPaged(deities);
  };

  domains = async (): Promise<PagedResponse<AvailableDomain>> => {
    const domains = await invoke<AvailableDomain[]>('get_all_domains');
    return this.wrapPaged(domains);
  };

  backgrounds = async (): Promise<PagedResponse<AvailableBackground>> => {
    const backgrounds = await invoke<AvailableBackground[]>('get_available_backgrounds');
    return this.wrapPaged(backgrounds);
  };

  feats = async (_characterId?: number, _featType?: number): Promise<AvailableFeat[]> => {
    return await invoke<AvailableFeat[]>('get_available_feats');
  };

  skills = async (): Promise<PagedResponse<AvailableSkill>> => {
    const skills = await invoke<AvailableSkill[]>('get_available_skills');
    return this.wrapPaged(skills);
  };

  spells = async (_characterId?: number, _filters?: {
    level?: number;
    school?: string;
    search?: string;
  }): Promise<AvailableSpell[]> => {
    return await invoke<AvailableSpell[]>('get_available_spells');
  };

  abilities = async (): Promise<PagedResponse<AvailableAbility>> => {
    const abilities = await invoke<AvailableAbility[]>('get_available_abilities');
    return this.wrapPaged(abilities);
  };

  baseItems = async (): Promise<PagedResponse<AvailableBaseItem>> => {
    const items = await invoke<AvailableBaseItem[]>('get_available_base_items');
    return this.wrapPaged(items);
  };

  itemProperties = async (): Promise<PagedResponse<AvailableItemProperty>> => {
    const props = await invoke<AvailableItemProperty[]>('get_available_item_properties');
    return this.wrapPaged(props);
  };

  featCategories = async (): Promise<PagedResponse<GameDataItem>> => this.wrapPaged([]);

  spellSchools = async (): Promise<PagedResponse<AvailableSpellSchool>> => {
    const schools = await invoke<AvailableSpellSchool[]>('get_available_spell_schools');
    return this.wrapPaged(schools);
  };

  skillCategories = async (): Promise<PagedResponse<GameDataItem>> => this.wrapPaged([]);

  companions = async (): Promise<PagedResponse<GameDataItem>> => this.wrapPaged([]);
  packages = async (): Promise<PagedResponse<GameDataItem>> => this.wrapPaged([]);
}

export const gameData = new GameDataService();
