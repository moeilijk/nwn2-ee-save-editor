use app_lib::parsers::mdb::types::MdbSecurityLimits;
use app_lib::parsers::mdb::{MdbError, MdbParser};

fn build_header(major: u16, minor: u16, packet_count: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity(12);
    data.extend_from_slice(b"NWN2");
    data.extend_from_slice(&major.to_le_bytes());
    data.extend_from_slice(&minor.to_le_bytes());
    data.extend_from_slice(&packet_count.to_le_bytes());
    data
}

#[test]
fn test_parse_invalid_signature() {
    let mut data = build_header(1, 0, 0);
    data[0] = b'X';

    let result = MdbParser::parse(&data);
    assert!(
        result.is_err(),
        "Parser must reject data with invalid signature"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, MdbError::InvalidSignature { .. }),
        "Expected InvalidSignature, got: {err}"
    );
}

#[test]
fn test_parse_empty_file() {
    let result = MdbParser::parse(&[]);
    assert!(result.is_err(), "Parser must reject empty input");
    let err = result.unwrap_err();
    assert!(
        matches!(err, MdbError::Io(_)),
        "Expected Io error on empty input, got: {err}"
    );
}

#[test]
fn test_parse_header_only() {
    let data = build_header(1, 0, 0);
    assert_eq!(data.len(), 12, "Header must be exactly 12 bytes");

    let result = MdbParser::parse(&data);
    assert!(
        result.is_ok(),
        "Valid 12-byte header with 0 packets must parse successfully: {:?}",
        result.err()
    );

    let file = result.unwrap();
    assert_eq!(file.header.packet_count, 0);
    assert!(file.rigid_meshes.is_empty());
    assert!(file.skin_meshes.is_empty());
    assert!(file.hooks.is_empty());
    assert!(file.hair.is_empty());
    assert!(file.helm.is_empty());
}

#[test]
fn test_parse_security_limits() {
    let data = build_header(1, 0, 0);

    let tight_limits = MdbSecurityLimits {
        max_file_size: 4,
        max_packet_count: 1000,
        max_vertex_count: 1_000_000,
        max_face_count: 1_000_000,
    };

    let result = MdbParser::parse_with_limits(&data, &tight_limits);
    assert!(
        result.is_err(),
        "Parser must reject files exceeding max_file_size"
    );
    let err = result.unwrap_err();
    assert!(
        matches!(err, MdbError::SecurityViolation { .. }),
        "Expected SecurityViolation for oversized file, got: {err}"
    );
}
