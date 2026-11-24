//! The unpacked (git-friendly) form of a `.sltwx` document: a directory
//! `<name>.sltwx/` (`.sltwxa`, `.sltwxd`) with the container's layout
//! (`docs/specs/FILE_FORMAT.md`, "Container" and "Unpacked (git-friendly)
//! form").

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde_json::Value;

use crate::cache::{
    check_cache, index_json, is_entry_path, read_cache, Cache, CACHE_DIR, INDEX, THUMBNAIL,
};
use crate::ids::{check_ids, model_for_write};
use crate::{
    model_hash, read_manifest, to_canonical_json, write_manifest, DocumentForm, DocumentKind,
    FormatError, Manifest,
};

const MANIFEST: &str = "manifest.json";
const MODEL: &str = "model.json";
const PROPERTIES: &str = "properties.json";
/// The folder (and ZIP name prefix) of embedded files (FILE_FORMAT.md,
/// "Container"; `docs/specs/M4-03.md` §1).
pub(crate) const ATTACHMENTS: &str = "attachments";

/// A whole `.sltwx` document: its three JSON files, its embedded files and
/// its derived cache.
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
    /// The files under `attachments/`, by file name (e.g. `"ft-3b10.step"`
    /// for `attachments/ft-3b10.step`), byte for byte. Names are plain file
    /// names: non-empty, no `/` or `\`, not `.` or `..`.
    pub attachments: BTreeMap<String, Vec<u8>>,
    /// Derived files (`cache/…` and `thumbnail.png`, recorded in
    /// `cache/index.json`; `docs/specs/M4-04.md` §2). `None`: the document
    /// has no readable cache index.
    pub cache: Option<Cache>,
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

    /// The bytes of the cache file `path` (e.g. `"cache/display.glb"`) when
    /// its entry is fresh, else `None` (`docs/specs/M4-04.md` §1 checks 1–4):
    /// `cache` has an entry at a valid path, and its `model_hash` is the
    /// hash of the model as the writers would write it now
    /// ([`Document::cache_is_fresh`]). Checks 2–3 (the file exists and
    /// matches its recorded SHA-256) were made when the cache was read.
    pub fn fresh_cache_file(&self, path: &str) -> Option<&[u8]> {
        let entry = self.cache.as_ref()?.entries.get(path)?;
        (is_entry_path(path) && self.cache_is_fresh(&entry.model_hash))
            .then_some(entry.bytes.as_slice())
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
/// optional. The files directly in `attachments/` are read into
/// [`Document::attachments`]. `cache/index.json` and the files it lists are
/// read into [`Document::cache`]: a missing or unreadable index gives
/// `None`, and an entry with a bad path or hash, or whose file is missing or
/// doesn't match its SHA-256, is dropped, never an error
/// (`docs/specs/M4-04.md` §4). Other entries (unlisted files in `cache/`,
/// subfolders of `attachments/`) are not read.
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
    let attachments = read_attachments(dir)?;
    let cache = read_cache(fs::read(dir.join(INDEX)).ok(), |path| {
        fs::read(dir.join(path)).ok()
    });
    Ok(Document {
        manifest,
        model,
        properties,
        attachments,
        cache,
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
/// when `doc.properties` is `Some` and removed when it is `None`. Each of
/// `doc.attachments` is written to `attachments/<name>` in name order, and
/// files directly in `attachments/` that are not in the map are removed
/// (the folder too, when it ends up empty).
///
/// With `doc.cache` `None`, `cache/` and `thumbnail.png` are left as they
/// are. With `Some(c)`, each entry's bytes are written at its path and
/// `cache/index.json` is written from `c`; the other files in `cache/` are
/// removed, and `thumbnail.png` too when `c` has no entry for it, so the
/// folder matches `c` (`docs/specs/M4-04.md` §2). The attachment and cache
/// cleanups each stay inside their own folder, and no other entry in `dir`
/// is touched, so unknown files stay as they were.
///
/// # Errors
///
/// Nothing is written when `dir`'s extension doesn't match the manifest's
/// `kind` ([`FormatError::ExtensionMismatch`]) or `model` or `properties` is
/// not a JSON object ([`FormatError::InvalidField`] naming the file), or a
/// part's IDs are invalid ([`FormatError::InvalidId`],
/// [`FormatError::DuplicateId`]), since [`read_unpacked`] would reject the
/// result, an attachment name is not a plain file name
/// ([`FormatError::InvalidField`] naming `attachments`), or a cache entry
/// has a bad path or `model_hash` ([`FormatError::InvalidField`] naming
/// `cache`). Otherwise
/// [`FormatError::Io`] when `dir` or a file can't be created, written or
/// removed.
pub fn write_unpacked(dir: &Path, doc: &Document) -> Result<(), FormatError> {
    check_extension(dir, DocumentForm::Folder, doc.manifest.kind)?;
    check_object(MODEL, &doc.model)?;
    if let Some(p) = &doc.properties {
        check_object(PROPERTIES, p)?;
    }
    check_attachment_names(&doc.attachments)?;
    if let Some(c) = &doc.cache {
        check_cache(c)?;
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
    write_attachments(dir, &doc.attachments)?;
    match &doc.cache {
        Some(c) => write_cache(dir, c),
        None => Ok(()),
    }
}

/// Checks that every attachment name is a plain file name, so it stays
/// inside `attachments/` in both container forms.
pub(crate) fn check_attachment_names(
    attachments: &BTreeMap<String, Vec<u8>>,
) -> Result<(), FormatError> {
    for name in attachments.keys() {
        if !is_plain_name(name) {
            return Err(FormatError::InvalidField {
                field: ATTACHMENTS.into(),
                reason: format!("{name:?} is not a plain file name"),
            });
        }
    }
    Ok(())
}

/// Whether `name` is a non-empty file name with no path separator that is
/// not `.` or `..`.
pub(crate) fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\'])
}

/// The regular files directly in `dir/attachments/`, by name. A missing
/// folder gives an empty map.
fn read_attachments(dir: &Path) -> Result<BTreeMap<String, Vec<u8>>, FormatError> {
    let folder = dir.join(ATTACHMENTS);
    let mut out = BTreeMap::new();
    let entries = match fs::read_dir(&folder) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(io_error(&folder, &e)),
    };
    for entry in entries {
        let entry = entry.map_err(|e| io_error(&folder, &e))?;
        let path = entry.path();
        let is_file = entry
            .file_type()
            .map_err(|e| io_error(&path, &e))?
            .is_file();
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if is_file && is_plain_name(&name) {
            let bytes = fs::read(&path).map_err(|e| io_error(&path, &e))?;
            out.insert(name, bytes);
        }
    }
    Ok(out)
}

/// Makes the files directly in `dir/attachments/` equal `attachments`.
fn write_attachments(
    dir: &Path,
    attachments: &BTreeMap<String, Vec<u8>>,
) -> Result<(), FormatError> {
    let folder = dir.join(ATTACHMENTS);
    if !attachments.is_empty() {
        fs::create_dir_all(&folder).map_err(|e| io_error(&folder, &e))?;
    }
    for (name, bytes) in attachments {
        let path = folder.join(name);
        fs::write(&path, bytes).map_err(|e| io_error(&path, &e))?;
    }
    let entries = match fs::read_dir(&folder) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(io_error(&folder, &e)),
    };
    for entry in entries {
        let entry = entry.map_err(|e| io_error(&folder, &e))?;
        let path = entry.path();
        let is_file = entry
            .file_type()
            .map_err(|e| io_error(&path, &e))?
            .is_file();
        let stale = match entry.file_name().to_str() {
            Some(name) => !attachments.contains_key(name),
            None => false,
        };
        if is_file && stale {
            fs::remove_file(&path).map_err(|e| io_error(&path, &e))?;
        }
    }
    if attachments.is_empty() {
        // Only succeeds when nothing else (a subfolder) is left in it.
        let _ = fs::remove_dir(&folder);
    }
    Ok(())
}

/// Writes `cache` into `dir` so that `cache/` and `thumbnail.png` match it.
fn write_cache(dir: &Path, cache: &Cache) -> Result<(), FormatError> {
    let folder = dir.join(CACHE_DIR);
    fs::create_dir_all(&folder).map_err(|e| io_error(&folder, &e))?;
    for (path, entry) in &cache.entries {
        let file = dir.join(path);
        fs::write(&file, &entry.bytes).map_err(|e| io_error(&file, &e))?;
    }
    let index = dir.join(INDEX);
    fs::write(&index, index_json(cache)).map_err(|e| io_error(&index, &e))?;
    for entry in fs::read_dir(&folder).map_err(|e| io_error(&folder, &e))? {
        let entry = entry.map_err(|e| io_error(&folder, &e))?;
        let path = entry.path();
        let is_file = entry
            .file_type()
            .map_err(|e| io_error(&path, &e))?
            .is_file();
        let listed = entry.file_name().to_str().is_some_and(|name| {
            name == "index.json" || cache.entries.contains_key(&format!("{CACHE_DIR}/{name}"))
        });
        if is_file && !listed {
            fs::remove_file(&path).map_err(|e| io_error(&path, &e))?;
        }
    }
    if !cache.entries.contains_key(THUMBNAIL) {
        let thumb = dir.join(THUMBNAIL);
        match fs::remove_file(&thumb) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(io_error(&thumb, &e)),
        }
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
pub(crate) mod tests {
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

    /// A 3-byte and a 100 KB attachment (M4-03 spec §6, test 1).
    pub(crate) fn two_attachments() -> BTreeMap<String, Vec<u8>> {
        let big: Vec<u8> = (0..100 * 1024u32)
            .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8)
            .collect();
        BTreeMap::from([
            ("z-small.bin".to_owned(), vec![0, 255, 10]),
            ("a-big.step".to_owned(), big),
        ])
    }

    #[test]
    fn attachments_round_trip_byte_for_byte_and_stale_files_go() {
        let mut doc = read_unpacked(&fixture()).unwrap();
        assert!(doc.attachments.is_empty());
        doc.attachments = two_attachments();
        let tmp = TempDir::new("attach");
        let out = tmp.0.join("bracket.sltwx");
        write_unpacked(&out, &doc).unwrap();
        for (name, bytes) in &doc.attachments {
            assert_eq!(&fs::read(out.join(ATTACHMENTS).join(name)).unwrap(), bytes);
        }
        let back = read_unpacked(&out).unwrap();
        assert_eq!(back.attachments, doc.attachments);
        assert_eq!(
            back.attachments.keys().collect::<Vec<_>>(),
            ["a-big.step", "z-small.bin"]
        );
        assert_eq!(back, doc);
        // The other three files are unchanged by having attachments.
        for name in [MANIFEST, MODEL, PROPERTIES] {
            assert_eq!(
                fs::read(out.join(name)).unwrap(),
                fs::read(fixture().join(name)).unwrap(),
                "{name}"
            );
        }

        // Dropping one removes its file; dropping all removes the folder.
        doc.attachments.remove("z-small.bin");
        write_unpacked(&out, &doc).unwrap();
        assert!(!out.join(ATTACHMENTS).join("z-small.bin").exists());
        assert_eq!(read_unpacked(&out).unwrap(), doc);
        doc.attachments.clear();
        write_unpacked(&out, &doc).unwrap();
        assert!(!out.join(ATTACHMENTS).exists());
    }

    #[test]
    fn attachment_names_must_be_plain() {
        let mut doc = read_unpacked(&fixture()).unwrap();
        let tmp = TempDir::new("badname");
        let out = tmp.0.join("bracket.sltwx");
        for bad in ["", ".", "..", "x/y.step", "x\\y.step"] {
            doc.attachments = BTreeMap::from([(bad.to_owned(), vec![1])]);
            let err = write_unpacked(&out, &doc).unwrap_err();
            assert!(
                matches!(&err, FormatError::InvalidField { field, .. } if field == ATTACHMENTS),
                "{bad:?}: {err}"
            );
        }
        assert!(!out.exists());
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

    /// The golden part with a fresh 3-entry cache (M4-04 spec §6 test 1).
    fn cached_doc() -> Document {
        use crate::{Cache, CacheEntry};
        use std::collections::BTreeMap;
        let mut doc = read_unpacked(&fixture()).unwrap();
        doc.refresh_model_hash();
        let hash = doc.manifest.model_hash.clone().unwrap();
        let entry = |bytes: Vec<u8>| CacheEntry {
            model_hash: hash.clone(),
            bytes,
        };
        let glb: Vec<u8> = (0..1024u32).map(|i| (i * 7 % 251) as u8).collect();
        doc.cache = Some(Cache {
            entries: BTreeMap::from([
                ("cache/body-b-1.brep".to_owned(), entry(vec![1, 2, 3])),
                ("cache/display.glb".to_owned(), entry(glb)),
                (
                    "thumbnail.png".to_owned(),
                    entry(b"\x89PNG\r\n\x1a\nthumb".to_vec()),
                ),
            ]),
        });
        doc
    }

    fn cache_paths(doc: &Document) -> Vec<&str> {
        doc.cache
            .as_ref()
            .unwrap()
            .entries
            .keys()
            .map(String::as_str)
            .collect()
    }

    /// Writes `doc` unpacked to a fresh folder and returns it.
    fn written(tmp: &TempDir, doc: &Document) -> PathBuf {
        let dir = tmp.0.join("bracket.sltwx");
        write_unpacked(&dir, doc).unwrap();
        dir
    }

    #[test]
    fn cache_round_trips_in_both_forms_and_index_is_canonical() {
        let doc = cached_doc();
        let tmp = TempDir::new("cache-rt");
        let dir = written(&tmp, &doc);
        let back = read_unpacked(&dir).unwrap();
        assert_eq!(back, doc);
        let index = fs::read_to_string(dir.join(INDEX)).unwrap();
        let v: Value = serde_json::from_str(&index).unwrap();
        assert_eq!(v["cache_format"], 1);
        let entries = v["entries"].as_array().unwrap();
        let paths: Vec<&str> = entries
            .iter()
            .map(|e| e["path"].as_str().unwrap())
            .collect();
        assert_eq!(
            paths,
            ["cache/body-b-1.brep", "cache/display.glb", "thumbnail.png"]
        );
        for e in entries {
            let entry = &doc.cache.as_ref().unwrap().entries[e["path"].as_str().unwrap()];
            assert_eq!(e["model_hash"], json_str(&entry.model_hash));
            assert_eq!(e["sha256"], json_str(&crate::cache::hash_of(&entry.bytes)));
        }
        assert_eq!(index, to_canonical_json(&v));
        assert_eq!(
            fs::read(dir.join("thumbnail.png")).unwrap(),
            b"\x89PNG\r\n\x1a\nthumb"
        );
        // The JSON files are as without a cache.
        for name in [MANIFEST, MODEL, PROPERTIES] {
            let mut plain = doc.clone();
            plain.cache = None;
            let other = TempDir::new("cache-plain");
            let p = written(&other, &plain);
            assert_eq!(
                fs::read(dir.join(name)).unwrap(),
                fs::read(p.join(name)).unwrap()
            );
        }
        // Writing twice is byte-identical.
        write_unpacked(&dir, &back).unwrap();
        assert_eq!(fs::read_to_string(dir.join(INDEX)).unwrap(), index);

        let zip = tmp.0.join("packed.sltwx");
        crate::write_sltwx(&zip, &doc).unwrap();
        let first = fs::read(&zip).unwrap();
        assert_eq!(crate::read_sltwx(&zip).unwrap(), doc);
        crate::write_sltwx(&zip, &doc).unwrap();
        assert_eq!(fs::read(&zip).unwrap(), first);
    }

    fn json_str(s: &str) -> Value {
        Value::String(s.to_owned())
    }

    #[test]
    fn model_edit_makes_the_cache_stale() {
        let mut doc = cached_doc();
        let paths: Vec<String> = cache_paths(&doc).iter().map(|p| p.to_string()).collect();
        for p in &paths {
            let want = &doc.cache.as_ref().unwrap().entries[p].bytes;
            assert_eq!(doc.fresh_cache_file(p), Some(want.as_slice()), "{p}");
        }
        assert_eq!(doc.fresh_cache_file("cache/missing.bin"), None);
        doc.model["parameters"][0]["expr"] = json_str("41 mm");
        for p in &paths {
            assert_eq!(doc.fresh_cache_file(p), None, "{p}");
        }
        assert_eq!(doc.cache.as_ref().unwrap().entries.len(), 3);
    }

    #[test]
    fn damaged_cache_file_drops_only_that_entry() {
        let tmp = TempDir::new("cache-damaged");
        let dir = written(&tmp, &cached_doc());
        let glb = dir.join("cache/display.glb");
        let mut bytes = fs::read(&glb).unwrap();
        bytes[100] ^= 1;
        fs::write(&glb, bytes).unwrap();
        let back = read_unpacked(&dir).unwrap();
        assert_eq!(cache_paths(&back), ["cache/body-b-1.brep", "thumbnail.png"]);
    }

    #[test]
    fn unreadable_index_reads_as_no_cache() {
        let tmp = TempDir::new("cache-index");
        let dir = written(&tmp, &cached_doc());
        for text in ["{", "[]", r#"{"cache_format": 2, "entries": []}"#] {
            fs::write(dir.join(INDEX), text).unwrap();
            assert_eq!(read_unpacked(&dir).unwrap().cache, None, "{text}");
        }
    }

    #[test]
    fn bad_index_entries_drop_only_themselves() {
        let tmp = TempDir::new("cache-entries");
        let doc = cached_doc();
        let dir = written(&tmp, &doc);
        let good: Value = serde_json::from_slice(&fs::read(dir.join(INDEX)).unwrap()).unwrap();
        let hash = doc.manifest.model_hash.clone().unwrap();
        let some = &good["entries"][0];
        let mut bad = vec![];
        for path in ["../x", "cache/index.json", "cache/a/b"] {
            let mut e = some.clone();
            e["path"] = json_str(path);
            bad.push(e);
        }
        let mut no_prefix = good["entries"][2].clone();
        let sha = no_prefix["sha256"].as_str().unwrap()["sha256:".len()..].to_owned();
        no_prefix["sha256"] = json_str(&sha);
        // Files exist for every bad path that stays inside the folder.
        fs::create_dir_all(dir.join("cache/a")).unwrap();
        fs::write(dir.join("cache/a/b"), [1, 2, 3]).unwrap();
        // `../x` resolves to a real file with the same bytes as the body, so
        // only the path check can drop it.
        fs::write(tmp.0.join("x"), [1, 2, 3]).unwrap();
        for e in bad.into_iter().chain([no_prefix]) {
            let mut v = good.clone();
            let entries = v["entries"].as_array_mut().unwrap();
            // A bad entry replaces the thumbnail's when it names it.
            if e["path"] == json_str("thumbnail.png") {
                entries.remove(2);
            }
            entries.push(e.clone());
            fs::write(dir.join(INDEX), to_canonical_json(&v)).unwrap();
            let back = read_unpacked(&dir).unwrap();
            let want: Vec<&str> = if e["path"] == json_str("thumbnail.png") {
                vec!["cache/body-b-1.brep", "cache/display.glb"]
            } else {
                vec!["cache/body-b-1.brep", "cache/display.glb", "thumbnail.png"]
            };
            assert_eq!(cache_paths(&back), want, "{e}");
            assert!(back
                .cache
                .unwrap()
                .entries
                .values()
                .all(|c| c.model_hash == hash));
        }
    }

    #[test]
    fn unlisted_cache_file_is_not_read_and_writing_a_cache_removes_it() {
        let tmp = TempDir::new("cache-extra");
        let doc = cached_doc();
        let dir = written(&tmp, &doc);
        fs::write(dir.join("cache/extra.bin"), b"extra").unwrap();
        let back = read_unpacked(&dir).unwrap();
        assert_eq!(back, doc);
        write_unpacked(&dir, &back).unwrap();
        assert!(!dir.join("cache/extra.bin").exists());
        // A cache without a thumbnail removes `thumbnail.png`.
        let mut no_thumb = doc.clone();
        no_thumb
            .cache
            .as_mut()
            .unwrap()
            .entries
            .remove("thumbnail.png");
        write_unpacked(&dir, &no_thumb).unwrap();
        assert!(!dir.join("thumbnail.png").exists());
        assert_eq!(read_unpacked(&dir).unwrap(), no_thumb);
    }

    #[test]
    fn no_cache_leaves_an_existing_cache_folder_untouched() {
        let tmp = TempDir::new("cache-none");
        let dir = written(&tmp, &cached_doc());
        fs::write(dir.join("cache/extra.bin"), b"extra").unwrap();
        let before: Vec<(PathBuf, Vec<u8>)> = [
            "cache/index.json",
            "cache/display.glb",
            "cache/extra.bin",
            "thumbnail.png",
        ]
        .iter()
        .map(|p| (dir.join(p), fs::read(dir.join(p)).unwrap()))
        .collect();
        let mut doc = read_unpacked(&dir).unwrap();
        doc.cache = None;
        write_unpacked(&dir, &doc).unwrap();
        for (path, bytes) in before {
            assert_eq!(fs::read(&path).unwrap(), bytes, "{}", path.display());
        }
    }

    #[test]
    fn attachments_and_cache_live_side_by_side() {
        let mut doc = cached_doc();
        doc.attachments = BTreeMap::from([
            ("b.step".to_owned(), b"step-b".to_vec()),
            ("a.step".to_owned(), b"step-a".to_vec()),
        ]);
        let tmp = TempDir::new("cache-attach");
        let dir = written(&tmp, &doc);
        assert_eq!(read_unpacked(&dir).unwrap(), doc);

        // Each folder's stale-file cleanup leaves the other folder alone.
        fs::write(dir.join("cache/extra.bin"), b"extra").unwrap();
        fs::write(dir.join("attachments/old.step"), b"old").unwrap();
        let mut fewer = doc.clone();
        fewer.attachments.remove("b.step");
        fewer
            .cache
            .as_mut()
            .unwrap()
            .entries
            .remove("cache/display.glb");
        write_unpacked(&dir, &fewer).unwrap();
        assert!(!dir.join("cache/extra.bin").exists());
        assert!(!dir.join("cache/display.glb").exists());
        assert!(!dir.join("attachments/old.step").exists());
        assert!(!dir.join("attachments/b.step").exists());
        assert_eq!(fs::read(dir.join("attachments/a.step")).unwrap(), b"step-a");
        assert_eq!(
            fs::read(dir.join("cache/body-b-1.brep")).unwrap(),
            [1, 2, 3]
        );
        assert_eq!(read_unpacked(&dir).unwrap(), fewer);

        // No cache: attachments still sync, `cache/` stays as it was.
        let index = fs::read(dir.join(INDEX)).unwrap();
        let mut no_cache = fewer.clone();
        no_cache.cache = None;
        no_cache.attachments.clear();
        write_unpacked(&dir, &no_cache).unwrap();
        assert!(!dir.join(ATTACHMENTS).exists());
        assert_eq!(fs::read(dir.join(INDEX)).unwrap(), index);

        // ZIP order: JSON files, attachments by name, cache by path, index last.
        let zip_path = tmp.0.join("packed.sltwx");
        crate::write_sltwx(&zip_path, &doc).unwrap();
        let bytes = fs::read(&zip_path).unwrap();
        let names: Vec<String> = crate::zip::list(&bytes)
            .unwrap()
            .iter()
            .map(|e| e.name().to_owned())
            .collect();
        assert_eq!(
            names,
            [
                MANIFEST,
                MODEL,
                PROPERTIES,
                "attachments/a.step",
                "attachments/b.step",
                "cache/body-b-1.brep",
                "cache/display.glb",
                "thumbnail.png",
                INDEX,
            ]
        );
        assert_eq!(crate::read_sltwx(&zip_path).unwrap(), doc);
    }

    #[test]
    fn writing_a_bad_cache_path_writes_nothing() {
        let mut doc = cached_doc();
        let entries = &mut doc.cache.as_mut().unwrap().entries;
        let e = entries["thumbnail.png"].clone();
        entries.insert("cache/index.json".into(), e);
        let tmp = TempDir::new("cache-bad");
        let dir = tmp.0.join("bracket.sltwx");
        let err = write_unpacked(&dir, &doc).unwrap_err();
        assert!(matches!(&err, FormatError::InvalidField { field, .. } if field == "cache"));
        assert!(!dir.exists());
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
