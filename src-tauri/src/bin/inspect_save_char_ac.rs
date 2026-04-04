use std::collections::HashMap;
use std::sync::Arc;

use app_lib::character::Character;
use app_lib::config::NWN2Paths;
use app_lib::loaders::data_model_loader::DataModelLoader;
use app_lib::parsers::gff::{GffParser, GffValue};
use app_lib::services::item_property_decoder::ItemPropertyDecoder;
use app_lib::services::resource_manager::ResourceManager;
use app_lib::services::savegame_handler::SaveGameHandler;
use app_lib::services::playerinfo::PlayerInfo;
use app_lib::state::session_state::SessionState;
use app_lib::character::types::SkillId;
use serde_json::Value as JsonValue;
use tokio::sync::RwLock;

fn gff_to_json_primitive(v: &GffValue<'static>) -> Option<JsonValue> {
    match v {
        GffValue::Byte(n) => Some(JsonValue::from(*n)),
        GffValue::Char(n) => Some(JsonValue::from(n.to_string())),
        GffValue::Word(n) => Some(JsonValue::from(*n)),
        GffValue::Short(n) => Some(JsonValue::from(*n)),
        GffValue::Dword(n) => Some(JsonValue::from(*n)),
        GffValue::Int(n) => Some(JsonValue::from(*n)),
        GffValue::Dword64(n) => Some(JsonValue::from(*n)),
        GffValue::Int64(n) => Some(JsonValue::from(*n)),
        GffValue::Float(n) => Some(JsonValue::from(*n as f64)),
        GffValue::Double(n) => Some(JsonValue::from(*n)),
        GffValue::String(s) => Some(JsonValue::from(s.to_string())),
        GffValue::ResRef(s) => Some(JsonValue::from(s.to_string())),
        _ => None,
    }
}

