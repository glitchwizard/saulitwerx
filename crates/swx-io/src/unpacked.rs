//! The unpacked (git-friendly) form of a `.sltwx` document: a directory
//! `<name>.sltwx/` (`.sltwxa`, `.sltwxd`) with the container's layout
//! (`docs/specs/FILE_FORMAT.md`, "Container" and "Unpacked (git-friendly)
//! form").

use std::fs;
use std::io;
use std::path::Path;

use serde_json::Value;

use crate::ids::{check_ids, model_for_write};
use crate::{
    model_hash, read_manifest, to_canonical_json, write_manifest, DocumentForm, DocumentKind,
    FormatError, Manifest,
};

const MANIFEST: &str = "manifest.json";
const MODEL: &str = "model.json";
const PROPERTIES: &str = "properties.json";

/// A whole `.sltwx` document: its three JSON files.
///
/// `model.json` and `properties.json` are kept as JSON objects: their typed
/// schemas come later. Unknown fields in them round-trip unchanged.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// `manifest.json`.
    pub manifest: Manifest,
    /// `model.json`, the source of truth. Always a JSON object.
    pub model: Value,
    /// `properties.json`, if the document has one. Always a JSON object.
    pub properties: Option<Value>,
}

impl Document {
    /// Sets `manifest.model_hash` to the hash of the `model.json` bytes the
    /// writers produce: [`model_hash`] of `model` after the writers' own
    /// preparation (a part's ID sets sorted by `id`, `ids.rs`).
    pub fn refresh_model_hash(&mut self) {
        self.manifest.model_hash = Some(self.written_model_hash());
    }

    /// Whether a cache built from the model hash `recorded` is still valid:
    /// true only when `recorded` equals the hash
    /// [`Document::refresh_model_hash`] records
    /// (FILE_FORMAT.md: "If they differ, the cache is stale"). A malformed
    /// `recorded` string is never equal, so it gives `false`.
    pub fn cache_is_fresh(&self, recorded: &str) -> bool {
        recorded == self.written_model_hash()
    }

    /// [`model_hash`] of the model as the writers write it. A model whose
    /// IDs fail the writers' check can't be written; it is hashed as given.
    fn written_model_hash(&self) -> String {
        match model_for_write(self.manifest.kind, &self.model) {
            Ok(model) => model_hash(&model),
            Err(_) => model_hash(&self.model),
        }
    }
}

/// Reads the document in the unpacked folder `dir`
/// (`docs/specs/FILE_FORMAT.md`, "Unpacked (git-friendly) form").
///
/// `manifest.json` and `model.json` are required; `properties.json` is
/// optional. Other entries (`cache/`, `thumbnail.png`, `attachments/`) are
/// not read.
///
/// # Errors
///
/// - [`FormatError::Io`]: `dir` is missing or not a directory, or a file
///   can't be read.
/// - [`FormatError::MissingFile`]: no `manifest.json` or `model.json`.
/// - [`FormatError::Json`]: a file is not valid JSON (the message names it).
/// - [`FormatError::InvalidField`] naming `model.json` or `properties.json`:
///   that file is not a JSON object. Manifest problems are reported as by
///   [`read_manifest`].
/// - [`FormatError::ExtensionMismatch`]: `dir`'s extension doesn't match the
///   manifest's `kind`, which is authoritative.
/// - [`FormatError::InvalidId`] / [`FormatError::DuplicateId`]: a part's
///   `model.json` breaks the stable-ID rules (an ID array that is not an
///   array is [`FormatError::InvalidField`]). Nothing is reordered on read.
pub fn read_unpacked(dir: &Path) -> Result<Document, FormatError> {
    let meta = fs::metadata(dir).map_err(|e| io_error(dir, &e))?;
    if !meta.is_dir() {
        return Err(FormatError::Io {
            path: dir.to_path_buf(),
            message: "not a directory".into(),
        });
    }
    let manifest_text =
        read_file(dir, MANIFEST)?.ok_or(FormatError::MissingFile(MANIFEST.into()))?;
    let manifest = read_manifest(&manifest_text).map_err(|e| match e {
        FormatError::Json(m) => FormatError::Json(format!("{MANIFEST}: {m}")),
        other => other,
    })?;
    check_extension(dir, DocumentForm::Folder, manifest.kind)?;
    let model_text = read_file(dir, MODEL)?.ok_or(FormatError::MissingFile(MODEL.into()))?;
    let model = parse_object(MODEL, &model_text)?;
    check_ids(manifest.kind, &model)?;
    let properties = match read_file(dir, PROPERTIES)? {
        Some(text) => Some(parse_object(PROPERTIES, &text)?),
        None => None,
    };
    Ok(Document {
        manifest,
        model,
        properties,
    })
}

