use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::time::Instant;

use ahash::AHashMap;
use memmap2::Mmap;
use tracing::{debug, info, instrument};

use super::error::{SecurityLimits, TDAError, TDAResult};
use super::tokenizer::{TDATokenizer, Token};
use super::types::{CellValue, ColumnInfo, SerializableTDAParser, TDAParser, TDARow};

impl TDAParser {
    #[instrument(name = "TDAParser::parse_from_bytes", skip_all, fields(size = data.len()))]
    pub fn parse_from_bytes(&mut self, data: &[u8]) -> TDAResult<()> {
        debug!("Parsing 2DA from byte buffer ({} bytes)", data.len());
        let start_time = Instant::now();

        self.security_limits().validate_file_size(data.len())?;

        self.clear();

        let content =
            std::str::from_utf8(data).map_err(|e| TDAError::InvalidUtf8 {
                position: e.valid_up_to(),
            })?;

        self.parse_content(content)?;

        self.metadata_mut().file_size = data.len();
        self.metadata_mut().parse_time_ns = start_time.elapsed().as_nanos() as u64;

        info!("2DA parsed: {} rows, {} columns, {:.2}ms",
              self.row_count(), self.column_count(),
              start_time.elapsed().as_secs_f64() * 1000.0);

        Ok(())
    }

