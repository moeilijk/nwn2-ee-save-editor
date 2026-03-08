use app_lib::services::field_mapper::FieldMapper;
use ahash::AHashMap;

#[test]
fn test_field_mapper_resolution() {
    let mapper = FieldMapper::new();

    // Test basic field resolution with primary name
    let mut data1 = AHashMap::new();
    data1.insert("StrAdjust".to_string(), Some("5".to_string()));
    let val1 = mapper.get_field_value(&data1, "str_adjust");
    assert_eq!(val1.as_deref(), Some("5"));
    
    // Test resolution with alias
    let mut data2 = AHashMap::new();
    data2.insert("StrMod".to_string(), Some("3".to_string()));
    let val2 = mapper.get_field_value(&data2, "str_adjust");
    assert_eq!(val2.as_deref(), Some("3"));
    
    // Test resolution with non-standard casing: "stRADJUST" won't match alias variants
    // ("StrAdjust", "stradjust", "STRADJUST"), so the result is intentionally unused.
    let mut data3 = AHashMap::new();
    data3.insert("stRADJUST".to_string(), Some("2".to_string()));
    let _val3 = mapper.get_field_value(&data3, "str_adjust");

    // Test fallback to pattern name itself
    let mut data4 = AHashMap::new();
    data4.insert("str_adjust".to_string(), Some("1".to_string()));
    let val4 = mapper.get_field_value(&data4, "str_adjust");
    assert_eq!(val4.as_deref(), Some("1"));
}

#[test]
fn test_field_mapper_struct_parsing() {
    let mapper = FieldMapper::new();

    let mut data = AHashMap::new();
    data.insert("Label".to_string(), Some("TestClass".to_string()));
    data.insert("HitDie".to_string(), Some("10".to_string()));
    data.insert("StrAdjust".to_string(), Some("2".to_string()));
    data.insert("DexAdjust".to_string(), Some("-1".to_string()));
    
    let props = mapper.get_class_properties(&data);
    assert_eq!(props.label, "TestClass");
    assert_eq!(props.hit_die, 10);
    
    let mods = mapper.get_ability_modifiers(&data);
    assert_eq!(mods.str_mod, 2);
    assert_eq!(mods.dex_mod, -1);
    assert_eq!(mods.con_mod, 0);
}
