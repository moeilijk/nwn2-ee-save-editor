use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use app_lib::parsers::{
    erf::ErfParser,
    gff::{GffParser, GffValue},
};
use indexmap::IndexMap;
use xz2::read::XzDecoder;
use xz2::stream::Stream;

fn walk_value(path: &str, value: &GffValue<'_>, target_feats: &[i64], hits: &mut Vec<String>) {
    match value {
        GffValue::Word(v) => {
            if path.contains("FeatList") && target_feats.contains(&i64::from(*v)) {
                hits.push(format!("{path} = WORD({v})"));
            }
        }
        GffValue::Short(v) => {
            if path.contains("FeatList") && target_feats.contains(&i64::from(*v)) {
                hits.push(format!("{path} = SHORT({v})"));
            }
        }
        GffValue::Dword(v) => {
            if path.contains("FeatList") && target_feats.contains(&i64::from(*v)) {
                hits.push(format!("{path} = DWORD({v})"));
            }
        }
        GffValue::Int(v) => {
            if path.contains("FeatList") && target_feats.contains(&i64::from(*v)) {
                hits.push(format!("{path} = INT({v})"));
            }
        }
        GffValue::StructOwned(fields) => walk_fields(path, fields, target_feats, hits),
        GffValue::ListOwned(entries) => {
            for (idx, entry) in entries.iter().enumerate() {
                let child = format!("{path}[{idx}]");
                walk_fields(&child, entry, target_feats, hits);
            }
        }
        GffValue::Struct(lazy) => {
            let fields = lazy.force_load();
            walk_fields(path, &fields, target_feats, hits);
        }
        GffValue::List(entries) => {
            for (idx, entry) in entries.iter().enumerate() {
                let child = format!("{path}[{idx}]");
                let fields = entry.force_load();
                walk_fields(&child, &fields, target_feats, hits);
            }
        }
        _ => {}
    }
}

fn walk_fields(
    path: &str,
    fields: &IndexMap<String, GffValue<'_>>,
    target_feats: &[i64],
    hits: &mut Vec<String>,
) {
    for (key, value) in fields {
        let child = if path.is_empty() {
            key.clone()
        } else {
            format!("{path}.{key}")
        };
        walk_value(&child, value, target_feats, hits);
    }
}

fn load_archive(path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("z"))
    {
        let file = fs::File::open(path)?;
        let stream = Stream::new_lzma_decoder(u64::MAX)?;
        let mut decoder = XzDecoder::new_stream(file, stream);
        let mut out = Vec::new();
        decoder.read_to_end(&mut out)?;
        Ok(out)
    } else {
        Ok(fs::read(path)?)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let save_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: scan_save_feat_paths <save-dir> <feat-id> [feat-id..]")?;
    let target_feats: Vec<i64> = std::env::args()
        .skip(2)
        .map(|s| s.parse::<i64>())
        .collect::<Result<_, _>>()?;

    for entry in fs::read_dir(&save_dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        if !matches!(ext.to_ascii_lowercase().as_str(), "z" | "zip" | "bic" | "ifo" | "ros" | "rst")
        {
            continue;
        }

        if ext.eq_ignore_ascii_case("zip") {
            let file = fs::File::open(&path)?;
            let mut zip = zip::ZipArchive::new(file)?;
            for idx in 0..zip.len() {
                let mut member = zip.by_index(idx)?;
                let name = member.name().to_string();
                let mut data = Vec::new();
                member.read_to_end(&mut data)?;
                let Ok(parser) = GffParser::from_bytes(data) else {
                    continue;
                };
                let Ok(fields) = parser.read_struct_fields(0) else {
                    continue;
                };
                let mut hits = Vec::new();
                walk_fields("", &fields, &target_feats, &mut hits);
                if !hits.is_empty() {
                    println!("archive={} member={}", path.display(), name);
                    for hit in hits {
                        println!("  {hit}");
                    }
                }
            }
            continue;
        }

        let archive_data = load_archive(&path)?;
        if ext.eq_ignore_ascii_case("z") {
            let mut erf = ErfParser::new();
            erf.parse_from_bytes(&archive_data)?;
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
                let mut hits = Vec::new();
                walk_fields("", &fields, &target_feats, &mut hits);
                if !hits.is_empty() {
                    println!("archive={} resource={}", path.display(), resource_name);
                    for hit in hits {
                        println!("  {hit}");
                    }
                }
            }
        } else if let Ok(parser) = GffParser::from_bytes(archive_data)
            && let Ok(fields) = parser.read_struct_fields(0)
        {
            let mut hits = Vec::new();
            walk_fields("", &fields, &target_feats, &mut hits);
            if !hits.is_empty() {
                println!("file={}", path.display());
                for hit in hits {
                    println!("  {hit}");
                }
            }
        }
    }

    Ok(())
}
