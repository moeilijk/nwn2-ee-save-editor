use app_lib::parsers::mdb::parser::MdbParser;
use app_lib::services::resource_manager::ResourceManager;
use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
#[ignore = "diagnostic — run manually with cargo test dump_armor_mdb_materials -- --ignored --nocapture"]
async fn dump_armor_mdb_materials() {
    let nwn2_paths = Arc::new(RwLock::new(app_lib::config::NWN2Paths::new()));
    let rm = Arc::new(RwLock::new(ResourceManager::new(nwn2_paths)));
    {
        let mut rm_guard = rm.write().await;
        let _ = rm_guard.initialize().await;
    }
    let rm = rm.read().await;

    let mut out = String::new();

    let bodies = [
        // Expected accessory resrefs for darksteel plate UTI
        // (ACLtShoulder=15, ACRtShoulder=15, ACLtArm=14, ACRtArm=12,
        //  ACLtBracer=17, ACRtBracer=17, ACLtShin=11, ACRtShin=11,
        //  ACLtLeg=14, ACRtKnee=11)
        "A_EEM_LShoulder15",
        "A_EEM_RShoulder15",
        "A_EEM_LUpArm14",
        "A_EEM_RUpArm12",
        "A_EEM_LBracer17",
        "A_EEM_RBracer17",
        "A_EEM_LLowLeg11",
        "A_EEM_RLowLeg11",
        "A_EEM_LUpLeg14",
        "A_EEM_RKnee11",
        // Sanity: body mesh for comparison
        "P_EEM_PF_Body03",
    ];

    for name in bodies {
        writeln!(out, "\n=== {name} ===").unwrap();
        let lower = name.to_lowercase();
        match rm.get_resource_bytes(&lower, "mdb") {
            Ok(bytes) => match MdbParser::parse(&bytes) {
                Ok(mdb) => {
                    writeln!(
                        out,
                        "  version: {}.{}, packets: {} (rigid={}, skin={}, hook={}, helm={}, hair={})",
                        mdb.header.major_version,
                        mdb.header.minor_version,
                        mdb.header.packet_count,
                        mdb.rigid_meshes.len(),
                        mdb.skin_meshes.len(),
                        mdb.hooks.len(),
                        mdb.helm.len(),
                        mdb.hair.len(),
                    )
                    .unwrap();
                    for m in &mdb.skin_meshes {
                        writeln!(
                            out,
                            "  SKIN name='{}' skel='{}' diff='{}' norm='{}' tint='{}' glow='{}' verts={} faces={}",
                            m.name,
                            m.skeleton_name,
                            m.material.diffuse_map_name,
                            m.material.normal_map_name,
                            m.material.tint_map_name,
                            m.material.glow_map_name,
                            m.vertices.len(),
                            m.faces.len(),
                        )
                        .unwrap();
                    }
                    for m in &mdb.rigid_meshes {
                        writeln!(
                            out,
                            "  RIGD name='{}' diff='{}' norm='{}' tint='{}' glow='{}' verts={} faces={}",
                            m.name,
                            m.material.diffuse_map_name,
                            m.material.normal_map_name,
                            m.material.tint_map_name,
                            m.material.glow_map_name,
                            m.vertices.len(),
                            m.faces.len(),
                        )
                        .unwrap();
                    }
                }
                Err(e) => writeln!(out, "  PARSE FAILED: {e}").unwrap(),
            },
            Err(e) => writeln!(out, "  NOT FOUND: {e}").unwrap(),
        }
    }

    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target_test")
        .join("armor_mdb_materials.txt");
    std::fs::create_dir_all(out_path.parent().unwrap()).unwrap();
    std::fs::write(&out_path, &out).unwrap();
    eprintln!("Output: {}", out_path.display());
    print!("{out}");
}
