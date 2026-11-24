//! The derived cache of a document and its index, `cache/index.json`
//! (`docs/specs/M4-04.md` §1, §2, §4).
//!
//! The index records every derived file (`cache/<name>` and
//! `thumbnail.png`), the SHA-256 of its bytes and the `model_hash` it was
//! built from. Reading a cache never fails: an unreadable index means no
//! cache, and a bad entry is dropped.

use std::collections::BTreeMap;

use serde_json::{json, Value};

use crate::{sha256, to_canonical_json, FormatError};

/// The index's container path.
pub(crate) const INDEX: &str = "cache/index.json";
/// The thumbnail's container path (FILE_FORMAT.md, "Container").
pub(crate) const THUMBNAIL: &str = "thumbnail.png";
/// The folder of derived files.
pub(crate) const CACHE_DIR: &str = "cache";
/// The only `cache_format` this reader understands.
const CACHE_FORMAT: u64 = 1;

/// The derived files of a document, keyed by container path
/// (`cache/body-b-1.brep`, `cache/display.glb`, `thumbnail.png`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cache {
    /// The files, by container path.
    pub entries: BTreeMap<String, CacheEntry>,
}

/// One derived file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheEntry {
    /// The `model_hash` (`sha256:<64 lowercase hex>`) of the model this file
    /// was built from.
    pub model_hash: String,
    /// The file's bytes, exactly as stored.
    pub bytes: Vec<u8>,
}

/// Whether `path` may hold a cache entry: `thumbnail.png`, or `cache/<name>`
/// where `<name>` is a plain file name (no `/` or `\`, not `.` or `..`)
/// other than `index.json` (spec §1).
pub(crate) fn is_entry_path(path: &str) -> bool {
    if path == THUMBNAIL {
        return true;
    }
    match path.strip_prefix("cache/") {
        Some(name) => {
            !name.is_empty()
                && name != "."
                && name != ".."
                && name != "index.json"
                && !name.contains(['/', '\\'])
        }
        None => false,
    }
}

/// Whether `s` is `sha256:` followed by 64 lowercase hex digits.
pub(crate) fn is_hash(s: &str) -> bool {
    s.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

/// `sha256:<hex>` of `bytes`.
pub(crate) fn hash_of(bytes: &[u8]) -> String {
    let hex: String = sha256(bytes).iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256:{hex}")
}

/// Checks that every entry of `cache` can be written: a valid path and a
/// well-formed `model_hash`. Writers call it before writing anything.
pub(crate) fn check_cache(cache: &Cache) -> Result<(), FormatError> {
    for (path, entry) in &cache.entries {
        if !is_entry_path(path) {
            return Err(FormatError::InvalidField {
                field: "cache".into(),
                reason: format!("{path:?} is not `thumbnail.png` or `cache/<name>`"),
            });
        }
        if !is_hash(&entry.model_hash) {
            return Err(FormatError::InvalidField {
                field: "cache".into(),
                reason: format!("the model_hash of {path:?} is not `sha256:<64 lowercase hex>`"),
            });
        }
    }
    Ok(())
}

/// The text of `cache/index.json` for `cache`, in canonical form, with
/// `entries` sorted by `path` in byte order.
pub(crate) fn index_json(cache: &Cache) -> String {
    let entries: Vec<Value> = cache
        .entries
        .iter()
        .map(|(path, e)| {
            json!({ "model_hash": e.model_hash, "path": path, "sha256": hash_of(&e.bytes) })
        })
        .collect();
    to_canonical_json(&json!({ "cache_format": CACHE_FORMAT, "entries": entries }))
}

/// One valid index entry: path, `model_hash`, `sha256`.
type IndexEntry = (String, String, String);

/// The valid entries of an index, or `None` when the index is unreadable
/// (not JSON, not an object, `cache_format` not 1, `entries` not an array;
/// spec §4). Entries with a bad `path`, `model_hash` or `sha256` are dropped.
fn parse_index(bytes: &[u8]) -> Option<Vec<IndexEntry>> {
    let v: Value = serde_json::from_slice(bytes).ok()?;
    if v.get("cache_format").and_then(Value::as_u64) != Some(CACHE_FORMAT) {
        return None;
    }
    let entries = v.get("entries")?.as_array()?;
    Some(
        entries
            .iter()
            .filter_map(|e| {
                let field = |k: &str| e.get(k).and_then(Value::as_str).map(str::to_owned);
                let (path, model_hash, sha) =
                    (field("path")?, field("model_hash")?, field("sha256")?);
                (is_entry_path(&path) && is_hash(&model_hash) && is_hash(&sha))
                    .then_some((path, model_hash, sha))
            })
            .collect(),
    )
}

/// Reads a cache: `index` is the bytes of `cache/index.json` if there is
/// one, and `file` gives the bytes of a container path if it exists. Keeps
/// each valid entry whose file exists and matches its `sha256` (spec §1
/// checks 1–3); never fails.
pub(crate) fn read_cache(
    index: Option<Vec<u8>>,
    mut file: impl FnMut(&str) -> Option<Vec<u8>>,
) -> Option<Cache> {
    let listed = parse_index(&index?)?;
    let mut entries = BTreeMap::new();
    for (path, model_hash, sha) in listed {
        if let Some(bytes) = file(&path) {
            if hash_of(&bytes) == sha {
                entries.insert(path, CacheEntry { model_hash, bytes });
            }
        }
    }
    Some(Cache { entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_paths() {
        for ok in ["thumbnail.png", "cache/body-b-1.brep", "cache/display.glb"] {
            assert!(is_entry_path(ok), "{ok}");
        }
        for bad in [
            "../x",
            "cache/index.json",
            "cache/a/b",
            "cache/",
            "cache/..",
            "cache/a\\b",
            "x.png",
            "/cache/a",
        ] {
            assert!(!is_entry_path(bad), "{bad}");
        }
    }

    #[test]
    fn hashes() {
        let good = hash_of(b"abc");
        assert_eq!(
            good,
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert!(is_hash(&good));
        assert!(!is_hash(&good["sha256:".len()..]));
        assert!(!is_hash(&good.to_uppercase()));
        assert!(!is_hash(&good[..good.len() - 1]));
    }

    #[test]
    fn check_cache_rejects_bad_paths_and_hashes() {
        let entry = |h: &str| CacheEntry {
            model_hash: h.into(),
            bytes: vec![],
        };
        let good = hash_of(b"");
        let mut c = Cache::default();
        c.entries.insert("cache/a".into(), entry(&good));
        assert!(check_cache(&c).is_ok());
        c.entries.insert("cache/index.json".into(), entry(&good));
        assert!(check_cache(&c).is_err());
        c.entries.remove("cache/index.json");
        c.entries.insert("thumbnail.png".into(), entry("sha256:x"));
        assert!(check_cache(&c).is_err());
    }
}
