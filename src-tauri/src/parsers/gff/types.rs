use std::borrow::Cow;
use std::sync::Arc;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::parser::GffParser;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GffFieldType {
    Byte = 0,
    Char = 1,
    Word = 2,
    Short = 3,
    Dword = 4,
    Int = 5,
    Dword64 = 6,
    Int64 = 7,
    Float = 8,
    Double = 9,
    String = 10,
    ResRef = 11,
    LocString = 12,
    Void = 13,
    Struct = 14,
    List = 15,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedSubstring<'a> {
    pub string: Cow<'a, str>,
    pub language: u32,
    pub gender: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedString<'a> {
    pub string_ref: i32,
    pub substrings: Vec<LocalizedSubstring<'a>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum GffValue<'a> {
    Byte(u8),
    Char(char),
    Word(u16),
    Short(i16),
    Dword(u32),
    Int(i32),
    Dword64(u64),
    Int64(i64),
    Float(f32),
    Double(f64),
    String(Cow<'a, str>),
    ResRef(Cow<'a, str>),
    LocString(LocalizedString<'a>),

    #[serde(with = "serde_bytes")]
    Void(Cow<'a, [u8]>),

    Struct(LazyStruct),
    List(Vec<LazyStruct>),

    StructOwned(Box<IndexMap<String, GffValue<'a>>>),
    ListOwned(Vec<IndexMap<String, GffValue<'a>>>),

    StructRef(u32),
    ListRef(Vec<u32>),
}

use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct LazyStruct {
    pub parser: Arc<GffParser>,
    pub struct_index: u32,
    pub struct_id: u32,
    pub cached_fields: Arc<RwLock<Option<IndexMap<String, GffValue<'static>>>>>,
}

impl Serialize for LazyStruct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let fields = self.force_load();
        let mut map = serializer.serialize_map(Some(fields.len()))?;
        for (k, v) in fields {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl LazyStruct {
    pub fn new(parser: Arc<GffParser>, struct_index: u32, struct_id: u32) -> Self {
        Self {
            parser,
            struct_index,
            struct_id,
            cached_fields: Arc::new(RwLock::new(None)),
        }
    }

    pub fn force_load(&self) -> IndexMap<String, GffValue<'static>> {
        if let Ok(guard) = self.cached_fields.read()
            && let Some(fields) = &*guard
        {
            return fields.clone();
        }

        let fields = self
            .parser
            .read_struct_fields(self.struct_index)
            .unwrap_or_default();
        let mut owned_fields: IndexMap<String, GffValue<'static>> = fields
            .into_iter()
            .map(|(k, v)| (k, v.into_owned()))
            .collect();

        // Inject struct_id as it is metadata required for properly identifying list items (e.g. Equip_ItemList slot)
        owned_fields.insert("__struct_id__".to_string(), GffValue::Dword(self.struct_id));

        if let Ok(mut guard) = self.cached_fields.write() {
            *guard = Some(owned_fields.clone());
        }
        owned_fields
    }
}

impl GffValue<'_> {
    pub fn into_owned(self) -> GffValue<'static> {
        match self {
            GffValue::Byte(v) => GffValue::Byte(v),
            GffValue::Char(v) => GffValue::Char(v),
            GffValue::Word(v) => GffValue::Word(v),
            GffValue::Short(v) => GffValue::Short(v),
            GffValue::Dword(v) => GffValue::Dword(v),
            GffValue::Int(v) => GffValue::Int(v),
            GffValue::Dword64(v) => GffValue::Dword64(v),
            GffValue::Int64(v) => GffValue::Int64(v),
            GffValue::Float(v) => GffValue::Float(v),
            GffValue::Double(v) => GffValue::Double(v),
            GffValue::String(cow) => GffValue::String(Cow::Owned(cow.into_owned())),
            GffValue::ResRef(cow) => GffValue::ResRef(Cow::Owned(cow.into_owned())),
            GffValue::LocString(ls) => {
                let owned_substrings = ls
                    .substrings
                    .into_iter()
                    .map(|sub| LocalizedSubstring {
                        string: Cow::Owned(sub.string.into_owned()),
                        language: sub.language,
                        gender: sub.gender,
                    })
                    .collect();
                GffValue::LocString(LocalizedString {
                    string_ref: ls.string_ref,
                    substrings: owned_substrings,
                })
            }
            GffValue::Void(cow) => GffValue::Void(Cow::Owned(cow.into_owned())),
            GffValue::Struct(lazy) => GffValue::Struct(lazy),
            GffValue::List(vec) => GffValue::List(vec),
            GffValue::StructOwned(map) => {
                let owned_map = map.into_iter().map(|(k, v)| (k, v.into_owned())).collect();
                GffValue::StructOwned(Box::new(owned_map))
            }
            GffValue::ListOwned(vec) => {
                let owned_vec = vec
                    .into_iter()
                    .map(|map| map.into_iter().map(|(k, v)| (k, v.into_owned())).collect())
                    .collect();
                GffValue::ListOwned(owned_vec)
            }
            GffValue::StructRef(idx) => GffValue::StructRef(idx),
            GffValue::ListRef(vec) => GffValue::ListRef(vec),
        }
    }

    /// Recursively convert all lazy-loaded structures to fully owned data.
    /// This eliminates all Arc<RwLock<>> from nested LazyStruct values,
    /// making the resulting data suitable for direct ownership without locks.
    pub fn force_owned(self) -> GffValue<'static> {
        match self {
            GffValue::Byte(v) => GffValue::Byte(v),
            GffValue::Char(v) => GffValue::Char(v),
            GffValue::Word(v) => GffValue::Word(v),
            GffValue::Short(v) => GffValue::Short(v),
            GffValue::Dword(v) => GffValue::Dword(v),
            GffValue::Int(v) => GffValue::Int(v),
            GffValue::Dword64(v) => GffValue::Dword64(v),
            GffValue::Int64(v) => GffValue::Int64(v),
            GffValue::Float(v) => GffValue::Float(v),
            GffValue::Double(v) => GffValue::Double(v),
            GffValue::String(cow) => GffValue::String(Cow::Owned(cow.into_owned())),
            GffValue::ResRef(cow) => GffValue::ResRef(Cow::Owned(cow.into_owned())),
            GffValue::LocString(ls) => {
                let owned_substrings = ls
                    .substrings
                    .into_iter()
                    .map(|sub| LocalizedSubstring {
                        string: Cow::Owned(sub.string.into_owned()),
                        language: sub.language,
                        gender: sub.gender,
                    })
                    .collect();
                GffValue::LocString(LocalizedString {
                    string_ref: ls.string_ref,
                    substrings: owned_substrings,
                })
            }
            GffValue::Void(cow) => GffValue::Void(Cow::Owned(cow.into_owned())),
            GffValue::Struct(lazy) => {
                let fields = lazy.force_load();
                let owned_fields: IndexMap<String, GffValue<'static>> = fields
                    .into_iter()
                    .map(|(k, v)| (k, v.force_owned()))
                    .collect();
                GffValue::StructOwned(Box::new(owned_fields))
            }
            GffValue::List(lazy_vec) => {
                let owned_vec: Vec<IndexMap<String, GffValue<'static>>> = lazy_vec
                    .into_iter()
                    .map(|lazy| {
                        lazy.force_load()
                            .into_iter()
                            .map(|(k, v)| (k, v.force_owned()))
                            .collect()
                    })
                    .collect();
                GffValue::ListOwned(owned_vec)
            }
            GffValue::StructOwned(map) => {
                let owned_map: IndexMap<String, GffValue<'static>> =
                    map.into_iter().map(|(k, v)| (k, v.force_owned())).collect();
                GffValue::StructOwned(Box::new(owned_map))
            }
            GffValue::ListOwned(vec) => {
                let owned_vec: Vec<IndexMap<String, GffValue<'static>>> = vec
                    .into_iter()
                    .map(|map| map.into_iter().map(|(k, v)| (k, v.force_owned())).collect())
                    .collect();
                GffValue::ListOwned(owned_vec)
            }
            GffValue::StructRef(idx) => GffValue::StructRef(idx),
            GffValue::ListRef(vec) => GffValue::ListRef(vec),
        }
    }
}