/// Writes `doc` to the unpacked folder `dir`
/// (`docs/specs/FILE_FORMAT.md`, "Unpacked (git-friendly) form"), creating
/// `dir` if it is missing.
///
/// Each file is written in canonical form ([`to_canonical_json`];
/// `manifest.json` through [`write_manifest`]). For a part, the set arrays of
/// `model.json` (`sketches`, `bodies`, and each sketch's `entities` and
/// `constraints`) are written sorted by `id` in byte order; `features` keep
/// their order, and `doc` itself is not changed. `properties.json` is written
/// when `doc.properties` is `Some` and removed when it is `None`. No other
/// entry in `dir` is touched, so `cache/`, `thumbnail.png`, `attachments/`
/// and unknown files stay as they were.
///
/// # Errors
///
/// Nothing is written when `dir`'s extension doesn't match the manifest's
/// `kind` ([`FormatError::ExtensionMismatch`]) or `model` or `properties` is
/// not a JSON object ([`FormatError::InvalidField`] naming the file), or a
/// part's IDs are invalid ([`FormatError::InvalidId`],
/// [`FormatError::DuplicateId`]), since [`read_unpacked`] would reject the
/// result. Otherwise
/// [`FormatError::Io`] when `dir` or a file can't be created, written or
/// removed.
pub fn write_unpacked(dir: &Path, doc: &Document) -> Result<(), FormatError> {
    check_extension(dir, DocumentForm::Folder, doc.manifest.kind)?;
    check_object(MODEL, &doc.model)?;
    if let Some(p) = &doc.properties {
        check_object(PROPERTIES, p)?;
    }
    let model = model_for_write(doc.manifest.kind, &doc.model)?;
    fs::create_dir_all(dir).map_err(|e| io_error(dir, &e))?;
    write_file(dir, MANIFEST, &write_manifest(&doc.manifest))?;
    write_file(dir, MODEL, &to_canonical_json(&model))?;
    let props = dir.join(PROPERTIES);
    match &doc.properties {
        Some(p) => write_file(dir, PROPERTIES, &to_canonical_json(p))?,
        None => match fs::remove_file(&props) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(io_error(&props, &e)),
        },
    }
    Ok(())
}

/// The folder extension of each document kind (FILE_FORMAT.md,
/// "Extensions").
pub(crate) fn extension_of(kind: DocumentKind) -> &'static str {
    match kind {
        DocumentKind::Part => "sltwx",
        DocumentKind::Assembly => "sltwxa",
        DocumentKind::Drawing => "sltwxd",
    }
}

pub(crate) fn check_extension(
    path: &Path,
    form: DocumentForm,
    kind: DocumentKind,
) -> Result<(), FormatError> {
    let ext = path.extension().map(|e| e.to_string_lossy().into_owned());
    if ext.as_deref() == Some(extension_of(kind)) {
        Ok(())
    } else {
        Err(FormatError::ExtensionMismatch {
            form,
            extension: ext,
            kind,
        })
    }
}

pub(crate) fn io_error(path: &Path, e: &io::Error) -> FormatError {
    FormatError::Io {
        path: path.to_path_buf(),
        message: e.to_string(),
    }
}

/// The text of `dir/name`, or `None` if it doesn't exist.
fn read_file(dir: &Path, name: &str) -> Result<Option<String>, FormatError> {
    let path = dir.join(name);
    match fs::read_to_string(&path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io_error(&path, &e)),
    }
}

