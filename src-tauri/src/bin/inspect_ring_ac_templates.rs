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
    for (resref, info) in templates {
        let fields = match rm.get_item_template_fields(&info) {
            Ok(fields) => fields,
            Err(_) => continue,
        };
        let base_item = match fields.get("BaseItem") {
            Some(GffValue::Int(v)) => *v,
            Some(GffValue::Word(v)) => *v as i32,
            _ => continue,
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
            if property_name != 1 && property_name != 39 {
                continue;
            }
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
            let desc = fields
                .get("DescIdentified")
                .and_then(|v| locstring_text(v, &rm))
                .unwrap_or_default();
            println!("resref={resref}");
            println!("base_item={base_item}");
            println!("property_name={property_name}");
            println!("name={name}");
            println!("cost={cost} param1_value={param1_value}");
            println!("desc={desc}");
            println!("---");
        }
    }
    Ok(())
}
