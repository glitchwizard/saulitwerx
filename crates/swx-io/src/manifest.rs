//! `manifest.json`: what a `.sltwx` document is and which format version
//! wrote it (`docs/specs/FILE_FORMAT.md`, "manifest.json" and "Versioning").

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::{to_canonical_json, FormatError};

/// The only `format` value of a `.sltwx` manifest.
const FORMAT: &str = "sltwx";

/// The major `format_version` this reader supports.
const SUPPORTED_MAJOR: u64 = 0;

/// Which kind of document the container holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentKind {
    /// A part (`.sltwx`), written `"part"`.
    Part,
    /// An assembly (`.sltwxa`), written `"assembly"`.
    Assembly,
    /// A drawing (`.sltwxd`), written `"drawing"`.
    Drawing,
}

/// The document's units, by unit name (e.g. `"mm"`, `"deg"`, `"kg"`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Units {
    /// Length unit.
    pub length: String,
    /// Angle unit.
    pub angle: String,
    /// Mass unit.
    pub mass: String,
    /// Fields this reader does not know, kept and written back unchanged.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// The contents of `manifest.json`.
///
/// Fields this reader does not know, at the top level and inside `units`,
/// are kept in `extra` and written back unchanged, so a file from a newer
/// minor version round-trips without data loss (FILE_FORMAT.md,
/// "Versioning").
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Manifest {
    /// Always `"sltwx"`.
    pub format: String,
    /// Semantic version of the format, `MAJOR.MINOR.PATCH`.
    pub format_version: String,
    /// Part, assembly or drawing.
    pub kind: DocumentKind,
    /// UUID in 8-4-4-4-12 hex form, fixed for the document's life.
    pub document_id: String,
    /// The document's units.
    pub units: Units,
    /// The application and version that created the document.
    pub created_by: String,
    /// The application and version that last saved the document.
    pub saved_by: String,
    /// `sha256:` plus 64 lowercase hex digits: the hash of the canonical
    /// `model.json`. `None` is written by omitting the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_hash: Option<String>,
    /// Fields this reader does not know, kept and written back unchanged.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Reads and checks `manifest.json`.
///
/// Errors: malformed JSON ([`FormatError::Json`]); a missing required field
/// ([`FormatError::MissingField`]); a top level that is not an object, a
/// field of the wrong JSON type, a
/// `format` other than `"sltwx"`, a `format_version` that is not
/// `MAJOR.MINOR.PATCH` in decimal digits, an unknown `kind`, a
/// `document_id` that is not a UUID in 8-4-4-4-12 hex form (either case),
/// or a `model_hash` that is not `sha256:` plus 64 lowercase hex digits
/// ([`FormatError::InvalidField`]); a major version other than 0
/// ([`FormatError::UnsupportedVersion`]). A newer minor or patch version
/// of major 0 is accepted.
pub fn read_manifest(json: &str) -> Result<Manifest, FormatError> {
    let value: Value = serde_json::from_str(json).map_err(|e| FormatError::Json(e.to_string()))?;
    let Value::Object(mut top) = value else {
        return Err(invalid("manifest", "must be a JSON object".to_string()));
    };

    let format = take_str(&mut top, "", "format")?;
    if format != FORMAT {
        return Err(invalid(
            "format",
            format!("must be \"{FORMAT}\", got {format:?}"),
        ));
    }
    let format_version = take_str(&mut top, "", "format_version")?;
    check_version(&format_version)?;
    let kind_name = take_str(&mut top, "", "kind")?;
    let kind = match kind_name.as_str() {
        "part" => DocumentKind::Part,
        "assembly" => DocumentKind::Assembly,
        "drawing" => DocumentKind::Drawing,
        _ => {
            return Err(invalid(
                "kind",
                format!("must be \"part\", \"assembly\" or \"drawing\", got {kind_name:?}"),
            ))
        }
    };
    let document_id = take_str(&mut top, "", "document_id")?;
    if !is_uuid(&document_id) {
        return Err(invalid(
            "document_id",
            format!("must be a UUID in 8-4-4-4-12 hex form, got {document_id:?}"),
        ));
    }
    let units = match top.remove("units") {
        None => return Err(FormatError::MissingField("units".to_string())),
        Some(Value::Object(mut u)) => Units {
            length: take_str(&mut u, "units.", "length")?,
            angle: take_str(&mut u, "units.", "angle")?,
            mass: take_str(&mut u, "units.", "mass")?,
            extra: u.into_iter().collect(),
        },
        Some(_) => return Err(invalid("units", "must be an object".to_string())),
    };
    let created_by = take_str(&mut top, "", "created_by")?;
    let saved_by = take_str(&mut top, "", "saved_by")?;
    let model_hash = match top.remove("model_hash") {
        None => None,
        Some(Value::String(h)) if is_model_hash(&h) => Some(h),
        Some(other) => {
            return Err(invalid(
                "model_hash",
                format!("must be \"sha256:\" plus 64 lowercase hex digits, got {other}"),
            ))
        }
    };

    Ok(Manifest {
        format,
        format_version,
        kind,
        document_id,
        units,
        created_by,
        saved_by,
        model_hash,
        extra: top.into_iter().collect(),
    })
}

/// Writes `m` as canonical JSON ([`to_canonical_json`]), extra fields
/// included. The JSON shape comes from the derived `Serialize`: `kind` in
/// lowercase, `extra` flattened into its object, and `model_hash` omitted
/// when `None`.
pub fn write_manifest(m: &Manifest) -> String {
    to_canonical_json(m)
}

impl DocumentKind {
    /// The name written in `manifest.json`: `"part"`, `"assembly"` or
    /// `"drawing"`.
    pub fn as_str(self) -> &'static str {
        match self {
            DocumentKind::Part => "part",
            DocumentKind::Assembly => "assembly",
            DocumentKind::Drawing => "drawing",
        }
    }
}