fn write_file(dir: &Path, name: &str, text: &str) -> Result<(), FormatError> {
    let path = dir.join(name);
    fs::write(&path, text).map_err(|e| io_error(&path, &e))
}

pub(crate) fn check_object(file: &str, v: &Value) -> Result<(), FormatError> {
    if v.is_object() {
        Ok(())
    } else {
        Err(FormatError::InvalidField {
            field: file.into(),
            reason: "must be a JSON object".into(),
        })
    }
}

pub(crate) fn parse_object(file: &str, text: &str) -> Result<Value, FormatError> {
    let v: Value =
        serde_json::from_str(text).map_err(|e| FormatError::Json(format!("{file}: {e}")))?;
    check_object(file, &v)?;
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A fresh, uniquely named directory under the system temp dir, removed
    /// when dropped (also when a test fails).
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> TempDir {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("swx-io-test-{}-{n}-{tag}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            TempDir(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/bracket.sltwx")
    }

    /// Copies the fixture's three files into `dir`, creating it.
    fn copy_fixture(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        for name in [MANIFEST, MODEL, PROPERTIES] {
            fs::copy(fixture().join(name), dir.join(name)).unwrap();
        }
    }

    #[test]
    fn fixture_round_trips_byte_for_byte() {
        let doc = read_unpacked(&fixture()).unwrap();
        assert_eq!(doc.manifest.kind, DocumentKind::Part);
        assert_eq!(doc.model["end_of_part"], "ft-91c2");
        assert_eq!(doc.properties.as_ref().unwrap()["title"], "Bracket");
        let tmp = TempDir::new("golden");
        let out = tmp.0.join("bracket.sltwx");
        write_unpacked(&out, &doc).unwrap();
        for name in [MANIFEST, MODEL, PROPERTIES] {
            assert_eq!(
                fs::read(out.join(name)).unwrap(),
                fs::read(fixture().join(name)).unwrap(),
                "{name}"
            );
        }
        assert_eq!(fs::read_dir(&out).unwrap().count(), 3);
        assert_eq!(read_unpacked(&out).unwrap(), doc);
    }

    #[test]
    fn unknown_model_fields_round_trip_unchanged() {
        let mut doc = read_unpacked(&fixture()).unwrap();
        doc.model
            .as_object_mut()
            .unwrap()
            .insert("x_future".into(), serde_json::json!({"a": 1}));
        let tmp = TempDir::new("future");
        let out = tmp.0.join("bracket.sltwx");
        write_unpacked(&out, &doc).unwrap();
        let text = fs::read_to_string(out.join(MODEL)).unwrap();
        assert!(
            text.contains("\"x_future\": {\n    \"a\": 1\n  }\n}\n"),
            "{text}"
        );
        let back = read_unpacked(&out).unwrap();
        assert_eq!(back.model["x_future"], serde_json::json!({"a": 1}));
        assert_eq!(back, doc);
    }

    #[test]
    fn properties_none_removes_the_file_and_leaves_other_entries() {
        let tmp = TempDir::new("props");
        let dir = tmp.0.join("bracket.sltwx");
        copy_fixture(&dir);
        fs::create_dir_all(dir.join("cache")).unwrap();
        let brep: Vec<u8> = (0..=255u8).rev().collect();
        fs::write(dir.join("cache/body-b-1.brep"), &brep).unwrap();
        let png = b"\x89PNG\r\n\x1a\nnot really a png";
        fs::write(dir.join("thumbnail.png"), png).unwrap();
        fs::write(dir.join("notes.txt"), "keep me").unwrap();

        let mut doc = read_unpacked(&dir).unwrap();
        doc.properties = None;
        write_unpacked(&dir, &doc).unwrap();
        assert!(!dir.join(PROPERTIES).exists());
        assert_eq!(fs::read(dir.join("cache/body-b-1.brep")).unwrap(), brep);
        assert_eq!(fs::read(dir.join("thumbnail.png")).unwrap(), png);
        assert_eq!(
            fs::read_to_string(dir.join("notes.txt")).unwrap(),
            "keep me"
        );
        assert_eq!(read_unpacked(&dir).unwrap().properties, None);
        // Writing `None` again, with no file to remove, is fine.
        write_unpacked(&dir, &doc).unwrap();
    }

    #[test]
    fn write_creates_missing_parent_folders() {
        let tmp = TempDir::new("nested");
        let out = tmp.0.join("a/b/bracket.sltwx");
        let doc = read_unpacked(&fixture()).unwrap();
        write_unpacked(&out, &doc).unwrap();
        assert_eq!(read_unpacked(&out).unwrap(), doc);
    }

    #[test]
    fn missing_files_are_named() {
        let tmp = TempDir::new("missing");
        let dir = tmp.0.join("bracket.sltwx");
        copy_fixture(&dir);
        fs::remove_file(dir.join(MODEL)).unwrap();
        assert_eq!(
            read_unpacked(&dir),
            Err(FormatError::MissingFile("model.json".into()))
        );
        fs::remove_file(dir.join(MANIFEST)).unwrap();
        assert_eq!(
            read_unpacked(&dir),
            Err(FormatError::MissingFile("manifest.json".into()))
        );
    }

    #[test]
    fn model_and_properties_must_be_objects() {
        let tmp = TempDir::new("objects");
        let dir = tmp.0.join("bracket.sltwx");
        copy_fixture(&dir);
        fs::write(dir.join(MODEL), "[1, 2]").unwrap();
        assert!(matches!(
            read_unpacked(&dir),
            Err(FormatError::InvalidField { field, .. }) if field == "model.json"
        ));
        fs::copy(fixture().join(MODEL), dir.join(MODEL)).unwrap();
        fs::write(dir.join(PROPERTIES), "\"Bracket\"").unwrap();
        assert!(matches!(
            read_unpacked(&dir),
            Err(FormatError::InvalidField { field, .. }) if field == "properties.json"
        ));
        fs::write(dir.join(PROPERTIES), "{").unwrap();
        assert!(matches!(
            read_unpacked(&dir),
            Err(FormatError::Json(m)) if m.starts_with("properties.json: ")
        ));
        // The same rule on write, before anything is written.
        let mut doc = read_unpacked(&fixture()).unwrap();
        doc.model = serde_json::json!([1, 2]);
        let out = tmp.0.join("never.sltwx");
        assert!(matches!(
            write_unpacked(&out, &doc),
            Err(FormatError::InvalidField { field, .. }) if field == "model.json"
        ));
        assert!(!out.exists());
    }

    #[test]
    fn extension_must_match_the_kind() {
        let tmp = TempDir::new("ext");
        let dir = tmp.0.join("x.sltwx");
        copy_fixture(&dir);
        let text = fs::read_to_string(dir.join(MANIFEST)).unwrap();
        fs::write(
            dir.join(MANIFEST),
            text.replace("\"kind\": \"part\"", "\"kind\": \"assembly\""),
        )
        .unwrap();
        assert_eq!(
            read_unpacked(&dir),
            Err(FormatError::ExtensionMismatch {
                form: DocumentForm::Folder,
                extension: Some("sltwx".into()),
                kind: DocumentKind::Assembly,
            })
        );
        // The assembly extension is accepted for that manifest.
        let asm = tmp.0.join("x.sltwxa");
        fs::rename(&dir, &asm).unwrap();
        assert_eq!(
            read_unpacked(&asm).unwrap().manifest.kind,
            DocumentKind::Assembly
        );
        // No extension at all.
        let bare = tmp.0.join("bracket");
        copy_fixture(&bare);
        assert_eq!(
            read_unpacked(&bare),
            Err(FormatError::ExtensionMismatch {
                form: DocumentForm::Folder,
                extension: None,
                kind: DocumentKind::Part,
            })
        );
        assert_eq!(
            read_unpacked(&bare).unwrap_err().to_string(),
            "folder has no extension; the manifest kind `part` needs `.sltwx`"
        );
        // Writing a part to a drawing folder writes nothing, and the message
        // names a folder.
        let doc = read_unpacked(&fixture()).unwrap();
        let out = tmp.0.join("bracket.sltwxd");
        assert_eq!(
            write_unpacked(&out, &doc).unwrap_err().to_string(),
            "folder extension `.sltwxd` does not match the manifest kind `part` (expected `.sltwx`)"
        );
        assert!(!out.exists());
    }

    #[test]
    fn a_file_or_a_missing_path_is_an_io_error() {
        let tmp = TempDir::new("file");
        let file = tmp.0.join("bracket.sltwx");
        fs::write(&file, "not a folder").unwrap();
        assert_eq!(
            read_unpacked(&file),
            Err(FormatError::Io {
                path: file.clone(),
                message: "not a directory".into(),
            })
        );
        let gone = tmp.0.join("gone.sltwx");
        assert!(matches!(
            read_unpacked(&gone),
            Err(FormatError::Io { path, .. }) if path == gone
        ));
        // Writing into a path that is a file fails cleanly too.
        let doc = read_unpacked(&fixture()).unwrap();
        assert!(matches!(
            write_unpacked(&file, &doc),
            Err(FormatError::Io { .. })
        ));
        assert_eq!(fs::read_to_string(&file).unwrap(), "not a folder");
    }

    #[test]
    fn new_errors_display_their_subject() {
        let e = FormatError::Io {
            path: PathBuf::from("/tmp/x.sltwx"),
            message: "not a directory".into(),
        };
        assert_eq!(e.to_string(), "/tmp/x.sltwx: not a directory");
        assert_eq!(
            FormatError::MissingFile("model.json".into()).to_string(),
            "missing required file `model.json`"
        );
        let e = FormatError::ExtensionMismatch {
            form: DocumentForm::Folder,
            extension: Some("sltwx".into()),
            kind: DocumentKind::Assembly,
        };
        assert_eq!(
            e.to_string(),
            "folder extension `.sltwx` does not match the manifest kind `assembly` (expected `.sltwxa`)"
        );
        let e = FormatError::ExtensionMismatch {
            form: DocumentForm::Folder,
            extension: None,
            kind: DocumentKind::Part,
        };
        assert_eq!(
            e.to_string(),
            "folder has no extension; the manifest kind `part` needs `.sltwx`"
        );
    }

    #[test]
    fn refreshed_model_hash_keeps_the_cache_fresh_until_the_model_changes() {
        let mut doc = read_unpacked(&fixture()).unwrap();
        doc.refresh_model_hash();
        let recorded = doc.manifest.model_hash.clone().unwrap();
        assert_eq!(recorded, model_hash(&doc.model));
        // The refreshed hash passes the manifest's own `model_hash` check.
        let reread = read_manifest(&write_manifest(&doc.manifest)).unwrap();
        assert_eq!(reread.model_hash.as_deref(), Some(recorded.as_str()));
        assert!(doc.cache_is_fresh(&recorded));

        let obj = doc.model.as_object_mut().unwrap();
        obj.insert("end_of_part".into(), Value::String("ft-other".into()));
        assert!(!doc.cache_is_fresh(&recorded));
        assert!(!doc.cache_is_fresh("sha256:xyz"));
        assert!(!doc.cache_is_fresh(""));
    }

    #[test]
    fn refreshed_model_hash_matches_written_model_json_for_unsorted_part() {
        let mut doc = read_unpacked(&fixture()).unwrap();
        // Append a body whose id sorts before `b-1`: the set is now out of
        // `id` order, and the writers sort it.
        let bodies = doc.model["bodies"].as_array_mut().unwrap();
        let mut early = bodies[0].clone();
        early["id"] = Value::String("b-0".into());
        bodies.push(early);
        assert_ne!(
            to_canonical_json(&doc.model),
            to_canonical_json(&model_for_write(doc.manifest.kind, &doc.model).unwrap())
        );

        doc.refresh_model_hash();
        let recorded = doc.manifest.model_hash.clone().unwrap();
        let tmp = TempDir::new("unsorted-hash");
        let out = tmp.0.join("bracket.sltwx");
        write_unpacked(&out, &doc).unwrap();
        let written = fs::read(out.join(MODEL)).unwrap();
        assert_eq!(
            recorded,
            format!("sha256:{}", crate::hash::hex(&crate::sha256(&written)))
        );
        assert!(doc.cache_is_fresh(&recorded));
        assert_ne!(recorded, model_hash(&doc.model));
    }
}
