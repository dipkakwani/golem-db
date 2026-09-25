//! GolemDB cells: complete keys and typed values with owned and borrowed forms.
//!
//! # Wire format
//!
//! ```text
//! [ indexable: 1 bit | type id: 7 bits ] [ value … ]
//!
//! u64 7, indexable   8D 00 00 00 00 00 00 00 07
//! str "hi"           02 68 69
//! ```
//!
//! Each storage row or slice contains exactly one cell. Fixed-width types
//! require their declared width; `str` and `bytes` consume the remaining slice.
//! No value has a length prefix. Concatenating cells requires external boundaries.
//!
//! Type id `0x00` is not a type. It is the branch overlay's "absent" marker
//! ([`CellParseError::AbsentTag`]); a tombstone is `Option<CellValue>::None`.
//!
//! # Order encoding
//!
//! Stored value bytes sort as their values do, so a value is copied verbatim
//! into its index term `cellName ‖ 0x00 ‖ typeTag ‖ value`, and a cursor walk
//! over that is a range query. A value has one byte form everywhere.
//!
//! | stored form                         | types                                                 |
//! | ----------------------------------- | ----------------------------------------------------- |
//! | natural bytes                       | `bool`, `str`, `bytes20`, `bytes4..32`, `u32..u256`   |
//! | sign bit flipped ([`flip_sign`])    | `i32..i256`, `dec32..dec256`, `date32`, `timestamp64` |
//! | IEEE total order ([`encode_float`]) | `f32`, `f64`                                          |
//! | not indexable                       | `bytes`                                               |
//!
//! ```text
//! i32 -1   FF FF FF FF  stored  7F FF FF FF
//! i32 100  00 00 00 64  stored  80 00 00 64   7F… < 80…, so -1 < 100
//! ```
//!
//! Stored NaN and negative zero are rejected; [`encode_float`] normalizes
//! input negative zero to positive zero. Strings use raw UTF-8 throughout.
//!
//! Decimals are signed integers at a fixed scale per width
//! ([`Width::decimal_scale`]): `dec32` 4, `dec64` 6, `dec128` 18, `dec256` 18.
//! `dec256` is fixed by Arkiv (wei); the others await sign-off.
//!
//! # Addressing and ownership
//!
//! [`CellName`] is the within-record name (including raw reserved-record keys).
//! [`CellKey`] is the complete address: `record_id` as eight BE bytes, then the
//! name. [`CellValue`] keeps read results alive independently of a storage
//! transaction; [`CellValueRef`] borrows payloads for inspection without copying.
//! [`CellLimits`] checks genesis-configured admission policy separately from
//! representation validation.
//!
//! ```
//! use golemdb_cells::{CellKey, CellLimits, CellValueRef, CellValue};
//!
//! let limits = CellLimits {
//!     max_cell_name_len: 64, max_str_len: 1024, max_bytes_len: 65536,
//! };
//! let key = CellKey::new(42, limits.parse_user_name(b"$owner")?);
//! assert_eq!(&key.encode()[8..], b"$owner");
//! let owned = CellValue::parse(vec![0x82, b'h', b'i'])?;
//! limits.validate_value(owned.as_view())?;
//! assert_eq!(owned.as_str(), Some("hi"));
//! assert_eq!(CellValueRef::parse(owned.encoded_bytes())?, owned.as_view());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod config;
mod error;
mod key;
mod name;
mod order;
mod types;
mod value;

#[cfg(test)]
mod tests;

pub use config::{CellLimitError, CellLimits};
pub use error::CellParseError;
pub use key::{CellKey, CellKeyError};
pub use name::{CellName, CellNameError, CellNameRef, reserved};
pub use order::{decode_float, encode_float, flip_sign};
pub use types::{CellType, FloatWidth, Width};
pub use value::{CellValue, CellValueRef};

/// The metadata byte's high bit: whether the cell is indexable. The low 7
/// bits are the type id.
pub(crate) const INDEXABLE_BIT: u8 = 0x80;
