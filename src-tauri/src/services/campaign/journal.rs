use crate::parsers::gff::{GffParser, GffValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestDefinition {
    pub category_tag: String,
    pub category_name: String,
    pub entry_id: u32,
    pub text: String,
    pub xp: u32,
    pub end: bool,
    pub source: String,
}

pub fn parse_journal_gff(
    data: &[u8],
    source: &str,
) -> Result<HashMap<String, QuestDefinition>, String> {
    let gff = GffParser::from_bytes(data.to_vec())
        .map_err(|e| format!("Failed to parse journal GFF: {e}"))?;
    let root = gff
        .read_struct_fields(0)
        .map_err(|e| format!("Failed to read root struct: {e}"))?;

    let mut definitions = HashMap::new();

    if let Some(GffValue::List(categories)) = root.get("Categories") {
        for category in categories {
            let cat_fields = category.force_load();
            let tag = match cat_fields.get("Tag") {
                Some(GffValue::String(s)) => s.to_string(),
                _ => continue,
            };

            let name = match cat_fields.get("Name") {
                Some(GffValue::LocString(ls)) => ls
                    .substrings
                    .first()
                    .map_or_else(|| tag.clone(), |sub| sub.string.to_string()),
                Some(GffValue::String(s)) => s.to_string(),
                _ => tag.clone(),
            };

            if let Some(GffValue::List(entries)) = cat_fields.get("EntryList") {
                for entry in entries {
                    let entry_fields = entry.force_load();
                    let id = match entry_fields.get("ID") {
                        Some(GffValue::Dword(v)) => *v,
                        Some(GffValue::Int(v)) => *v as u32,
                        _ => continue,
                    };

                    let text = match entry_fields.get("Text") {
                        Some(GffValue::LocString(ls)) => ls
                            .substrings
                            .first()
                            .map(|sub| sub.string.to_string())
                            .unwrap_or_default(),
                        Some(GffValue::String(s)) => s.to_string(),
                        _ => String::new(),
                    };

                    let xp = match entry_fields.get("XP") {
                        Some(GffValue::Dword(v)) => *v,
                        Some(GffValue::Int(v)) => *v as u32,
                        _ => 0,
                    };

                    let end = match entry_fields.get("End") {
                        Some(GffValue::Byte(v)) => *v != 0,
                        Some(GffValue::Int(v)) => *v != 0,
                        _ => false,
                    };

                    let key = format!("{tag}_{id}");
                    definitions.insert(
                        key,
                        QuestDefinition {
                            category_tag: tag.clone(),
                            category_name: name.clone(),
                            entry_id: id,
                            text,
                            xp,
                            end,
                            source: source.to_string(),
                        },
                    );
                }
            }
        }
    }

    Ok(definitions)
}