fn invalid(field: &str, reason: String) -> FormatError {
    FormatError::InvalidField {
        field: field.to_string(),
        reason,
    }
}

/// Removes the required string field `key` from `obj`. `prefix` is the
/// path of `obj` for error messages (`""` or `"units."`).
fn take_str(obj: &mut Map<String, Value>, prefix: &str, key: &str) -> Result<String, FormatError> {
    match obj.remove(key) {
        None => Err(FormatError::MissingField(format!("{prefix}{key}"))),
        Some(Value::String(s)) => Ok(s),
        Some(other) => Err(invalid(
            &format!("{prefix}{key}"),
            format!("must be a string, got {other}"),
        )),
    }
}

/// `MAJOR.MINOR.PATCH`, each a non-empty run of ASCII digits that fits a
/// `u64`; then the major version must be supported.
fn check_version(v: &str) -> Result<(), FormatError> {
    let parts: Vec<&str> = v.split('.').collect();
    let numbers: Option<Vec<u64>> = (parts.len() == 3)
        .then(|| {
            parts
                .iter()
                .map(|p| {
                    if !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()) {
                        p.parse().ok()
                    } else {
                        None
                    }
                })
                .collect()
        })
        .flatten();
    match numbers {
        None => Err(invalid(
            "format_version",
            format!("must be MAJOR.MINOR.PATCH with decimal numbers, got {v:?}"),
        )),
        Some(n) if n[0] != SUPPORTED_MAJOR => Err(FormatError::UnsupportedVersion(v.to_string())),
        Some(_) => Ok(()),
    }
}

/// A UUID in 8-4-4-4-12 hex form, upper or lower case (RFC 9562, section 4).
fn is_uuid(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && b.iter().enumerate().all(|(i, &c)| match i {
            8 | 13 | 18 | 23 => c == b'-',
            _ => c.is_ascii_hexdigit(),
        })
}