    #[instrument(name = "TDAParser::parse_from_file", skip_all, fields(path = ?path.as_ref()))]
    pub fn parse_from_file<P: AsRef<Path>>(&mut self, path: P) -> TDAResult<()> {
        debug!("Opening 2DA file");
        let file = File::open(&path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len() as usize;

        debug!("2DA file size: {} bytes", file_size);
        self.security_limits().validate_file_size(file_size)?;

        if file_size > 64 * 1024 {
            debug!("Using memory-mapped parsing (file > 64KB)");
            self.parse_from_mmap(file)
        } else {
            debug!("Using buffered parsing (file <= 64KB)");
            let mut content = String::new();
            let mut reader = BufReader::new(file);
            reader.read_to_string(&mut content)?;
            self.parse_from_bytes(content.as_bytes())
        }
    }

    #[instrument(name = "TDAParser::parse_from_mmap", skip_all)]
    fn parse_from_mmap(&mut self, file: File) -> TDAResult<()> {
        let start_time = Instant::now();

        debug!("Memory-mapping 2DA file");
        let mmap = unsafe {
            Mmap::map(&file).map_err(|e| TDAError::MemoryMapError {
                details: e.to_string(),
            })?
        };

        self.clear();

        let content =
            std::str::from_utf8(&mmap).map_err(|e| TDAError::InvalidUtf8 {
                position: e.valid_up_to(),
            })?;

        self.parse_content(content)?;

        self.metadata_mut().file_size = mmap.len();
        self.metadata_mut().parse_time_ns = start_time.elapsed().as_nanos() as u64;

        info!("2DA parsed (mmap): {} rows, {} columns, {:.2}ms",
              self.row_count(), self.column_count(),
              start_time.elapsed().as_secs_f64() * 1000.0);

        Ok(())
    }

    fn parse_content(&mut self, content: &str) -> TDAResult<()> {
        let mut tokenizer = TDATokenizer::new();
        let mut header_parsed = false;
        let mut columns_parsed = false;
        let mut line_count = 0;

        for line in content.lines() {
            line_count += 1;

            self.security_limits().validate_line_length(line.len())?;

            let tokens = tokenizer.tokenize_line(line)?;

            if tokens.is_empty() {
                continue;
            }

            if !header_parsed {
                self.parse_header_direct(line.trim())?;
                header_parsed = true;
            } else if !columns_parsed {
                self.parse_columns(&tokens)?;
                self.security_limits()
                    .validate_column_count(self.column_count())?;
                columns_parsed = true;
            } else {
                self.parse_data_row(&tokens)?;
                self.security_limits()
                    .validate_row_count(self.row_count())?;
            }
        }

        self.metadata_mut().line_count = line_count;

        Ok(())
    }

    fn parse_header_direct(&mut self, line: &str) -> TDAResult<()> {
        let bytes = line.as_bytes();

        if bytes.len() < 8 {
            return Err(TDAError::InvalidHeader(line.to_string()));
        }

        let is_2dam = bytes.len() >= 4 && &bytes[0..4] == b"2DAM";
        let is_2da = &bytes[0..3] == b"2DA";

        if !is_2da && !is_2dam {
            return Err(TDAError::InvalidHeader(line.to_string()));
        }

        let version = if is_2dam {
            if bytes.len() >= 8 {
                std::str::from_utf8(&bytes[4..8]).unwrap_or("")
            } else {
                ""
            }
        } else if bytes.len() >= 8 && (bytes[3] == b' ' || bytes[3] == b'\t') {
            std::str::from_utf8(&bytes[4..8]).unwrap_or("")
        } else {
            ""
        };

        if version != "V2.0" && version != "V1.0" {
            return Err(TDAError::InvalidHeader(line.to_string()));
        }

        if is_2dam {
            self.metadata_mut().format_version = format!("2DAM{version}");
            self.metadata_mut().has_warnings = true;
        } else {
            self.metadata_mut().format_version = format!("2DA {version}");
        }

        Ok(())
    }

    fn parse_columns(&mut self, tokens: &[Token]) -> TDAResult<()> {
        if tokens.is_empty() {
            return Err(TDAError::MalformedLine {
                line_number: 2,
                details: "No column headers found".to_string(),
            });
        }

        let column_tokens = if tokens.len() > 1 && tokens[0].content.is_empty() {
            &tokens[1..]
        } else {
            tokens
        };

        if column_tokens.is_empty() {
            return Err(TDAError::MalformedLine {
                line_number: 2,
                details: "No valid column headers found after skipping empty first column"
                    .to_string(),
            });
        }

        self.columns_mut().reserve(column_tokens.len());
        self.column_map_mut().reserve(column_tokens.len());

        for (index, token) in column_tokens.iter().enumerate() {
            let symbol = self.interner_mut().get_or_intern(token.content);
            let column_info = ColumnInfo {
                name: symbol,
                index,
            };

            self.columns_mut().push(column_info);
            self.column_map_mut()
                .insert(token.content.to_lowercase(), index);
        }

        Ok(())
    }

    fn parse_data_row(&mut self, tokens: &[Token]) -> TDAResult<()> {
        if tokens.is_empty() {
            return Ok(());
        }

        let data_tokens = if tokens.len() > 1 { &tokens[1..] } else { &[] };

        let mut row = TDARow::new();
        row.reserve(self.columns().len());

        for (col_index, token) in data_tokens.iter().enumerate() {
            if col_index >= self.column_count() {
                break;
            }

            let cell_value = CellValue::new(token.content, self.interner_mut());
            row.push(cell_value);
        }

        while row.len() < self.column_count() {
            row.push(CellValue::Empty);
        }

        self.rows_mut().push(row);
        Ok(())
    }

    pub fn parse_from_string(&mut self, data: &str) -> TDAResult<()> {
        self.parse_from_bytes(data.as_bytes())
    }

    pub fn to_msgpack_compressed(&self) -> TDAResult<Vec<u8>> {
        use flate2::{write::ZlibEncoder, Compression};
        use std::io::Write;

        let serializable = SerializableTDAParser::from_parser(self);

        let msgpack_data = rmp_serde::to_vec(&serializable)?;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&msgpack_data)
            .map_err(|e| TDAError::CompressionError {
                details: e.to_string(),
            })?;

        encoder.finish().map_err(|e| TDAError::CompressionError {
            details: e.to_string(),
        })
    }

    pub fn from_msgpack_compressed(data: &[u8]) -> TDAResult<Self> {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| TDAError::CompressionError {
                details: e.to_string(),
            })?;

