use app_lib::services::resource_manager::module_loader::load_module_2das;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let module_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: inspect_module_2das <module-path>")?;
    let overrides = load_module_2das(&module_path, module_path.is_dir())?;
    println!("override_count={}", overrides.len());
    for key in overrides.keys() {
        if key.contains("background") || key.contains("feat") || key.contains("racial") {
            println!("{key}");
        }
    }
    Ok(())
}
