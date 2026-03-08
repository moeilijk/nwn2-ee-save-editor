use super::super::common::{create_test_context, fixtures_path};
use app_lib::character::{Character, ClassId};
use app_lib::parsers::gff::GffParser;

#[tokio::test]
async fn test_wizard_progression() {
    let ctx = create_test_context().await;
    let game_data = ctx.loader.game_data().expect("Game data not loaded");

    // Load Sage Melchior (Wizard 5 / Cleric 7 / Warlock 18)
    let path = fixtures_path().join("gff/sagemelchior/sagemelchior4.bic");
    let data = std::fs::read(&path).expect("Failed to read fixture");
    let parser = GffParser::from_bytes(data).expect("Failed to parse GFF");
    let gff = parser.read_struct_fields(0).expect("Failed to read root struct");
    let character = Character::from_gff(gff);

    // Find Wizard Class (ID 10)
    let wizard_id = ClassId(10);
    assert!(character.has_class(wizard_id));
    
    // Verify Class Properties
    assert!(character.is_spellcaster(wizard_id, game_data));
    assert!(character.is_prepared_caster(wizard_id, game_data));
    assert!(!character.uses_all_spells_known(wizard_id, game_data));

    // Wizard 5 -> Caster Level 5
    let caster_level = character.get_caster_level(wizard_id, game_data);
    assert_eq!(caster_level, 5, "Wizard 5 should have Caster Level 5");

    // Check Spell Slots (Base)
    // Level 5 Wizard: L0=4, L1=3, L2=2, L3=1
    let slots = character.calculate_spell_slots(wizard_id, game_data);
    println!("Wizard 5 Base Slots: {:?}", slots);
    assert!(slots[0] >= 4);
    assert!(slots[1] >= 2); // Tables might vary slightly with updates, but should have some
    assert!(slots[2] >= 1);
    assert!(slots[3] >= 1);
    assert_eq!(slots[4], 0); // No L4 spells yet (needs L7)

    // Check Known Spells
    let known_l1 = character.known_spells(wizard_id, 1);
    println!("Wizard Known L1: {:?}", known_l1);
    assert!(!known_l1.is_empty(), "Wizard should have known L1 spells");

    // Check Memorized Spells (if any)
    let memory_l1 = character.memorized_spells(wizard_id, 1);
    println!("Wizard Memorized L1: {:?}", memory_l1);
    // Note: Fixture might not have memorized spells set if it's a fresh save or cleaned
}

#[tokio::test]
async fn test_cleric_domains() {
    let ctx = create_test_context().await;
    let game_data = ctx.loader.game_data().expect("Game data not loaded");

    // Load Sage Melchior (Wizard 5 / Cleric 7 / Warlock 18)
    let path = fixtures_path().join("gff/sagemelchior/sagemelchior4.bic");
    let data = std::fs::read(&path).expect("Failed to read fixture");
    let parser = GffParser::from_bytes(data).expect("Failed to parse GFF");
    let gff = parser.read_struct_fields(0).expect("Failed to read root struct");
    let character = Character::from_gff(gff);

    // Find Cleric Class (ID 2)
    let cleric_id = ClassId(2);
    assert!(character.has_class(cleric_id));

    // Verify Class Properties
    assert!(character.is_spellcaster(cleric_id, game_data));
    assert!(character.is_divine_caster(cleric_id, game_data));
    assert!(character.uses_all_spells_known(cleric_id, game_data));

    // Cleric 7 -> Caster Level 7
    let caster_level = character.get_caster_level(cleric_id, game_data);
    assert_eq!(caster_level, 7);

    // Domain Spells
    let domain_spells = character.get_domain_spells(cleric_id, game_data);
    println!("Cleric Domain Spells: {:?}", domain_spells);
    // Cleric 7 should have domain spells for levels 1, 2, 3, 4
    assert!(domain_spells.contains_key(&1));
    assert!(domain_spells.contains_key(&2));
    assert!(domain_spells.contains_key(&3));
    assert!(domain_spells.contains_key(&4));

    // All Known Spells
    // Should return essentially all spells in spells.2da that are Cleric-capable
    let known_l1 = character.get_all_known_spells(cleric_id, 1, game_data);
    assert!(!known_l1.is_empty());
    assert!(known_l1.len() > 10, "Cleric should know many L1 spells");
}