fn locstring_text(value: &GffValue<'_>, rm: &ResourceManager) -> Option<String> {
    match value {
        GffValue::LocString(ls) => {
            if let Some(first) = ls.substrings.first() {
                Some(first.string.to_string())
            } else if ls.string_ref >= 0 {
                Some(rm.get_string(ls.string_ref))
            } else {
                None
            }
        }
        GffValue::String(s) => Some(s.to_string()),
        GffValue::ResRef(s) => Some(s.to_string()),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let save_path = std::env::args()
        .nth(1)
        .ok_or("usage: inspect_save_char_ac <save-path> <player-index>")?;
    let player_index = std::env::args()
        .nth(2)
        .ok_or("usage: inspect_save_char_ac <save-path> <player-index>")?
        .parse::<usize>()?;

    let paths = NWN2Paths::new();
    let paths_arc = Arc::new(RwLock::new(paths));
    let resource_manager = Arc::new(RwLock::new(ResourceManager::new(Arc::clone(&paths_arc))));
    {
        let mut rm = resource_manager.write().await;
        rm.initialize().await?;
    }
    let tlk = {
        let rm = resource_manager.read().await;
        rm.get_tlk_parser().ok_or("Missing TLK parser")?
    };
    let mut loader = DataModelLoader::with_options(Arc::clone(&resource_manager), true, false);
    let game_data = loader.load_game_data(Arc::clone(&tlk)).await?;
    let decoder = ItemPropertyDecoder::new(Arc::clone(&resource_manager));

    let handler = SaveGameHandler::new(&save_path, false, false)?;
    let playerlist_data = handler.extract_player_data()?;
    let playerlist_parser = GffParser::from_bytes(playerlist_data)?;
    let playerlist_root = playerlist_parser.read_struct_fields(0)?;
    let playerlist_entry = match playerlist_root.get("Mod_PlayerList") {
        Some(GffValue::ListOwned(entries)) => entries.get(player_index).cloned(),
        Some(GffValue::List(entries)) => entries.get(player_index).map(|entry| entry.force_load()),
        _ => None,
    };
    let player_bic_root = handler
        .extract_player_bic()?
        .map(GffParser::from_bytes)
        .transpose()?
        .map(|parser| parser.read_struct_fields(0))
        .transpose()?;
    let playerinfo = PlayerInfo::load(std::path::Path::new(&save_path).join("playerinfo.bin"))?;

    let mut session = SessionState::new(Arc::clone(&resource_manager));
    session.load_character(&save_path, Some(player_index))?;
    let character = session.character.as_ref().ok_or("No character loaded")?;
    let ac = character.get_armor_class(&game_data, &decoder);
    let effective = character.get_effective_abilities(&game_data);
    let base_scores = character.base_scores();
    let saves = character.get_save_summary(&game_data, &decoder);
    let overview = character.get_overview_state(&game_data);
    let feats = character.feat_entries();
    let feat_save_bonuses = character.get_feat_save_bonuses(&game_data);
    let feat_ac_bonus = character.get_feat_ac_bonuses(&game_data);
    let item_bonuses = character.get_equipment_bonuses(&game_data, &decoder);
    let rm = resource_manager.read().await;

    println!("save={save_path}");
    println!("player_index={player_index}");
    println!("name={}", character.full_name());
    println!("loaded_background_id={:?}", character.background_id(&game_data));
    println!("loaded_background={:?}", character.background(&game_data));
    println!("race={}", character.race_name(&game_data));
    println!("subrace={:?}", character.subrace_name(&game_data));
    println!("deity={}", character.deity());
    println!(
        "alignment={} ({}/{})",
        overview.alignment_string,
        overview.alignment.law_chaos,
        overview.alignment.good_evil
    );
    println!("classes={:?}", character.class_entries());
    println!("stored_ac={}", character.get_i32("ArmorClass").unwrap_or(-1));
    println!("stored_bab={}", character.get_i32("BaseAttackBonus").unwrap_or(-1));
    println!("stored_fortbonus={}", character.get_i32("fortbonus").unwrap_or(-1));
    println!("stored_refbonus={}", character.get_i32("refbonus").unwrap_or(-1));
    println!("stored_willbonus={}", character.get_i32("willbonus").unwrap_or(-1));
    println!("stored_sr={}", character.get_i32("SR").unwrap_or(-1));
    println!(
        "playerinfo name={} background_row={} classes={:?} stats={}/{}/{}/{}/{}/{}",
        playerinfo.data.display_name(),
        playerinfo.data.unknown4,
        playerinfo.data.classes,
        playerinfo.data.str_score,
        playerinfo.data.dex_score,
        playerinfo.data.con_score,
        playerinfo.data.int_score,
        playerinfo.data.wis_score,
        playerinfo.data.cha_score
    );
    if let Some(entry) = playerlist_entry.as_ref() {
        let playerlist_character = Character::from_gff(entry.clone());
        println!(
            "playerlist background_id={:?} background={:?} race={} classes={:?}",
            playerlist_character.background_id(&game_data),
            playerlist_character.background(&game_data),
            playerlist_character.race_name(&game_data),
            playerlist_character.class_entries()
        );
    }
    if let Some(root) = player_bic_root.as_ref() {
        let bic_character = Character::from_gff(
            root.iter()
                .map(|(k, v)| (k.clone(), v.clone().force_owned()))
                .collect()
        );
        println!(
            "player_bic background_id={:?} background={:?} race={} classes={:?}",
            bic_character.background_id(&game_data),
            bic_character.background(&game_data),
            bic_character.race_name(&game_data),
            bic_character.class_entries()
        );
    }
    println!("calc_bab={}", character.calculate_bab(&game_data));
    println!("creature_size={}", character.creature_size());
    println!(
        "size_modifier={}",
        character.get_size_modifier(character.creature_size(), &game_data)
    );
    println!(
        "base_scores str={} dex={} con={} int={} wis={} cha={}",
        base_scores.str_, base_scores.dex, base_scores.con, base_scores.int, base_scores.wis, base_scores.cha
    );
    println!(
        "effective_scores str={} dex={} con={} int={} wis={} cha={}",
        effective.str_, effective.dex, effective.con, effective.int, effective.wis, effective.cha
    );
    println!("tumble_rank={}", character.skill_rank(SkillId(21)));
    println!("spell_resistance_base={}", character.spell_resistance());
    println!(
        "spell_resistance_racial={}",
        character.get_racial_spell_resistance(&game_data)
    );
    println!(
        "spell_resistance_item={}",
        item_bonuses.spell_resistance
    );
    println!(
        "spell_resistance_total={}",
        character.get_total_spell_resistance(&game_data)
    );
    println!(
        "feat_save_bonuses fort={} reflex={} will={}",
        feat_save_bonuses.fortitude, feat_save_bonuses.reflex, feat_save_bonuses.will
    );
    println!(
        "item_bonuses str={} dex={} con={} int={} wis={} cha={} fort={} reflex={} will={} sr={} ac={} armor={} shield={} natural={} deflection={} dodge={}",
        item_bonuses.str_bonus,
        item_bonuses.dex_bonus,
        item_bonuses.con_bonus,
        item_bonuses.int_bonus,
        item_bonuses.wis_bonus,
        item_bonuses.cha_bonus,
        item_bonuses.fortitude_bonus,
        item_bonuses.reflex_bonus,
        item_bonuses.will_bonus,
        item_bonuses.spell_resistance,
        item_bonuses.ac_bonus,
        item_bonuses.ac_armor_bonus,
        item_bonuses.ac_shield_bonus,
        item_bonuses.ac_natural_bonus,
        item_bonuses.ac_deflection_bonus,
        item_bonuses.ac_dodge_bonus
    );
    println!("feat_ac_bonus={}", feat_ac_bonus);
    println!(
        "saves fort={} reflex={} will={}",
        saves.fortitude, saves.reflex, saves.will
    );
    println!(
        "save_breakdown fort base={} ability={} equip={} feat={} racial={} class={} misc={}",
        saves.saves.fortitude.base,
        saves.saves.fortitude.ability,
        saves.saves.fortitude.equipment,
        saves.saves.fortitude.feat,
        saves.saves.fortitude.racial,
        saves.saves.fortitude.class_bonus,
        saves.saves.fortitude.misc
    );
    println!(
        "save_breakdown reflex base={} ability={} equip={} feat={} racial={} class={} misc={}",
        saves.saves.reflex.base,
        saves.saves.reflex.ability,
        saves.saves.reflex.equipment,
        saves.saves.reflex.feat,
        saves.saves.reflex.racial,
        saves.saves.reflex.class_bonus,
        saves.saves.reflex.misc
    );
    println!(
        "save_breakdown will base={} ability={} equip={} feat={} racial={} class={} misc={}",
        saves.saves.will.base,
        saves.saves.will.ability,
        saves.saves.will.equipment,
        saves.saves.will.feat,
        saves.saves.will.racial,
        saves.saves.will.class_bonus,
        saves.saves.will.misc
    );
    println!(
        "breakdown base={} armor={} shield={} dex={} natural={} dodge={} deflection={} size={} misc={}",
        ac.breakdown.base,
        ac.breakdown.armor,
        ac.breakdown.shield,
        ac.breakdown.dex,
        ac.breakdown.natural,
        ac.breakdown.dodge,
        ac.breakdown.deflection,
        ac.breakdown.size,
        ac.breakdown.misc
    );
    println!("total={}", ac.total);
    println!("mode_related_fields:");
    for key in character.field_names() {
        let lower = key.to_ascii_lowercase();
        if lower.contains("mode")
            || lower.contains("state")
            || lower.contains("expert")
            || lower.contains("dodge")
            || lower.contains("defens")
        {
            if let Some(value) = character.gff().get(key) {
                println!("  {key}={value:?}");
            }
        }
    }
    println!("feat_count={}", feats.len());
    println!("feats:");
    for entry in feats {
        let feat_name = character.get_feat_name(entry.feat_id, &game_data);
        println!("  feat_id={} name={}", entry.feat_id.0, feat_name);
        if let Some(info) = character.get_feat_info(entry.feat_id, &game_data) {
            let desc = info.description.replace('\n', " ");
            if entry.feat_id.0 >= 1600
                || (227..=233).contains(&entry.feat_id.0)
                || desc.to_ascii_lowercase().contains("strength")
                || desc.to_ascii_lowercase().contains("dexterity")
                || desc.to_ascii_lowercase().contains("constitution")
                || desc.to_ascii_lowercase().contains("save")
                || desc.to_ascii_lowercase().contains("spell resistance")
                || desc.to_ascii_lowercase().contains("armor class")
                || desc.to_ascii_lowercase().contains("ac ")
            {
                println!("    desc={}", desc);
            }
        }
    }

    if let Some(GffValue::ListOwned(items)) = character.gff().get("Equip_ItemList") {
        for item in items {
            let struct_id = item
                .get("__struct_id__")
                .and_then(|v| match v {
                    GffValue::Dword(v) => Some(*v),
                    GffValue::Int(v) => Some(*v as u32),
                    _ => None,
                })
                .unwrap_or(0);
            if struct_id == 0 {
                continue;
            }
            let base_item = item
                .get("BaseItem")
                .and_then(|v| match v {
                    GffValue::Int(v) => Some(*v),
                    GffValue::Word(v) => Some(*v as i32),
                    _ => None,
                })
                .unwrap_or(-1);
            let name = item
                .get("LocalizedName")
                .and_then(|v| locstring_text(v, &rm))
                .unwrap_or_else(|| "<unnamed>".to_string());
            let resref = item
                .get("TemplateResRef")
                .and_then(|v| locstring_text(v, &rm))
                .unwrap_or_default();
            let desc = item
                .get("DescIdentified")
                .and_then(|v| locstring_text(v, &rm))
                .unwrap_or_default()
                .replace('\n', " ");
            println!("item slot=0x{struct_id:04x} base_item={base_item} resref={resref} name={name}");
            if !desc.is_empty() {
                println!("  desc={desc}");
            }
            if let Some(GffValue::ListOwned(props)) = item.get("PropertiesList") {
                let mut prop_maps: Vec<HashMap<String, JsonValue>> = Vec::new();
                for prop in props {
                    let mut map = HashMap::new();
                    for (k, v) in prop {
                        if let Some(json_val) = gff_to_json_primitive(v) {
                            map.insert(k.clone(), json_val);
                        }
                    }
                    prop_maps.push(map);
                }
                for prop in decoder.decode_all_properties(&prop_maps) {
                    println!(
                        "  prop id={} label={} type={} ac_type={:?} bonus={:?}",
                        prop.property_id, prop.label, prop.bonus_type, prop.ac_type, prop.bonus_value
                    );
                }
            }
        }
    }

    Ok(())
}
