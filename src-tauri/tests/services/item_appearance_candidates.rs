use crate::common::create_test_context;

#[tokio::test]
async fn list_armor_mesh_candidates_returns_body_mdbs_for_chest_slot() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;
    let result = app_lib::commands::item_appearance::list_armor_mesh_candidates_impl(
        16,
        ctx.loader.game_data().unwrap(),
        &rm,
    );
    assert!(
        !result.is_empty(),
        "expected at least one body MDB candidate"
    );
    assert!(
        result.iter().all(|r| r.to_lowercase().contains("_body")),
        "all candidates should be body MDBs, got: {result:?}"
    );
    assert!(
        result
            .iter()
            .any(|r| r.to_lowercase().starts_with("p_hhm_")),
        "expected at least one p_hhm_ prefixed candidate"
    );
}

#[tokio::test]
async fn list_armor_mesh_candidates_returns_helm_mdbs_for_head_slot() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;
    let result = app_lib::commands::item_appearance::list_armor_mesh_candidates_impl(
        17,
        ctx.loader.game_data().unwrap(),
        &rm,
    );
    assert!(
        !result.is_empty(),
        "expected at least one helm MDB candidate"
    );
    assert!(
        result.iter().all(|r| r.to_lowercase().contains("_helm")),
        "all candidates should be helm MDBs, got: {result:?}"
    );
}

#[tokio::test]
async fn list_armor_mesh_candidates_returns_empty_for_non_armor() {
    let ctx = create_test_context().await;
    let rm = ctx.resource_manager.read().await;
    let result = app_lib::commands::item_appearance::list_armor_mesh_candidates_impl(
        5,
        ctx.loader.game_data().unwrap(),
        &rm,
    );
    assert!(
        result.is_empty(),
        "weapons should return no armor candidates"
    );
}

#[tokio::test]
async fn load_item_model_respects_override_resref() {
    use app_lib::character::{ArmorAccessories, ItemAppearance, TintChannels};

    let ctx = create_test_context().await;
    let game_data = ctx.loader.game_data().unwrap();
    let rm = ctx.resource_manager.read().await;

    let appearance = ItemAppearance {
        variation: 2,
        model_parts: [0, 0, 0],
        tints: TintChannels::default(),
        armor_visual_type: Some(8),
        boots: None,
        gloves: None,
        accessories: ArmorAccessories::default(),
    };

    let result = app_lib::commands::item_appearance::load_item_model_impl(
        16,
        &appearance,
        None,
        Some("p_hhm_sc_body01"),
        game_data,
        &rm,
    );

    assert!(
        result.is_ok(),
        "load_item_model_impl should succeed: {result:?}"
    );
    let data = result.unwrap();
    assert!(
        !data.meshes.is_empty(),
        "override resref should produce at least one mesh"
    );
}
