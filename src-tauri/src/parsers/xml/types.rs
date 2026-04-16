use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize)]
pub struct XmlData {
    pub integers: IndexMap<String, i32>,
    pub booleans: IndexMap<String, i32>,
    pub floats: IndexMap<String, f32>,
    pub strings: IndexMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IntegerEntry {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: i32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct IntegersWrapper {
    #[serde(rename = "Integer", default)]
    pub entries: Vec<IntegerEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BooleanEntry {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: i32,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BooleansWrapper {
    #[serde(rename = "Boolean", default)]
    pub entries: Vec<BooleanEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FloatEntry {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value", serialize_with = "serialize_f32_fixed")]
    pub value: f32,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn serialize_f32_fixed<S: serde::Serializer>(v: &f32, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&format!("{v:.6}"))
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FloatsWrapper {
    #[serde(rename = "Float", default)]
    pub entries: Vec<FloatEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StringEntry {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct StringsWrapper {
    #[serde(rename = "String", default)]
    pub entries: Vec<StringEntry>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename = "Globals")]
pub struct GlobalsXml {
    #[serde(rename = "Integers", default)]
    pub integers: IntegersWrapper,
    #[serde(rename = "Booleans", default)]
    pub booleans: BooleansWrapper,
    #[serde(rename = "Floats", default)]
    pub floats: FloatsWrapper,
    #[serde(rename = "Strings", default)]
    pub strings: StringsWrapper,
}

impl XmlData {
    pub fn from_xml_struct(xml: GlobalsXml) -> Self {
        let mut data = XmlData::default();

        for entry in xml.integers.entries {
            data.integers.insert(entry.name, entry.value);
        }
        for entry in xml.booleans.entries {
            data.booleans.insert(entry.name, entry.value);
        }
        for entry in xml.floats.entries {
            data.floats.insert(entry.name, entry.value);
        }
        for entry in xml.strings.entries {
            data.strings.insert(entry.name, entry.value);
        }
        data
    }

    pub fn to_xml_struct(&self) -> GlobalsXml {
        let integers = self
            .integers
            .iter()
            .map(|(k, v)| IntegerEntry {
                name: k.clone(),
                value: *v,
            })
            .collect();

        let booleans = self
            .booleans
            .iter()
            .map(|(k, v)| BooleanEntry {
                name: k.clone(),
                value: *v,
            })
            .collect();

        let floats = self
            .floats
            .iter()
            .map(|(k, v)| FloatEntry {
                name: k.clone(),
                value: *v,
            })
            .collect();

        let strings = self
            .strings
            .iter()
            .map(|(k, v)| StringEntry {
                name: k.clone(),
                value: v.clone(),
            })
            .collect();

        GlobalsXml {
            integers: IntegersWrapper { entries: integers },
            booleans: BooleansWrapper { entries: booleans },
            floats: FloatsWrapper { entries: floats },
            strings: StringsWrapper { entries: strings },
        }
    }
}
