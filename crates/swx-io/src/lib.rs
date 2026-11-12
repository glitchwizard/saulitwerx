//! Saulitwerx file I/O for the native `.sltwx` family
//! (`docs/specs/FILE_FORMAT.md`).
//!
//! This first slice covers `manifest.json` ([`Manifest`], [`read_manifest`],
//! [`write_manifest`]) and the canonical JSON writer
//! ([`to_canonical_json`]) that every file of the format uses.

mod canonical;
mod manifest;

pub use canonical::to_canonical_json;
pub use manifest::{read_manifest, write_manifest, DocumentKind, Manifest, Units};

use std::fmt;

/// Why a `.sltwx` file could not be read. Reading never panics; every
/// problem is one of these.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// The text is not valid JSON. Holds the parser's message.
    Json(String),
    /// A required field is missing. Holds its path, e.g. `"units.length"`.
    MissingField(String),
    /// A field is present but its value is not allowed.
    InvalidField {
        /// The field's path, e.g. `"format_version"` or `"units.mass"`.
        field: String,
        /// What is wrong with it.
        reason: String,
    },
    /// The file's `format_version` has a major version this reader does not
    /// support. Holds the version string.
    UnsupportedVersion(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Json(msg) => write!(f, "invalid JSON: {msg}"),
            FormatError::MissingField(field) => write!(f, "missing required field `{field}`"),
            FormatError::InvalidField { field, reason } => {
                write!(f, "invalid field `{field}`: {reason}")
            }
            FormatError::UnsupportedVersion(v) => {
                write!(
                    f,
                    "unsupported format_version {v:?}: only major version 0 is supported"
                )
            }
        }
    }
}

impl std::error::Error for FormatError {}