        let serializable: SerializableTDAParser = rmp_serde::from_slice(&decompressed)?;

        Ok(serializable.to_parser())
    }

    pub fn load_with_cache<P: AsRef<Path>>(
        &mut self,
        source_path: P,
        cache_path: Option<P>,
    ) -> TDAResult<bool> {
        if let Some(ref cache_path) = cache_path
            && let Ok(cache_data) = std::fs::read(cache_path)
                && let Ok(cached_parser) = Self::from_msgpack_compressed(&cache_data) {
                    *self = cached_parser;
                    return Ok(true);
                }

        self.parse_from_file(source_path)?;

        if let Some(cache_path) = cache_path
            && let Ok(compressed_data) = self.to_msgpack_compressed() {
                std::fs::write(cache_path, compressed_data)?;
            }

        Ok(false)
    }

    pub fn statistics(&self) -> ParserStatistics {
        ParserStatistics {
            total_cells: self.rows().len() * self.columns().len(),
            memory_usage: self.memory_usage(),
            interned_strings: self.interner().len(),
            parse_time_ms: self.metadata().parse_time_ns as f64 / 1_000_000.0,
            compression_ratio: if self.metadata().file_size > 0 {
                self.memory_usage() as f64 / self.metadata().file_size as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParserStatistics {
    pub total_cells: usize,
    pub memory_usage: usize,
    pub interned_strings: usize,
    pub parse_time_ms: f64,
    pub compression_ratio: f64,
}

pub fn load_multiple_files<P: AsRef<Path> + Send + Sync>(
    file_paths: &[P],
    security_limits: Option<SecurityLimits>,
) -> TDAResult<AHashMap<String, TDAParser>> {
    use rayon::prelude::*;
    use std::collections::HashMap;

    let limits = security_limits.unwrap_or_default();

    let results: Result<HashMap<String, TDAParser>, TDAError> = file_paths
        .par_iter()
        .map(|path| {
            let path_str = path.as_ref().to_string_lossy().to_string();
            let mut parser = TDAParser::with_limits(limits.clone());

            parser.parse_from_file(path).map(|()| (path_str, parser))
        })
        .collect();

    results.map(|hashmap| {
        let mut ahashmap = AHashMap::new();
        for (k, v) in hashmap {
            ahashmap.insert(k, v);
        }
        ahashmap
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_2DA: &str = r#"2DA V2.0

Label       Name        Description
0           test1       "Test Item 1"
1           test2       "Test Item 2"
2           ****        "Empty Label"
"#;

    #[test]
    fn test_basic_parsing() {
        let mut parser = TDAParser::new();
        parser.parse_from_string(SAMPLE_2DA).unwrap();

        assert_eq!(parser.column_count(), 3);
        assert_eq!(parser.row_count(), 3);

        let columns = parser.column_names();
        assert_eq!(columns, vec!["Label", "Name", "Description"]);
    }

    #[test]
    fn test_cell_access() {
        let mut parser = TDAParser::new();
        parser.parse_from_string(SAMPLE_2DA).unwrap();

        assert_eq!(parser.get_cell(0, 0).unwrap(), Some("test1"));
        assert_eq!(parser.get_cell(0, 1).unwrap(), Some("Test Item 1"));
        assert_eq!(parser.get_cell(1, 1).unwrap(), Some("Test Item 2"));

        assert_eq!(
            parser.get_cell_by_name(0, "Label").unwrap(),
            Some("test1")
        );
        assert_eq!(
            parser.get_cell_by_name(0, "Name").unwrap(),
            Some("Test Item 1")
        );
        assert_eq!(parser.get_cell_by_name(2, "Label").unwrap(), None);
    }

    #[test]
    fn test_security_limits() {
        let limits = SecurityLimits {
            max_file_size: 100,
            ..SecurityLimits::default()
        };

        let mut parser = TDAParser::with_limits(limits);
        let large_data = "x".repeat(200);

        assert!(parser.parse_from_string(&large_data).is_err());
    }
}
