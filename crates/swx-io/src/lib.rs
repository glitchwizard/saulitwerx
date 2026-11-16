//! Saulitwerx file I/O for the native `.sltwx` family
//! (`docs/specs/FILE_FORMAT.md`).
//!
//! So far this covers `manifest.json` ([`Manifest`], [`read_manifest`],
//! [`write_manifest`]), the canonical JSON writer ([`to_canonical_json`])
//! that every file of the format uses, and whole documents in the unpacked
//! folder form ([`Document`], [`read_unpacked`], [`write_unpacked`]) and as a
//! single ZIP archive ([`read_sltwx`], [`write_sltwx`]).

mod canonical;
mod manifest;
mod packed;
mod unpacked;
mod zip;

pub use canonical::to_canonical_json;
pub use manifest::{read_manifest, write_manifest, DocumentKind, Manifest, Units};
pub use packed::{read_sltwx, write_sltwx};
pub use unpacked::{read_unpacked, write_unpacked, Document};

use std::fmt;
use std::path::PathBuf;

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
        /// The field's path, e.g. `"format_version"` or `"units.mass"`, or
        /// the file name (`"model.json"`) when the whole file is wrong.
        field: String,
        /// What is wrong with it.
        reason: String,
    },
    /// The file's `format_version` has a major version this reader does not
    /// support. Holds the version string.
    UnsupportedVersion(String),
    /// A file or folder could not be read, written or removed, or a path that
    /// must be a folder is not one.
    Io {
        /// The file or folder concerned.
        path: PathBuf,
        /// What went wrong, e.g. the operating system's message.
        message: String,
    },
    /// A required file of the document is missing. Holds its name:
    /// `"manifest.json"` or `"model.json"`.
    MissingFile(String),
    /// The folder's extension does not match the manifest's `kind`
    /// (`.sltwx` ↔ part, `.sltwxa` ↔ assembly, `.sltwxd` ↔ drawing). The
    /// manifest is authoritative (FILE_FORMAT.md, "Extensions").
    ExtensionMismatch {
        /// The folder's extension without the dot, or `None` if it has none.
        extension: Option<String>,
        /// The manifest's kind.
        kind: DocumentKind,
    },
    /// The file is not a readable ZIP archive: it has no end-of-central-
    /// directory record, an entry is truncated, a header is inconsistent, an
    /// entry's deflate data is corrupt, or the file is not a ZIP at all.
    /// Holds what is wrong.
    MalformedArchive(String),
    /// An archive entry uses a compression method other than 0 (stored) or
    /// 8 (deflate).
    UnsupportedMethod {
        /// The entry's name.
        entry: String,
        /// The ZIP compression method number.
        method: u16,
    },
    /// An archive entry's data does not match its recorded CRC-32. Holds the
    /// entry's name.
    CrcMismatch(String),
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
            FormatError::Io { path, message } => write!(f, "{}: {message}", path.display()),
            FormatError::MissingFile(name) => write!(f, "missing required file `{name}`"),
            FormatError::MalformedArchive(why) => write!(f, "malformed ZIP archive: {why}"),
            FormatError::UnsupportedMethod { entry, method } => write!(
                f,
                "archive entry `{entry}` uses unsupported compression method {method} (only 0 and 8 are supported)"
            ),
            FormatError::CrcMismatch(entry) => {
                write!(f, "archive entry `{entry}` fails its CRC-32 check")
            }
            FormatError::ExtensionMismatch { extension, kind } => {
                let expected = unpacked::extension_of(*kind);
                let kind = kind.as_str();
                match extension {
                    Some(ext) => write!(
                        f,
                        "folder extension `.{ext}` does not match the manifest kind `{kind}` (expected `.{expected}`)"
                    ),
                    None => write!(
                        f,
                        "folder has no extension; the manifest kind `{kind}` needs `.{expected}`"
                    ),
                }
            }
        }
    }
}

impl std::error::Error for FormatError {}
