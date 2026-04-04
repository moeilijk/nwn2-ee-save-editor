use std::path::PathBuf;

use app_lib::parsers::{
    erf::ErfParser,
    gff::{GffParser, GffValue},
};
use indexmap::IndexMap;
use std::fs;
use std::io::Read;
use xz2::read::XzDecoder;
use xz2::stream::Stream;

fn walk_value(path: &str, value: &GffValue<'_>, string_needle: Option<&str>, number_needle: Option<i64>) -> bool {
    let mut matched = false;
    match value {
        GffValue::String(s) | GffValue::ResRef(s) => {
            if let Some(needle) = string_needle {
                if s.to_ascii_lowercase().contains(&needle.to_ascii_lowercase()) {
                    println!("{path} = {s}");
                    matched = true;
                }
            }
        }
        GffValue::LocString(ls) => {
            if let Some(needle) = string_needle {
                for (idx, sub) in ls.substrings.iter().enumerate() {
                    if sub.string.to_ascii_lowercase().contains(&needle.to_ascii_lowercase()) {
                        println!("{path}[{idx}] = {}", sub.string);
                        matched = true;
                    }
                }
            }
        }
        GffValue::Byte(v) => {
            if number_needle == Some(i64::from(*v)) {
                println!("{path} = {v}");
                matched = true;
            }
        }
        GffValue::Char(v) => {
            if number_needle == Some(i64::from(*v as u8)) {
                println!("{path} = {}", *v as u8);
                matched = true;
            }
        }
        GffValue::Word(v) => {
            if number_needle == Some(i64::from(*v)) {
                println!("{path} = {v}");
                matched = true;
            }
        }
        GffValue::Short(v) => {
            if number_needle == Some(i64::from(*v)) {
                println!("{path} = {v}");
                matched = true;
            }
        }
        GffValue::Dword(v) => {
            if number_needle == Some(i64::from(*v)) {
                println!("{path} = {v}");
                matched = true;
            }
        }
        GffValue::Int(v) => {
            if number_needle == Some(i64::from(*v)) {
                println!("{path} = {v}");
                matched = true;
            }
        }
        GffValue::Dword64(v) => {
            if number_needle == Some(*v as i64) {
                println!("{path} = {v}");
                matched = true;
            }
        }
        GffValue::Int64(v) => {
            if number_needle == Some(*v) {
                println!("{path} = {v}");
                matched = true;
            }
        }
        GffValue::StructOwned(fields) => {
            matched |= walk_fields(path, fields, string_needle, number_needle);
        }
        GffValue::ListOwned(entries) => {
            for (idx, entry) in entries.iter().enumerate() {
                let child = format!("{path}[{idx}]");
                matched |= walk_fields(&child, entry, string_needle, number_needle);
            }
        }
        GffValue::Struct(lazy) => {
            let fields = lazy.force_load();
            matched |= walk_fields(path, &fields, string_needle, number_needle);
        }
        GffValue::List(entries) => {
            for (idx, entry) in entries.iter().enumerate() {
                let child = format!("{path}[{idx}]");
                let fields = entry.force_load();
                matched |= walk_fields(&child, &fields, string_needle, number_needle);
            }
        }
        _ => {}
    }
    matched
}

fn walk_fields(
    path: &str,
    fields: &IndexMap<String, GffValue<'_>>,
    string_needle: Option<&str>,
    number_needle: Option<i64>,
) -> bool {
    let mut matched = false;
    for (key, value) in fields {
        let child = if path.is_empty() {
            key.clone()
        } else {
            format!("{path}.{key}")
        };
        matched |= walk_value(&child, value, string_needle, number_needle);
    }
    matched
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: search_z_gff <path.z|path.mod> <needle>")?;
    let needle = std::env::args()
        .nth(2)
        .ok_or("usage: search_z_gff <path.z|path.mod> <needle>")?;

    let archive_data = if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("z"))
    {
        let file = fs::File::open(&path)?;
        let stream = Stream::new_lzma_decoder(u64::MAX)?;
        let mut decoder = XzDecoder::new_stream(file, stream);
        let mut out = Vec::new();
        decoder.read_to_end(&mut out)?;
        out
    } else {
        fs::read(&path)?
    };

    let mut erf = ErfParser::new();
    erf.parse_from_bytes(&archive_data)?;

    let string_needle = needle
        .parse::<i64>()
        .ok()
        .map(|_| None)
        .unwrap_or(Some(needle.as_str()));
    let number_needle = needle.parse::<i64>().ok();

    for (resource_name, _, _) in erf.list_resources(None) {
        let Ok(resource_data) = erf.extract_resource(&resource_name) else {
            continue;
        };
        let Ok(parser) = GffParser::from_bytes(resource_data) else {
            continue;
        };
        let Ok(fields) = parser.read_struct_fields(0) else {
            continue;
        };

        if walk_fields("", &fields, string_needle, number_needle) {
            println!("resource={resource_name}");
        }
    }

    Ok(())
}