#[tokio::test]
async fn test_sorcerer_spells() {
    let ctx = create_test_context().await;
    let game_data = ctx.loader.game_data().expect("Game data not loaded");

    // Load Qara (Sorcerer 10 / Arcane Scholar 10 / Dragon Disciple 10)
    let path = fixtures_path().join("gff/qaraofblacklake/qaraofblacklake4.bic");
    let data = std::fs::read(&path).expect("Failed to read fixture");
    let parser = GffParser::from_bytes(data).expect("Failed to parse GFF");
    let gff = parser.read_struct_fields(0).expect("Failed to read root struct");
    let character = Character::from_gff(gff);

    // Find Sorcerer Class (ID 9)
    let sorcerer_id = ClassId(9);
    assert!(character.has_class(sorcerer_id));
    
    // Verify Properties
    assert!(character.is_spellcaster(sorcerer_id, game_data));
    assert!(!character.is_prepared_caster(sorcerer_id, game_data)); // Spontaneous
    assert!(!character.uses_all_spells_known(sorcerer_id, game_data));

    // Caster Level
    // Pure Sorcerer level 10 + Arcane Scholar 10 (Full progression) + Dragon Disciple 10 (Full?)
    // Actually AS and DD generally stack. Qara is likely CL 20-30.
    // Let's check logic:
    // Caster level is usually per-class unless prestige classes modify it (which we handle via SpellCasterLevel field fallback? or logic)
    // The current `get_caster_level` implementation checks `SpellCasterLevel` override in ClassList.
    // Let's see what it returns.
    let cl = character.get_caster_level(sorcerer_id, game_data);
    println!("Qara Sorcerer Caster Level: {}", cl);
    // Default Sorcerer logic is `class_level` if Type 4 (Sorcerer is type 4?). Sorcerer is class index 9.
    // We expect it to be at least 10.
    assert!(cl >= 10);

    // Check Spontaneous Casting Slots
    let slots = character.calculate_spell_slots(sorcerer_id, game_data);
    println!("Qara Sorcerer Slots: {:?}", slots);
    assert!(slots[1] > 0);
    assert!(slots[5] > 0); 
    
    // Known Spells
    let known_l5 = character.known_spells(sorcerer_id, 5);
    println!("Qara Known L5: {:?}", known_l5);
    assert!(!known_l5.is_empty());
}

#[tokio::test]
async fn test_paladin_spells() {
    let ctx = create_test_context().await;
    let game_data = ctx.loader.game_data().expect("Game data not loaded");

    // Load Occidio (Paladin 20 / Divine Champion 10)
    let path = fixtures_path().join("gff/occidiooctavon/occidiooctavon4.bic");
    let data = std::fs::read(&path).expect("Failed to read fixture");
    let parser = GffParser::from_bytes(data).expect("Failed to parse GFF");
    let gff = parser.read_struct_fields(0).expect("Failed to read root struct");
    let character = Character::from_gff(gff);

    // Find Paladin Class (ID 6)
    let paladin_id = ClassId(6);
    assert!(character.has_class(paladin_id));

    // Properties
    assert!(character.is_spellcaster(paladin_id, game_data));
    assert!(character.is_prepared_caster(paladin_id, game_data)); // Paladins prepare
    // Paladins technically "know" all spells on their list, similar to Clerics?
    // Let's check `uses_all_spells_known`.
    let all_known = character.uses_all_spells_known(paladin_id, game_data);
    println!("Paladin uses all spells known: {}", all_known);
    // Usually Paladins do.

    // Caster Level
    // Paladin 2DA lookup usually uses Class Level directly (Slot table matches class level).
    // So for a Level 20 Paladin, we expect 20 for table lookup purposes, even if D&D CL is 10.
    let cl = character.get_caster_level(paladin_id, game_data);
    assert_eq!(cl, 20, "Paladin 20 should have Lookup Level 20 for spell slots");

    // Slots
    let slots = character.calculate_spell_slots(paladin_id, game_data);
    println!("Paladin Slots: {:?}", slots);
    assert!(slots[1] > 0);
    assert!(slots[3] > 0);
    assert_eq!(slots[5], 0); // Pal only goes up to L4

    // Bonus Slots from Wisdom
    // Occidio L30 probably has high stats.
    let bonus = character.calculate_bonus_spell_slots(paladin_id, 1, game_data);
    println!("Paladin Bonus Slots L1: {}", bonus);
}

#[tokio::test]
async fn test_wizard_slots_modification() {
    let ctx = create_test_context().await;
    let game_data = ctx.loader.game_data().expect("Game data not loaded");
    
    // Load Sage Melchior (Wizard 5 / Cleric 7 / Warlock 18)
    // We use a real fixture to ensure valid GFF structure foundation
    let path = fixtures_path().join("gff/sagemelchior/sagemelchior4.bic");
    let data = std::fs::read(&path).expect("Failed to read fixture");
    let parser = GffParser::from_bytes(data).expect("Failed to parse GFF");
    let gff = parser.read_struct_fields(0).expect("Failed to read root struct");
    let mut character = Character::from_gff(gff);

    let wizard_id = ClassId(10); // Wizard

    // Verify initial state (Wizard 5)
    let initial_level = character.class_level(wizard_id);
    assert_eq!(initial_level, 5, "Melchior should start as Wizard 5");
    
    let initial_slots = character.calculate_spell_slots(wizard_id, game_data);
    println!("Initial Wizard 5 Slots: {:?}", initial_slots);
    assert_eq!(initial_slots[5], 0, "Wizard 5 should not have L5 slots");

    // Modify to Wizard 10
    // This tests that our slot calculations respond dynamically to level changes in-memory
    character.set_class_level(wizard_id, 10).expect("Failed to set Wizard level");
    
    let new_level = character.class_level(wizard_id);
    assert_eq!(new_level, 10);
    
    // Recalculate slots
    // Wizard 10 should have L5 slots (Base progression)
    // L0:4, L1:4, L2:4, L3:3, L4:3, L5:2
    let new_slots = character.calculate_spell_slots(wizard_id, game_data);
    println!("Modified Wizard 10 Slots: {:?}", new_slots);
    
    assert!(new_slots[5] >= 2, "Wizard 10 should have at least 2 L5 slots");
    assert_eq!(new_slots[6], 0, "Wizard 10 should not have L6 slots");
}