/// `sha256:` followed by exactly 64 lowercase hex digits.
fn is_model_hash(s: &str) -> bool {
    s.strip_prefix("sha256:").is_some_and(|h| {
        h.len() == 64
            && h.bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOLDEN: &str = include_str!("../tests/golden/manifest-v0.1.json");

    fn golden_value() -> Value {
        serde_json::from_str(GOLDEN).unwrap()
    }

    /// The golden manifest with `key` set to `v` (or removed, for `None`).
    fn with(key: &str, v: Option<Value>) -> String {
        let mut obj = golden_value();
        let map = obj.as_object_mut().unwrap();
        match v {
            Some(v) => map.insert(key.to_string(), v),
            None => map.remove(key),
        };
        obj.to_string()
    }

    fn invalid_field(json: &str) -> String {
        match read_manifest(json) {
            Err(FormatError::InvalidField { field, .. }) => field,
            other => panic!("expected InvalidField, got {other:?}"),
        }
    }

    #[test]
    fn golden_round_trips_byte_for_byte() {
        let m = read_manifest(GOLDEN).unwrap();
        assert_eq!(m.kind, DocumentKind::Part);
        assert_eq!(m.units.length, "mm");
        assert!(m.extra.is_empty() && m.units.extra.is_empty());
        assert_eq!(write_manifest(&m), GOLDEN);
    }

    #[test]
    fn unknown_fields_round_trip_unchanged() {
        let mut v = golden_value();
        v["x_future"] = serde_json::json!([1, 2]);
        v["units"]["temperature"] = Value::String("K".into());
        let m = read_manifest(&v.to_string()).unwrap();
        assert_eq!(m.extra["x_future"], serde_json::json!([1, 2]));
        assert_eq!(m.units.extra["temperature"], "K");
        let back: Value = serde_json::from_str(&write_manifest(&m)).unwrap();
        assert_eq!(back, v);
        // `x_future` sorts after `units`, so it is the last member.
        assert!(write_manifest(&m).ends_with("  \"x_future\": [\n    1,\n    2\n  ]\n}\n"));
        assert!(write_manifest(&m).contains("    \"temperature\": \"K\"\n"));
    }

    #[test]
    fn keys_in_any_order_come_out_canonical() {
        let shuffled = r#"{"units": {"mass": "kg", "length": "mm", "angle": "deg"},
            "saved_by": "saulitwerx 0.4.1", "model_hash":
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "kind": "part", "format_version": "0.1.0", "format": "sltwx",
            "document_id": "6f1c2e0a-7b7d-4c55-9d3e-2a1f0e4b9c11",
            "created_by": "saulitwerx 0.3.0"}"#;
        assert_eq!(write_manifest(&read_manifest(shuffled).unwrap()), GOLDEN);
    }

    #[test]
    fn model_hash_is_optional() {
        let m = read_manifest(&with("model_hash", None)).unwrap();
        assert_eq!(m.model_hash, None);
        assert!(!write_manifest(&m).contains("model_hash"));
    }

    #[test]
    fn wrong_format_is_invalid() {
        assert_eq!(invalid_field(&with("format", Some("zip".into()))), "format");
    }

    #[test]
    fn malformed_versions_are_invalid() {
        for bad in [
            "0.1",
            "0.1.0.0",
            "",
            "0..1",
            "0.1.x",
            "v0.1.0",
            "0.1.-1",
            "0.1.0-rc1",
            " 0.1.0",
        ] {
            assert_eq!(
                invalid_field(&with("format_version", Some(bad.into()))),
                "format_version",
                "{bad:?}"
            );
        }
        assert_eq!(
            invalid_field(&with("format_version", Some(serde_json::json!(1)))),
            "format_version"
        );
    }

    #[test]
    fn other_major_versions_are_unsupported() {
        for v in ["1.0.0", "2.3.4"] {
            assert_eq!(
                read_manifest(&with("format_version", Some(v.into()))),
                Err(FormatError::UnsupportedVersion(v.to_string()))
            );
        }
    }

    #[test]
    fn newer_minor_with_extra_field_is_accepted() {
        let mut v = golden_value();
        v["format_version"] = "0.2.0".into();
        v["new_in_0_2"] = serde_json::json!({"b": 1, "a": 2});
        let m = read_manifest(&v.to_string()).unwrap();
        assert_eq!(m.format_version, "0.2.0");
        assert_eq!(m.extra["new_in_0_2"], serde_json::json!({"a": 2, "b": 1}));
    }

    #[test]
    fn unknown_kind_is_invalid() {
        for bad in ["Part", "sheet", ""] {
            assert_eq!(invalid_field(&with("kind", Some(bad.into()))), "kind");
        }
        for (name, kind) in [
            ("assembly", DocumentKind::Assembly),
            ("drawing", DocumentKind::Drawing),
        ] {
            let m = read_manifest(&with("kind", Some(name.into()))).unwrap();
            assert_eq!(m.kind, kind);
            assert!(write_manifest(&m).contains(&format!("\"kind\": \"{name}\"")));
        }
    }

    #[test]
    fn document_id_must_be_a_uuid() {
        for bad in [
            "6f1c2e0a7b7d4c559d3e2a1f0e4b9c11",
            "6f1c2e0a-7b7d-4c55-9d3e-2a1f0e4b9c1",
            "6f1c2e0a-7b7d-4c55-9d3e-2a1f0e4b9c111",
            "6f1c2e0a-7b7d-4c55-9d3e_2a1f0e4b9c11",
            "6g1c2e0a-7b7d-4c55-9d3e-2a1f0e4b9c11",
            "{6f1c2e0a-7b7d-4c55-9d3e-2a1f0e4b9c1}",
        ] {
            assert_eq!(
                invalid_field(&with("document_id", Some(bad.into()))),
                "document_id",
                "{bad:?}"
            );
        }
        let upper = "6F1C2E0A-7B7D-4C55-9D3E-2A1F0E4B9C11";
        assert!(read_manifest(&with("document_id", Some(upper.into()))).is_ok());
    }

    #[test]
    fn model_hash_must_be_sha256_lowercase_hex() {
        let hex = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        for bad in [
            format!("sha256:{}", hex.to_uppercase()),
            format!("sha256:{}", &hex[1..]),
            format!("sha256:{hex}0"),
            format!("sha1:{hex}"),
            hex.to_string(),
            "sha256:…".to_string(),
        ] {
            assert_eq!(
                invalid_field(&with("model_hash", Some(bad.clone().into()))),
                "model_hash",
                "{bad:?}"
            );
        }
        assert_eq!(
            invalid_field(&with("model_hash", Some(Value::Null))),
            "model_hash"
        );
    }

    #[test]
    fn missing_required_fields_are_named() {
        for key in [
            "format",
            "format_version",
            "kind",
            "document_id",
            "units",
            "created_by",
            "saved_by",
        ] {
            assert_eq!(
                read_manifest(&with(key, None)),
                Err(FormatError::MissingField(key.to_string()))
            );
        }
        let mut v = golden_value();
        v["units"].as_object_mut().unwrap().remove("mass");
        assert_eq!(
            read_manifest(&v.to_string()),
            Err(FormatError::MissingField("units.mass".to_string()))
        );
    }

    #[test]
    fn malformed_json_is_a_json_error() {
        for bad in ["{", "", "{\"format\": }", "{} x"] {
            assert!(
                matches!(read_manifest(bad), Err(FormatError::Json(_))),
                "{bad:?}"
            );
        }
        for not_object in ["[]", "\"sltwx\"", "null"] {
            assert_eq!(invalid_field(not_object), "manifest", "{not_object:?}");
        }
    }

    #[test]
    fn wrong_types_are_invalid() {
        assert_eq!(invalid_field(&with("units", Some("mm".into()))), "units");
        assert_eq!(
            invalid_field(&with("saved_by", Some(serde_json::json!(4)))),
            "saved_by"
        );
        let mut v = golden_value();
        v["units"]["angle"] = serde_json::json!(null);
        assert_eq!(invalid_field(&v.to_string()), "units.angle");
    }

    #[test]
    fn derived_serialize_matches_the_format() {
        for kind in [
            DocumentKind::Part,
            DocumentKind::Assembly,
            DocumentKind::Drawing,
        ] {
            assert_eq!(serde_json::to_value(kind).unwrap(), kind.as_str());
        }
        let mut m = read_manifest(GOLDEN).unwrap();
        m.model_hash = None;
        m.units.extra.insert("temperature".into(), "K".into());
        let v = serde_json::to_value(&m).unwrap();
        assert!(v.get("model_hash").is_none());
        assert!(v.get("extra").is_none() && v["units"].get("extra").is_none());
        assert_eq!(v["units"]["temperature"], "K");
    }

    #[test]
    fn errors_display_their_field() {
        let e = read_manifest(&with("format", Some("zip".into()))).unwrap_err();
        assert!(e.to_string().contains("`format`"), "{e}");
        let e = read_manifest("{").unwrap_err();
        assert!(e.to_string().starts_with("invalid JSON"), "{e}");
    }
}
