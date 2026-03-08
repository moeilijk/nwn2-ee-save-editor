use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

pub static ABILITY_MAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "Str");
    m.insert(1, "Dex");
    m.insert(2, "Con");
    m.insert(3, "Int");
    m.insert(4, "Wis");
    m.insert(5, "Cha");
    m
});

pub static SAVE_MAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "Fortitude");
    m.insert(1, "Reflex");
    m.insert(2, "Will");
    m
});

pub static DAMAGE_TYPE_MAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "Bludgeoning");
    m.insert(1, "Piercing");
    m.insert(2, "Slashing");
    m.insert(3, "Subdual");
    m.insert(4, "Physical");
    m.insert(5, "Magical");
    m.insert(6, "Acid");
    m.insert(7, "Cold");
    m.insert(8, "Divine");
    m.insert(9, "Electrical");
    m.insert(10, "Fire");
    m.insert(11, "Negative");
    m.insert(12, "Positive");
    m.insert(13, "Sonic");
    m
});

pub static IMMUNITY_TYPE_MAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "Backstab");
    m.insert(1, "Level/Ability Drain");
    m.insert(2, "Mind-Affecting Spells");
    m.insert(3, "Poison");
    m.insert(4, "Disease");
    m.insert(5, "Fear");
    m.insert(6, "Knockdown");
    m.insert(7, "Paralysis");
    m.insert(8, "Critical Hits");
    m.insert(9, "Death Magic");
    m
});

pub static SAVE_ELEMENT_MAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "Universal");
    m.insert(1, "Acid");
    m.insert(2, "Backstab");
    m.insert(3, "Cold");
    m.insert(4, "Death");
    m.insert(5, "Disease");
    m.insert(6, "Divine");
    m.insert(7, "Electrical");
    m.insert(8, "Fear");
    m.insert(9, "Fire");
    m.insert(10, "Illusion");
    m.insert(11, "Mind-Affecting");
    m.insert(12, "Negative Energy");
    m.insert(13, "Poison");
    m.insert(14, "Positive Energy");
    m.insert(15, "Sonic");
    m.insert(16, "Traps");
    m.insert(17, "Spells");
    m.insert(18, "Law");
    m.insert(19, "Chaos");
    m.insert(20, "Good");
    m.insert(21, "Evil");
    m
});

pub static AC_TYPE_MAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "Dodge");
    m.insert(1, "Natural");
    m.insert(2, "Armor");
    m.insert(3, "Shield");
    m.insert(4, "Deflection");
    m
});

pub static ALIGNMENT_GROUP_MAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "Good");
    m.insert(1, "Evil");
    m.insert(2, "Lawful");
    m.insert(3, "Chaotic");
    m
});

pub static ALIGNMENT_MAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "Lawful Good");
    m.insert(1, "Neutral Good");
    m.insert(2, "Chaotic Good");
    m.insert(3, "Lawful Neutral");
    m.insert(4, "True Neutral");
    m.insert(5, "Chaotic Neutral");
    m.insert(6, "Lawful Evil");
    m.insert(7, "Neutral Evil");
    m.insert(8, "Chaotic Evil");
    m
});

pub static LIGHT_MAP: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(0, "Dim (5m)");
    m.insert(1, "Bright (5m)");
    m.insert(2, "Dim (10m)");
    m.insert(3, "Bright (10m)");
    m.insert(4, "Dim (15m)");
    m.insert(5, "Bright (15m)");
    m.insert(6, "Dim (20m)");
    m.insert(7, "Bright (20m)");
    m
});

pub static LABEL_CLEANUPS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("TN", "True Neutral");
    m.insert("AC_Natural", "Natural Armor");
    m.insert("AC_Dodge", "Dodge");
    m.insert("AC_Deflection", "Deflection");
    m.insert("AC_Armor", "Armor");
    m.insert("AC_Shield", "Shield");
    m.insert("Bonus_1", "+1");
    m.insert("Bonus_2", "+2");
    m.insert("Bonus_3", "+3");
    m.insert("Bonus_4", "+4");
    m.insert("Bonus_5", "+5");
    m.insert("Bonus_6", "+6");
    m.insert("Bonus_7", "+7");
    m.insert("Bonus_8", "+8");
    m.insert("Bonus_9", "+9");
    m.insert("Bonus_10", "+10");
    m.insert("Bonus_11", "+11");
    m.insert("Bonus_12", "+12");
    m.insert("Bonus_14", "DC 14");
    m.insert("Bonus_15", "DC 15");
    m.insert("Bonus_16", "DC 16");
    m.insert("Bonus_17", "DC 17");
    m.insert("Bonus_18", "DC 18");
    m.insert("Bonus_19", "DC 19");
    m.insert("Bonus_20", "DC 20");
    m.insert("1d4", "1d4");
    m.insert("1d6", "1d6");
    m.insert("1d8", "1d8");
    m.insert("1d10", "1d10");
    m.insert("2d6", "2d6");
    m.insert("2d8", "2d8");
    m.insert("2d10", "2d10");
    m
});

