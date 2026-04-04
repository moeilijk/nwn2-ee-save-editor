use std::collections::BTreeMap;
use std::sync::Arc;

use app_lib::config::NWN2Paths;
use app_lib::parsers::gff::GffValue;
use app_lib::services::resource_manager::ResourceManager;
use tokio::sync::RwLock;

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
        _ => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = NWN2Paths::new();
    let paths_arc = Arc::new(RwLock::new(paths));
    let resource_manager = Arc::new(RwLock::new(ResourceManager::new(Arc::clone(&paths_arc))));
    {
        let mut rm = resource_manager.write().await;
        rm.initialize().await?;
    }

    let rm = resource_manager.read().await;
    let templates = rm.get_all_item_templates();
    let mut groups: BTreeMap<(i32, i32, i32), Vec<String>> = BTreeMap::new();

    for (resref, info) in templates {
        let fields = match rm.get_item_template_fields(&info) {
            Ok(fields) => fields,
            Err(_) => continue,
        };

        let Some(GffValue::ListOwned(props)) = fields.get("PropertiesList") else {
            continue;
        };

        for prop in props {
            let property_name = match prop.get("PropertyName") {
                Some(GffValue::Word(v)) => *v as i32,
                Some(GffValue::Int(v)) => *v,
                _ => continue,
            };
            if property_name != 1 {
                continue;
            }

            let base_item = match fields.get("BaseItem") {
                Some(GffValue::Int(v)) => *v,
                Some(GffValue::Word(v)) => *v as i32,
                _ => -1,
            };
            let cost = match prop.get("CostValue") {
                Some(GffValue::Word(v)) => *v as i32,
                Some(GffValue::Int(v)) => *v,
                _ => -1,
            };
            let param1_value = match prop.get("Param1Value") {
                Some(GffValue::Byte(v)) => *v as i32,
                Some(GffValue::Word(v)) => *v as i32,
                Some(GffValue::Int(v)) => *v,
                _ => -1,
            };
            let name = fields
                .get("LocalizedName")
                .and_then(|v| locstring_text(v, &rm))
                .unwrap_or_else(|| resref.clone());
            groups
                .entry((base_item, cost, param1_value))
                .or_default()
                .push(name);
        }
    }

    for ((base_item, cost, param1_value), mut names) in groups {
        names.sort();
        names.truncate(12);
        println!(
            "base_item={base_item} cost={cost} param1_value={param1_value} count={} names={:?}",
            names.len(),
            names
        );
    }

    Ok(())
}
