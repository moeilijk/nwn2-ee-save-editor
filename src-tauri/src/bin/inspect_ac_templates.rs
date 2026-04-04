use std::sync::Arc;

use app_lib::config::NWN2Paths;
use app_lib::services::resource_manager::ResourceManager;
use tokio::sync::RwLock;

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
    if let Ok(itempropdef) = rm.get_2da("itempropdef") {
        for idx in [1usize, 39, 40, 44, 53] {
            if let Ok(row) = itempropdef.get_row_dict(idx) {
                println!("ITEMPROPDEF[{idx}]={row:?}");
            }
        }
    }
    if let Ok(table) = rm.get_2da("iprp_costtable") {
        for idx in [2usize, 3, 18] {
            if let Ok(row) = table.get_row_dict(idx) {
                println!("IPRP_COSTTABLE[{idx}]={row:?}");
            }
        }
    }
    for table_name in ["iprp_lightcost", "iprp_spells", "iprp_savingthrowcost"] {
        if let Ok(table) = rm.get_2da(table_name) {
            println!("{table_name}:");
            for idx in 0..table.row_count().min(12) {
                if let Ok(row) = table.get_row_dict(idx) {
                    println!("  [{idx}]={row:?}");
                }
            }
        }
    }
    if let Ok(table) = rm.get_2da("iprp_acmodtype") {
        println!("IPRP_ACMODTYPE:");
        for idx in 0..table.row_count().min(8) {
            if let Ok(row) = table.get_row_dict(idx) {
                println!("  [{idx}]={row:?}");
            }
        }
    }
    if let Ok(table) = rm.get_2da("armor") {
        println!("ARMOR:");
        for idx in 0..table.row_count().min(8) {
            if let Ok(row) = table.get_row_dict(idx) {
                println!("  [{idx}]={row:?}");
            }
        }
    }
    if let Ok(table) = rm.get_2da("creaturesize") {
        println!("CREATURESIZE:");
        for idx in 0..table.row_count().min(8) {
            if let Ok(row) = table.get_row_dict(idx) {
                println!("  [{idx}]={row:?}");
            }
        }
    }
    let templates = rm.get_all_item_templates();
    for resref in [
        "nw_maarcl049",
        "nw_it_mbracer008",
        "nw_ashmto002",
        "x0_it_mring009",
        "nw_it_mneck012",
    ] {
        println!("RESREF={resref}");
        let info = templates.get(resref).ok_or("missing template")?;
        let fields = rm.get_item_template_fields(info)?;
        for (k, v) in &fields {
            if matches!(
                k.as_str(),
                "TemplateResRef"
                    | "LocalizedName"
                    | "DescIdentified"
                    | "PropertiesList"
                    | "Tag"
                    | "BaseItem"
                    | "ArmorRulesType"
            ) {
                println!("  {k}={v:?}");
            }
        }
    }
    Ok(())
}