pub static INVALID_LABEL_PATTERNS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "****",
        "**",
        "DEL_",
        "DELETED",
        "padding",
        "None",
        "REMOVED",
        "TEST",
        "INVALID",
        "RESERVED",
        "unused",
        "placeholder",
    ]
});

pub static HIDDEN_PROPERTY_IDS: LazyLock<HashSet<u32>> = LazyLock::new(|| {
    let mut s = HashSet::new();
    s.insert(30);
    s.insert(47);
    s.insert(72);
    s.insert(76);
    s.insert(77);
    s.insert(79);
    s.insert(82);
    s.insert(83);
    s.insert(90);
    s.insert(92);
    s.insert(94);
    s.insert(96);
    s.insert(100);
    s
});

pub static PROPERTY_LABEL_OVERRIDES: LazyLock<HashMap<u32, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(81, "Weight Modifier (Lbs)");
    m.insert(15, "Cast Spell");
    m.insert(16, "Damage Bonus");
    m.insert(23, "Damage Resistance");
    m.insert(24, "Damage Vulnerability");
    m
});

pub fn get_ability_name(id: u32) -> &'static str {
    ABILITY_MAP.get(&id).copied().unwrap_or("Unknown")
}

pub fn get_save_name(id: u32) -> &'static str {
    SAVE_MAP.get(&id).copied().unwrap_or("Unknown")
}

pub fn get_damage_type_name(id: u32) -> &'static str {
    DAMAGE_TYPE_MAP.get(&id).copied().unwrap_or("Unknown")
}

pub fn get_immunity_type_name(id: u32) -> &'static str {
    IMMUNITY_TYPE_MAP.get(&id).copied().unwrap_or("Unknown")
}

pub fn get_save_element_name(id: u32) -> &'static str {
    SAVE_ELEMENT_MAP.get(&id).copied().unwrap_or("Unknown")
}

pub fn get_ac_type_name(id: u32) -> &'static str {
    AC_TYPE_MAP.get(&id).copied().unwrap_or("Unknown")
}

pub fn get_alignment_group_name(id: u32) -> &'static str {
    ALIGNMENT_GROUP_MAP.get(&id).copied().unwrap_or("Unknown")
}

pub fn get_alignment_name(id: u32) -> &'static str {
    ALIGNMENT_MAP.get(&id).copied().unwrap_or("Unknown")
}

pub fn get_light_name(id: u32) -> &'static str {
    LIGHT_MAP.get(&id).copied().unwrap_or("Unknown")
}

pub fn is_invalid_label(label: &str) -> bool {
    let label_upper = label.to_uppercase();
    INVALID_LABEL_PATTERNS
        .iter()
        .any(|pattern| label_upper.contains(&pattern.to_uppercase()))
}

pub fn clean_label(label: &str) -> String {
    if let Some(&cleaned) = LABEL_CLEANUPS.get(label) {
        return cleaned.to_string();
    }

    label
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn is_hidden_property(id: u32) -> bool {
    HIDDEN_PROPERTY_IDS.contains(&id)
}

pub fn get_property_label_override(id: u32) -> Option<&'static str> {
    PROPERTY_LABEL_OVERRIDES.get(&id).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_map() {
        assert_eq!(get_ability_name(0), "Str");
        assert_eq!(get_ability_name(5), "Cha");
        assert_eq!(get_ability_name(99), "Unknown");
    }

    #[test]
    fn test_label_cleanup() {
        assert_eq!(clean_label("AC_Natural"), "Natural Armor");
        assert_eq!(clean_label("Some_Thing"), "Some Thing");
    }

    #[test]
    fn test_invalid_label_detection() {
        assert!(is_invalid_label("****"));
        assert!(is_invalid_label("DEL_Something"));
        assert!(is_invalid_label("DELETED"));
        assert!(!is_invalid_label("Valid Label"));
    }

    #[test]
    fn test_hidden_properties() {
        assert!(is_hidden_property(30));
        assert!(is_hidden_property(47));
        assert!(!is_hidden_property(0));
        assert!(!is_hidden_property(1));
    }
}
