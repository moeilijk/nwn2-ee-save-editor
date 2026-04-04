use app_lib::config::NWN2Paths;
use app_lib::services::campaign::content::extract_module_info;
use app_lib::services::savegame_handler::SaveGameHandler;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let save_path = std::env::args()
        .nth(1)
        .ok_or("usage: inspect_save_module <save-path>")?;
    let handler = SaveGameHandler::new(&save_path, false, false)?;
    let paths = NWN2Paths::new();
    let (info, _vars) = extract_module_info(&handler, &paths)?;
    println!("current_module={}", info.current_module);
    println!("module_name={}", info.module_name);
    println!("entry_area={}", info.entry_area);
    println!("campaign_id={}", info.campaign_id);
    println!("campaign={}", info.campaign);
    println!("custom_tlk={}", info.custom_tlk);
    println!("hak_list={:?}", info.hak_list);
    Ok(())
}
