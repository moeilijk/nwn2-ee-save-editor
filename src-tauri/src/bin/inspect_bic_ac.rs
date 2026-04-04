use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use app_lib::character::Character;
use app_lib::config::NWN2Paths;
use app_lib::loaders::data_model_loader::DataModelLoader;
use app_lib::parsers::gff::{GffParser, GffValue};
use app_lib::services::item_property_decoder::ItemPropertyDecoder;
use app_lib::services::resource_manager::ResourceManager;
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
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: inspect_bic_ac <path-to-bic>")?;

    let bytes = std::fs::read(&path)?;
    let gff = GffParser::from_bytes(bytes)?;
    let fields = gff.read_struct_fields(0)?;
    let character = Character::from_gff(fields);

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

    println!("path={}", path.display());
    println!("name={}", character.full_name());
    println!("race={}", character.race_name(&game_data));
    println!("background_id={:?}", character.background_id(&game_data));
    println!("background={:?}", character.background(&game_data));
    println!("classes={:?}", character.class_entries());
    println!("creature_size={}", character.creature_size());
    println!(
        "size_modifier={}",
        character.get_size_modifier(character.creature_size(), &game_data)
    );
    println!("stored_armor_class={}", character.get_i32("ArmorClass").unwrap_or(-1));
    println!(
        "stored_base_attack_bonus={}",
        character.get_i32("BaseAttackBonus").unwrap_or(-1)
    );
    let ac = character.get_armor_class(&game_data, &decoder);
    println!("computed_ac_total={}", ac.total);
    println!(
        "computed_ac_breakdown base={} armor={} shield={} dex={} natural={} dodge={} deflection={} size={} misc={}",
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

    let rm = resource_manager.read().await;
    println!("feat_count={}", character.feat_entries().len());
    for entry in character.feat_entries() {
        let feat_name = character.get_feat_name(entry.feat_id, &game_data);
        if feat_name.to_ascii_lowercase().contains("natural leader")
            || feat_name.to_ascii_lowercase().contains("bully")
            || feat_name.to_ascii_lowercase().contains("merchant")
            || feat_name.to_ascii_lowercase().contains("harborman")
            || feat_name.to_ascii_lowercase().contains("kalach")
            || feat_name.to_ascii_lowercase().contains("city watch")
            || feat_name.to_ascii_lowercase().contains("waukeen")
        {
            println!("feat id={} name={}", entry.feat_id.0, feat_name);
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
            println!("item slot=0x{struct_id:04x} base_item={base_item} name={name}");
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
                let decoded = decoder.decode_all_properties(&prop_maps);
                for prop in decoded {
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
