use std::path::PathBuf;

use app_lib::parsers::gff::{GffParser, GffValue};
use indexmap::IndexMap;

fn walk_value(path: &str, value: &GffValue<'_>, needle: i64) {
    match value {
        GffValue::Byte(n) if i64::from(*n) == needle => println!("{path} = {}", n),
        GffValue::Char(n) if i64::from(u32::from(*n)) == needle => println!("{path} = {}", n),
        GffValue::Word(n) if i64::from(*n) == needle => println!("{path} = {}", n),
        GffValue::Short(n) if i64::from(*n) == needle => println!("{path} = {}", n),
        GffValue::Dword(n) if i64::from(*n) == needle => println!("{path} = {}", n),
        GffValue::Int(n) if i64::from(*n) == needle => println!("{path} = {}", n),
        GffValue::Dword64(n) if *n as i64 == needle => println!("{path} = {}", n),
        GffValue::Int64(n) if *n == needle => println!("{path} = {}", n),
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

fn walk_fields(path: &str, fields: &IndexMap<String, GffValue<'_>>, needle: i64) {
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
        .ok_or("usage: search_gff_number <path> <number>")?;
    let needle = std::env::args()
        .nth(2)
        .ok_or("usage: search_gff_number <path> <number>")?
        .parse::<i64>()?;

    let data = std::fs::read(&path)?;
    let parser = GffParser::from_bytes(data)?;
    let fields = parser.read_struct_fields(0)?;
    walk_fields("", &fields, needle);
    Ok(())
}
