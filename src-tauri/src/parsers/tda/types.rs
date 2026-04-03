use ahash::AHashMap;
use lasso::{Spur, ThreadedRodeo};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use super::error::{SecurityLimits, TDAError, TDAResult};

pub type Symbol = Spur;
pub type TDAStringInterner = ThreadedRodeo;

#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    Interned(Symbol),
    Raw(String),
    Null,
    Empty,
}

impl CellValue {
    pub fn new(value: &str, interner: &mut TDAStringInterner) -> Self {
        match value {
            "" => Self::Empty,
            "****" => Self::Null,
            _ => {
                if value.len() <= 32 && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                {
                    Self::Interned(interner.get_or_intern(value))
                } else {
                    Self::Raw(value.to_string())
                }
            }
        }
    }

    pub fn as_str<'a>(&'a self, interner: &'a TDAStringInterner) -> Option<&'a str> {
        match self {
            Self::Interned(symbol) => Some(interner.resolve(symbol)),
            Self::Raw(string) => Some(string),
            Self::Null => None,
            Self::Empty => Some(""),
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

pub type TDARow = SmallVec<[CellValue; 16]>;

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: Symbol,
    pub index: usize,
}

#[derive(Debug)]
pub struct TDAParser {
    interner: TDAStringInterner,
    columns: Vec<ColumnInfo>,
    column_map: AHashMap<String, usize>,
    rows: Vec<TDARow>,
    security_limits: SecurityLimits,
    metadata: TDAMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TDAMetadata {
    pub file_size: usize,
    pub line_count: usize,
    pub parse_time_ns: u64,
    pub has_warnings: bool,
    pub format_version: String,
}

impl Default for TDAMetadata {
    fn default() -> Self {
        Self {
            file_size: 0,
            line_count: 0,
            parse_time_ns: 0,
            has_warnings: false,
            format_version: "2DA V2.0".to_string(),
        }
    }
}

impl TDAParser {
    pub fn new() -> Self {
        Self::with_limits(SecurityLimits::default())
    }

    pub fn with_limits(limits: SecurityLimits) -> Self {
        Self {
            interner: TDAStringInterner::default(),
            columns: Vec::new(),
            column_map: AHashMap::new(),
            rows: Vec::new(),
            security_limits: limits,
            metadata: TDAMetadata::default(),
        }
    }

    #[cfg(test)]
    pub fn add_column(&mut self, name: &str) {
        let index = self.columns.len();
        let symbol = self.interner.get_or_intern(name);
        self.columns.push(ColumnInfo {
            name: symbol,
            index,
        });
        self.column_map.insert(name.to_lowercase(), index);
    }

    #[cfg(test)]
    pub fn add_row(&mut self, values: AHashMap<String, Option<String>>) {
        let mut row = TDARow::new();
        for column in &self.columns {
            let col_name = self.interner.resolve(&column.name);
            let val = values.get(col_name).and_then(|v| v.as_deref());
            let cell = match val {
                Some(s) => CellValue::new(s, &mut self.interner),
                None => CellValue::Null,
            };
            row.push(cell);
        }
        self.rows.push(row);
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn column_names(&self) -> Vec<&str> {
        self.columns
            .iter()
            .map(|col| self.interner.resolve(&col.name))
            .collect()
    }

    pub fn find_column_index(&self, name: &str) -> Option<usize> {
        self.column_map.get(&name.to_lowercase()).copied()
    }

    pub fn get_cell(&self, row_index: usize, col_index: usize) -> TDAResult<Option<&str>> {
        let row = self
            .rows
            .get(row_index)
            .ok_or(TDAError::RowIndexOutOfBounds {
                index: row_index,
                max: self.rows.len(),
            })?;

        let cell = row.get(col_index).ok_or(TDAError::ColumnIndexOutOfBounds {
            index: col_index,
            max: row.len(),
        })?;

        Ok(cell.as_str(&self.interner))
    }

    pub fn get_cell_by_name(&self, row_index: usize, column_name: &str) -> TDAResult<Option<&str>> {
        let col_index =
            self.find_column_index(column_name)
                .ok_or_else(|| TDAError::ColumnNotFound {
                    column: column_name.to_string(),
                })?;

        self.get_cell(row_index, col_index)
    }

    pub fn get_row_dict(&self, row_index: usize) -> TDAResult<AHashMap<String, Option<String>>> {
        let row = self
            .rows
            .get(row_index)
            .ok_or(TDAError::RowIndexOutOfBounds {
                index: row_index,
                max: self.rows.len(),
            })?;

        let mut result = AHashMap::with_capacity(self.columns.len());

        for (col_info, cell) in self.columns.iter().zip(row.iter()) {
            let col_name = self.interner.resolve(&col_info.name);
            let value = cell
                .as_str(&self.interner)
                .map(std::string::ToString::to_string);
            result.insert(col_name.to_lowercase(), value);
        }

        Ok(result)
    }

    pub fn get_all_rows_dict(&self) -> Vec<AHashMap<String, Option<String>>> {
        let col_names: Vec<String> = self
            .columns
            .iter()
            .map(|col_info| self.interner.resolve(&col_info.name).to_lowercase())
            .collect();

        self.rows
            .iter()
            .map(|row| {
                let mut result = AHashMap::with_capacity(self.columns.len());
                for (col_name, cell) in col_names.iter().zip(row.iter()) {
                    let value = cell
                        .as_str(&self.interner)
                        .map(std::string::ToString::to_string);
                    result.insert(col_name.clone(), value);
                }
                result
            })
            .collect()
    }

    pub fn find_row(&self, column_name: &str, value: &str) -> TDAResult<Option<usize>> {
        let col_index =
            self.find_column_index(column_name)
                .ok_or_else(|| TDAError::ColumnNotFound {
                    column: column_name.to_string(),
                })?;

        for (row_index, row) in self.rows.iter().enumerate() {
            if let Some(cell) = row.get(col_index)
                && let Some(cell_value) = cell.as_str(&self.interner)
                && cell_value == value
            {
                return Ok(Some(row_index));
            }
        }

        Ok(None)
    }

    pub fn metadata(&self) -> &TDAMetadata {
        &self.metadata
    }

    pub(crate) fn metadata_mut(&mut self) -> &mut TDAMetadata {
        &mut self.metadata
    }

    pub fn security_limits(&self) -> &SecurityLimits {
        &self.security_limits
    }

    pub fn security_limits_mut(&mut self) -> &mut SecurityLimits {
        &mut self.security_limits
    }

    pub(crate) fn columns(&self) -> &Vec<ColumnInfo> {
        &self.columns
    }

    pub(crate) fn columns_mut(&mut self) -> &mut Vec<ColumnInfo> {
        &mut self.columns
    }

    pub(crate) fn column_map_mut(&mut self) -> &mut AHashMap<String, usize> {
        &mut self.column_map
    }

    pub(crate) fn rows_mut(&mut self) -> &mut Vec<TDARow> {
        &mut self.rows
    }

    pub(crate) fn rows(&self) -> &Vec<TDARow> {
        &self.rows
    }

    pub(crate) fn interner_mut(&mut self) -> &mut TDAStringInterner {
        &mut self.interner
    }

    pub(crate) fn interner(&self) -> &TDAStringInterner {
        &self.interner
    }

    pub fn clear(&mut self) {
        self.interner = TDAStringInterner::default();
        self.columns.clear();
        self.column_map.clear();
        self.rows.clear();
        self.metadata = TDAMetadata::default();
    }

    pub fn memory_usage(&self) -> usize {
        let interner_size = self.interner.len() * 32;
        let columns_size = self.columns.len() * std::mem::size_of::<ColumnInfo>();
        let column_map_size = self.column_map.len() * (32 + 8);
        let rows_size = self
            .rows
            .iter()
            .map(|row| row.len() * std::mem::size_of::<CellValue>())
            .sum::<usize>();

        interner_size + columns_size + column_map_size + rows_size
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = impl Iterator<Item = Option<&str>> + '_> + '_ {
        self.rows
            .iter()
            .map(move |row| row.iter().map(move |cell| cell.as_str(&self.interner)))
    }

    pub fn iter_column(&self, col_index: usize) -> impl Iterator<Item = Option<&str>> + '_ {
        self.rows.iter().map(move |row| {
            row.get(col_index)
                .and_then(|cell| cell.as_str(&self.interner))
        })
    }

    pub fn iter_column_by_name(
        &self,
        column_name: &str,
    ) -> Option<impl Iterator<Item = Option<&str>> + '_> {
        let col_index = self.find_column_index(column_name)?;
        Some(self.iter_column(col_index))
    }
}

impl Default for TDAParser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SerializableCellValue {
    String(String),
    Null,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableTDAParser {
    pub column_names: Vec<String>,
    pub rows: Vec<Vec<SerializableCellValue>>,
    pub security_limits: SecurityLimits,
    pub metadata: TDAMetadata,
}

impl SerializableTDAParser {
    pub fn from_parser(parser: &TDAParser) -> Self {
        let column_names = parser
            .column_names()
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect();

        let rows = parser
            .rows()
            .iter()
            .map(|row| {
                row.iter()
                    .map(|cell| match cell {
                        CellValue::Interned(symbol) => {
                            let s = parser.interner().resolve(symbol);
                            SerializableCellValue::String(s.to_string())
                        }
                        CellValue::Raw(s) => SerializableCellValue::String(s.clone()),
                        CellValue::Null => SerializableCellValue::Null,
                        CellValue::Empty => SerializableCellValue::String(String::new()),
                    })
                    .collect()
            })
            .collect();

        Self {
            column_names,
            rows,
            security_limits: SecurityLimits::default(),
            metadata: parser.metadata().clone(),
        }
    }

    pub fn to_parser(self) -> TDAParser {
        let mut parser = TDAParser::with_limits(self.security_limits);
        parser.metadata = self.metadata;

        for (idx, name) in self.column_names.into_iter().enumerate() {
            let symbol = parser.interner_mut().get_or_intern(&name);
            parser.columns_mut().push(ColumnInfo {
                name: symbol,
                index: idx,
            });
            parser.column_map_mut().insert(name.to_lowercase(), idx);
        }

        for row_data in self.rows {
            let mut row = TDARow::new();
            for cell in row_data {
                let cell_value = match cell {
                    SerializableCellValue::String(s) => CellValue::new(&s, parser.interner_mut()),
                    SerializableCellValue::Null => CellValue::Null,
                };
                row.push(cell_value);
            }
            parser.rows_mut().push(row);
        }

        parser
    }
}
