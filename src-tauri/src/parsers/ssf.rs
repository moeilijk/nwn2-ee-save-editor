/// SSF binary format (NWN2 Sound Set File):
/// Both V1.0 and V1.1 share the same header layout:
/// - Header: "SSF " (4b) + version (4b) + entry_count (u32) + table_offset (u32)
/// - Offset table: entry_count x u32 absolute offsets into the file
/// - V1.0 entry: 16-byte resref + 4-byte strref (20 bytes)
/// - V1.1 entry: 32-byte resref + 4-byte strref (36 bytes)
pub fn parse_ssf(data: &[u8]) -> Result<Vec<String>, String> {
    if data.len() < 16 {
        return Err("SSF file too short".to_string());
    }
    if &data[0..4] != b"SSF " {
        return Err(format!("Invalid SSF magic: {:?}", &data[0..4]));
    }

    let version = &data[4..8];
    let resref_len: usize = match version {
        b"V1.1" => 32,
        _ => 16, // V1.0 and any other version
    };

    let entry_count = u32::from_le_bytes(
        data[8..12]
            .try_into()
            .map_err(|e| format!("Bad entry count: {e}"))?,
    ) as usize;
    let table_offset = u32::from_le_bytes(
        data[12..16]
            .try_into()
            .map_err(|e| format!("Bad table offset: {e}"))?,
    ) as usize;

    let mut resrefs = Vec::new();

    for i in 0..entry_count {
        let offset_pos = table_offset + i * 4;
        if offset_pos + 4 > data.len() {
            break;
        }
        let entry_offset =
            u32::from_le_bytes(data[offset_pos..offset_pos + 4].try_into().unwrap()) as usize;

        if entry_offset == 0 || entry_offset + resref_len > data.len() {
            continue;
        }

        let resref_bytes = &data[entry_offset..entry_offset + resref_len];
        let end = resref_bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(resref_len);
        let resref = String::from_utf8_lossy(&resref_bytes[..end])
            .trim()
            .to_string();

        if !resref.is_empty() {
            resrefs.push(resref);
        }
    }

    Ok(resrefs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_ssf_v10() -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"SSF ");
        buf.extend_from_slice(b"V1.0");
        buf.extend_from_slice(&3u32.to_le_bytes());
        buf.extend_from_slice(&16u32.to_le_bytes());

        buf.extend_from_slice(&28u32.to_le_bytes());
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf.extend_from_slice(&48u32.to_le_bytes());

        let mut entry0 = [0u8; 20];
        entry0[..14].copy_from_slice(b"vs_battlecry01");
        entry0[16..20].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        buf.extend_from_slice(&entry0);

        let mut entry2 = [0u8; 20];
        entry2[..10].copy_from_slice(b"vs_death01");
        entry2[16..20].copy_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        buf.extend_from_slice(&entry2);

        buf
    }

    fn build_test_ssf_v11() -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"SSF ");
        buf.extend_from_slice(b"V1.1");
        buf.extend_from_slice(&2u32.to_le_bytes());
        buf.extend_from_slice(&16u32.to_le_bytes());

        buf.extend_from_slice(&24u32.to_le_bytes());
        buf.extend_from_slice(&60u32.to_le_bytes());

        // Entry 0: 32-byte resref + 4-byte strref
        let mut entry0 = [0u8; 36];
        entry0[..16].copy_from_slice(b"gl_pc_male2_0000");
        entry0[32..36].copy_from_slice(&177617u32.to_le_bytes());
        buf.extend_from_slice(&entry0);

        let mut entry1 = [0u8; 36];
        entry1[..16].copy_from_slice(b"gl_pc_male2_0001");
        entry1[32..36].copy_from_slice(&177618u32.to_le_bytes());
        buf.extend_from_slice(&entry1);

        buf
    }

    #[test]
    fn test_parse_ssf_v10() {
        let data = build_test_ssf_v10();
        let result = parse_ssf(&data).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "vs_battlecry01");
        assert_eq!(result[1], "vs_death01");
    }

    #[test]
    fn test_parse_ssf_v11() {
        let data = build_test_ssf_v11();
        let result = parse_ssf(&data).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "gl_pc_male2_0000");
        assert_eq!(result[1], "gl_pc_male2_0001");
    }

    #[test]
    fn test_parse_ssf_rejects_invalid_magic() {
        let data = b"NOPE1234";
        assert!(parse_ssf(data).is_err());
    }

    #[test]
    fn test_parse_ssf_handles_too_short() {
        let data = b"SSF V1.0";
        assert!(parse_ssf(data).is_err());
    }
}
