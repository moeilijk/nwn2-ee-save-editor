use std::path::PathBuf;

use app_lib::parsers::gff::{GffParser, GffValue};
use indexmap::IndexMap;

fn walk_value(path: &str, value: &GffValue<'_>) {
    match value {
        GffValue::Byte(v) => println!("{path} = BYTE({v})"),
        GffValue::Char(v) => println!("{path} = CHAR({})", *v as u8),
        GffValue::Word(v) => println!("{path} = WORD({v})"),
        GffValue::Short(v) => println!("{path} = SHORT({v})"),
        GffValue::Dword(v) => println!("{path} = DWORD({v})"),
        GffValue::Int(v) => println!("{path} = INT({v})"),
        GffValue::Dword64(v) => println!("{path} = DWORD64({v})"),
        GffValue::Int64(v) => println!("{path} = INT64({v})"),
        GffValue::Float(v) => println!("{path} = FLOAT({v})"),
        GffValue::Double(v) => println!("{path} = DOUBLE({v})"),
        GffValue::String(s) => println!("{path} = STRING({s})"),
        GffValue::ResRef(s) => println!("{path} = RESREF({s})"),
        GffValue::LocString(ls) => {
            println!("{path} = LOCSTR(ref={})", ls.string_ref);
            for (idx, sub) in ls.substrings.iter().enumerate() {
                println!(
                    "{path}[{idx}] = LOCSUB(lang={}, gender={}, text={})",
                    sub.language, sub.gender, sub.string
                );
            }
        }
        GffValue::Void(bytes) => println!("{path} = VOID(len={})", bytes.len()),
        GffValue::StructOwned(fields) => walk_fields(path, fields),
        GffValue::ListOwned(entries) => {
            for (idx, entry) in entries.iter().enumerate() {
                let child = format!("{path}[{idx}]");
                walk_fields(&child, entry);
            }
        }
        GffValue::Struct(lazy) => {
            let fields = lazy.force_load();
            walk_fields(path, &fields);
        }
        GffValue::List(entries) => {
            for (idx, entry) in entries.iter().enumerate() {
                let child = format!("{path}[{idx}]");
                let fields = entry.force_load();
                walk_fields(&child, &fields);
            }
        }
        other => println!("{path} = {other:?}"),
    }
}

fn walk_fields(path: &str, fields: &IndexMap<String, GffValue<'_>>) {
    for (key, value) in fields {
        let child = if path.is_empty() {
            key.clone()
        } else {
            format!("{path}.{key}")
        };
        walk_value(&child, value);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: dump_gff_flat <path>")?;
    let data = std::fs::read(path)?;
    let parser = GffParser::from_bytes(data)?;
    let fields = parser.read_struct_fields(0)?;
    walk_fields("", &fields);
    Ok(())
}
