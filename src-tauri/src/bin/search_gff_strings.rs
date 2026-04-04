use std::path::PathBuf;

use app_lib::parsers::gff::{GffParser, GffValue};
use indexmap::IndexMap;

fn walk_value(path: &str, value: &GffValue<'_>, needle: &str) {
    match value {
        GffValue::String(s) | GffValue::ResRef(s) => {
            if s.to_ascii_lowercase().contains(&needle.to_ascii_lowercase()) {
                println!("{path} = {s}");
            }
        }
        GffValue::LocString(ls) => {
            if ls.string_ref >= 0 {
                println!("{path} = <locstring strref={}>", ls.string_ref);
            }
            for (idx, sub) in ls.substrings.iter().enumerate() {
                if sub.string.to_ascii_lowercase().contains(&needle.to_ascii_lowercase()) {
                    println!("{path}[{idx}] = {}", sub.string);
                }
            }
        }
        GffValue::StructOwned(fields) => walk_fields(path, fields, needle),
        GffValue::ListOwned(entries) => {
            for (idx, entry) in entries.iter().enumerate() {
                let child = format!("{path}[{idx}]");
                walk_fields(&child, entry, needle);
            }
        }
        GffValue::Struct(lazy) => {
            let fields = lazy.force_load();
            walk_fields(path, &fields, needle);
        }
        GffValue::List(entries) => {
            for (idx, entry) in entries.iter().enumerate() {
                let child = format!("{path}[{idx}]");
                let fields = entry.force_load();
                walk_fields(&child, &fields, needle);
            }
        }
        _ => {}
    }
}

fn walk_fields(path: &str, fields: &IndexMap<String, GffValue<'_>>, needle: &str) {
    for (key, value) in fields {
        let child = if path.is_empty() {
            key.clone()
        } else {
            format!("{path}.{key}")
        };
        walk_value(&child, value, needle);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: search_gff_strings <path> <needle>")?;
    let needle = std::env::args()
        .nth(2)
        .ok_or("usage: search_gff_strings <path> <needle>")?;

    let data = std::fs::read(&path)?;
    let parser = GffParser::from_bytes(data)?;
    let fields = parser.read_struct_fields(0)?;
    walk_fields("", &fields, &needle);
    Ok(())
}
