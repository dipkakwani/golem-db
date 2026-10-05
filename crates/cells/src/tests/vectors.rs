//! The cell vectors: byte strings that must parse, and ones that must not.

use crate::*;

/// Name, wire bytes, indexable, type, stored value.
pub(super) type Vector = (&'static str, &'static [u8], bool, CellType, &'static [u8]);

/// `ty`'s type id followed by `N - 1` zero bytes, for the wide rows below.
const fn zero_cell<const N: usize>(ty: CellType) -> [u8; N] {
    let mut out = [0; N];
    out[0] = ty.id();
    out
}

const BYTES16: CellType = CellType::FixedBytes(Width::W16);
const BYTES32: CellType = CellType::FixedBytes(Width::W32);
const U128: CellType = CellType::Uint(Width::W16);
const U256: CellType = CellType::Uint(Width::W32);
const I128: CellType = CellType::Int(Width::W16);
const I256: CellType = CellType::Int(Width::W32);
const DEC128: CellType = CellType::Decimal(Width::W16);
const DEC256: CellType = CellType::Decimal(Width::W32);

/// Every type at least once, with both settings of the indexable bit represented.
#[rustfmt::skip]
pub(super) const VECTORS: &[Vector] = &[
    // name              wire bytes idx                                                  type                              value
    ("bool false",       &[0x01, 0x00],                                           false, CellType::Bool,                   &[0x00]),
    ("bool true, idx",   &[0x81, 0x01],                                           true,  CellType::Bool,                   &[0x01]),
    ("str empty",        &[0x02],                                                 false, CellType::Str,                    &[]),
    ("str ascii",        &[0x02, b'h', b'i'],                                     false, CellType::Str,                    b"hi"),
    ("str 4-byte, idx",  &[0x82, 0xF0, 0x9F, 0xA6, 0x80],                         true,  CellType::Str,                    &[0xF0, 0x9F, 0xA6, 0x80]),
    ("str with a NUL",   &[0x02, b'a', 0x00, b'b'],                               false, CellType::Str,                    &[b'a', 0x00, b'b']),
    ("bytes empty",      &[0x03],                                                 false, CellType::Bytes,                  &[]),
    ("bytes 2",          &[0x03, 0xDE, 0xAD],                                     false, CellType::Bytes,                  &[0xDE, 0xAD]),
    ("bytes with zeros", &[0x03, 0, 0, 0, 1, 255, 0, 128],                        false, CellType::Bytes,                  &[0, 0, 0, 1, 255, 0, 128]),
    ("bytes20",          &zero_cell::<21>(CellType::Bytes20),                     false, CellType::Bytes20,                &[0; 20]),
    ("bytes4",           &[0x08, 1, 2, 3, 4],                                     false, CellType::FixedBytes(Width::W4),  &[1, 2, 3, 4]),
    ("bytes8, idx",      &[0x89, 1, 2, 3, 4, 5, 6, 7, 8],                         true,  CellType::FixedBytes(Width::W8),  &[1, 2, 3, 4, 5, 6, 7, 8]),
    ("bytes16",          &zero_cell::<17>(BYTES16),                               false, BYTES16,                          &[0; 16]),
    ("bytes32",          &zero_cell::<33>(BYTES32),                               false, BYTES32,                          &[0; 32]),
    ("u32 7",            &[0x0C, 0, 0, 0, 7],                                     false, CellType::Uint(Width::W4),        &[0, 0, 0, 7]),
    ("u64 7, idx",       &[0x8D, 0, 0, 0, 0, 0, 0, 0, 7],                         true,  CellType::Uint(Width::W8),        &[0, 0, 0, 0, 0, 0, 0, 7]),
    ("u128 0",           &zero_cell::<17>(U128),                                  false, U128,                             &[0; 16]),
    ("u256 0",           &zero_cell::<33>(U256),                                  false, U256,                             &[0; 32]),
    // Signed values are stored sign-flipped: 7F FF… is -1, 00 00… is MIN.
    ("i32 -1",           &[0x10, 0x7F, 0xFF, 0xFF, 0xFF],                         false, CellType::Int(Width::W4),         &[0x7F, 0xFF, 0xFF, 0xFF]),
    ("i64 -1",           &[0x11, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], false, CellType::Int(Width::W8),         &[0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]),
    ("i128 MIN",         &zero_cell::<17>(I128),                                  false, I128,                             &[0; 16]),
    ("i256 MIN",         &zero_cell::<33>(I256),                                  false, I256,                             &[0; 32]),
    ("dec32 -1",         &[0x14, 0x7F, 0xFF, 0xFF, 0xFF],                         false, CellType::Decimal(Width::W4),     &[0x7F, 0xFF, 0xFF, 0xFF]),
    ("dec64 MIN",        &[0x15, 0, 0, 0, 0, 0, 0, 0, 0],                         false, CellType::Decimal(Width::W8),     &[0, 0, 0, 0, 0, 0, 0, 0]),
    ("dec128 MIN",       &zero_cell::<17>(DEC128),                                false, DEC128,                           &[0; 16]),
    ("dec256 MIN",       &zero_cell::<33>(DEC256),                                false, DEC256,                           &[0; 32]),
    // 1.0 is 3F 80 00 00; stored with the sign bit flipped.
    ("f32 1.0",          &[0x18, 0xBF, 0x80, 0, 0],                               false, CellType::Float(FloatWidth::F32), &[0xBF, 0x80, 0, 0]),
    ("f64 1.0",          &[0x19, 0xBF, 0xF0, 0, 0, 0, 0, 0, 0],                   false, CellType::Float(FloatWidth::F64), &[0xBF, 0xF0, 0, 0, 0, 0, 0, 0]),
    ("date32 20000",     &[0x1C, 0x80, 0, 0x4E, 0x20],                            false, CellType::Date32,                 &[0x80, 0, 0x4E, 0x20]),
    ("timestamp64 1",    &[0x9D, 0x80, 0, 0, 0, 0, 0, 0, 1],                      true,  CellType::Timestamp64,            &[0x80, 0, 0, 0, 0, 0, 0, 1]),
];

/// Name, wire bytes, and the exact error they must produce.
pub(super) type BadVector = (&'static str, &'static [u8], CellValueParseError);

#[rustfmt::skip]
pub(super) const BAD_VECTORS: &[BadVector] = &[
    ("empty slice",           &[],                                                     CellValueParseError::Empty),
    ("absent tag",            &[0x00],                                                 CellValueParseError::AbsentTag),
    ("absent tag, idx bit",   &[0x80, 9],                                              CellValueParseError::AbsentTag),
    ("reserved 5",            &[5],                                                    CellValueParseError::ReservedType(5)),
    ("reserved 64, payload",  &[64, 0, 0, 0, 0],                                       CellValueParseError::ReservedType(64)),
    ("reserved 127, idx bit", &[0xFF, 1, 2, 3],                                        CellValueParseError::ReservedType(127)),

    ("bytes20 too short",     &[0x04, 0, 0],                                           CellValueParseError::LengthMismatch { ty: CellType::Bytes20, expected: 20, actual: 2 }),
    ("u64 empty",             &[0x0D],                                                 CellValueParseError::LengthMismatch { ty: CellType::Uint(Width::W8), expected: 8, actual: 0 }),
    ("bool, spare byte",      &[0x01, 1, 9],                                           CellValueParseError::LengthMismatch { ty: CellType::Bool, expected: 1, actual: 2 }),

    ("bool byte 2",           &[0x01, 2],                                              CellValueParseError::InvalidBool(2)),
    ("str, bad byte mid-way", &[0x02, b'h', 0x80, b'i'],                               CellValueParseError::InvalidUtf8 { valid_up_to: 1 }),
    // An incomplete UTF-8 code point is invalid.
    ("str, incomplete char",  &[0x02, 0xC3],                                           CellValueParseError::InvalidUtf8 { valid_up_to: 0 }),

    // Stored forms: NaN 7FC00000 flips to FFC00000; -0.0 80000000 inverts to 7FFFFFFF.
    ("f32 NaN",               &[0x18, 0xFF, 0xC0, 0, 0],                               CellValueParseError::FloatNaN),
    ("f32 -0.0",              &[0x18, 0x7F, 0xFF, 0xFF, 0xFF],                         CellValueParseError::NegativeZero),
    ("f64 negative NaN",      &[0x19, 0x00, 0x07, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], CellValueParseError::FloatNaN),
    ("f64 -0.0",              &[0x19, 0x7F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], CellValueParseError::NegativeZero),

    ("bytes, idx bit",        &[0x83],                                                 CellValueParseError::NotIndexable),
];

/// Large payloads are generated to keep the fixture source compact.
pub(super) const LARGE_PAYLOAD_LEN: usize = u16::MAX as usize + 1;

pub(super) fn large_value_vectors() -> impl Iterator<Item = (CellType, Vec<u8>, Vec<u8>)> {
    [(CellType::Str, 0x02), (CellType::Bytes, 0x03)]
        .into_iter()
        .map(|(ty, tag)| {
            let payload = vec![b'a'; LARGE_PAYLOAD_LEN];
            let mut wire = vec![tag];
            wire.extend_from_slice(&payload);
            (ty, wire, payload)
        })
}

/// Name, native IEEE bytes, expected ordered bytes, and cell validation result.
pub(super) type FloatInputVector<const N: usize> = (
    &'static str,
    [u8; N],
    [u8; N],
    std::result::Result<(), CellValueParseError>,
);

#[rustfmt::skip]
pub(super) const FLOAT32_INPUTS: &[FloatInputVector<4>] = &[
    ("f32 +0.0", [0x00, 0, 0, 0],    [0x80, 0, 0, 0],    Ok(())),
    ("f32 -0.0", [0x80, 0, 0, 0],    [0x80, 0, 0, 0],    Ok(())),
    ("f32 NaN",  [0x7F, 0xC0, 0, 0], [0xFF, 0xC0, 0, 0], Err(CellValueParseError::FloatNaN)),
];

#[rustfmt::skip]
pub(super) const FLOAT64_INPUTS: &[FloatInputVector<8>] = &[
    ("f64 +0.0", [0x00, 0, 0, 0, 0, 0, 0, 0],    [0x80, 0, 0, 0, 0, 0, 0, 0],    Ok(())),
    ("f64 -0.0", [0x80, 0, 0, 0, 0, 0, 0, 0],    [0x80, 0, 0, 0, 0, 0, 0, 0],    Ok(())),
    ("f64 NaN",  [0x7F, 0xF8, 0, 0, 0, 0, 0, 0], [0xFF, 0xF8, 0, 0, 0, 0, 0, 0], Err(CellValueParseError::FloatNaN)),
];
