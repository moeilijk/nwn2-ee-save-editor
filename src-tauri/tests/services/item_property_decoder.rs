use app_lib::parsers::gff::GffParser;
use app_lib::services::item_property_decoder::ItemPropertyDecoder;
use serde_json::Value;
use std::collections::HashMap;

#[path = "../common/mod.rs"]
mod common;

#[tokio::test]
async fn test_item_property_decoding_from_fixture() {
    let context = common::create_test_context().await;
    let mut decoder = ItemPropertyDecoder::new(context.resource_manager.clone());

    // Initialize decoder (loads 2da)
    decoder
        .initialize()
        .await
        .expect("Failed to initialize decoder");

    // Load a character fixture
    let data = common::load_test_gff("ryathstrongarm/ryathstrongarm4.bic");
    let parser = GffParser::from_bytes(data).expect("Failed to parse GFF");

    let root_struct = parser
        .read_struct_fields(0)
        .expect("Failed to read root fields");

    let mut item_with_props = None;

    if let Some(app_lib::parsers::gff::GffValue::List(items)) = root_struct.get("ItemList") {
        for lazy_item in items {
            let item_struct_fields = lazy_item.force_load();
            if let Some(app_lib::parsers::gff::GffValue::List(props)) =
                item_struct_fields.get("PropertiesList")
            {
                if !props.is_empty() {
                    item_with_props = Some(props.clone());
                    break;
                }
            }
        }
    }

    if item_with_props.is_none() {
        // Try Equip_ItemList
        if let Some(app_lib::parsers::gff::GffValue::List(items)) =
            root_struct.get("Equip_ItemList")
        {
            for lazy_item in items {
                let item_struct_fields = lazy_item.force_load();
                if let Some(app_lib::parsers::gff::GffValue::List(props)) =
                    item_struct_fields.get("PropertiesList")
                {
                    if !props.is_empty() {
                        item_with_props = Some(props.clone());
                        break;
                    }
                }
            }
        }
    }

    let properties_list =
        item_with_props.expect("Could not find any item with properties in fixture");

    // Convert first property to JSON-like map for decoder
    // Property structure: PropertyName (word), Subtype (word), CostValue (word), Param1Value (word) usually.
    // GffParser values are explicitly typed.

    for lazy_prop in properties_list {
        let prop_fields = lazy_prop.force_load();
        let mut prop_map = HashMap::new();

        if let Some(app_lib::parsers::gff::GffValue::Word(v)) = prop_fields.get("PropertyName") {
            prop_map.insert("PropertyName".to_string(), Value::from(*v));
        }
        if let Some(app_lib::parsers::gff::GffValue::Word(v)) = prop_fields.get("Subtype") {
            prop_map.insert("Subtype".to_string(), Value::from(*v));
        }
        if let Some(app_lib::parsers::gff::GffValue::Word(v)) = prop_fields.get("CostValue") {
            prop_map.insert("CostValue".to_string(), Value::from(*v));
        }
        if let Some(app_lib::parsers::gff::GffValue::Word(v)) = prop_fields.get("Param1Value") {
            prop_map.insert("Param1Value".to_string(), Value::from(*v));
        } else if let Some(app_lib::parsers::gff::GffValue::Byte(v)) =
            prop_fields.get("Param1Value")
        {
            prop_map.insert("Param1Value".to_string(), Value::from(*v));
        }

        let decoded = decoder.decode_property(&prop_map);
        if let Some(d) = decoded {
            println!("Decoded Property: {:?}", d);
            assert!(!d.label.is_empty());
        }
    }
}
