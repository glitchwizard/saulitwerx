//! The packed form of a `.sltwx` document: a single ZIP archive
//! (`docs/specs/FILE_FORMAT.md`, "Container").

use std::fs;
use std::path::Path;

use crate::unpacked::{check_extension, check_object, io_error, parse_object};
use crate::zip;
use crate::{read_manifest, to_canonical_json, write_manifest, Document, FormatError};

const MANIFEST: &str = "manifest.json";
const MODEL: &str = "model.json";
const PROPERTIES: &str = "properties.json";

/// Reads the document in the ZIP archive `path`
/// (`docs/specs/FILE_FORMAT.md`, "Container"; ZIP layout per PKWARE
/// APPNOTE 6.3.x, sections 4.3.7, 4.3.12 and 4.3.16).
///
/// `manifest.json` and `model.json` are required; `properties.json` is
/// optional. Other entries (`cache/…`, `thumbnail.png`, `attachments/…`)
/// are ignored. Entries may be stored (method 0) or deflated (method 8);
/// each entry read is checked against its CRC-32 and uncompressed size.
///
/// # Errors
///
/// - [`FormatError::Io`]: the file can't be read.
/// - [`FormatError::MalformedArchive`]: not a ZIP archive (no end-of-central-
///   directory record), a truncated entry, corrupt deflate data, or an
///   unsupported feature (ZIP64, encryption, multi-disk).
/// - [`FormatError::UnsupportedMethod`]: a needed entry uses a compression
///   method other than 0 or 8.
/// - [`FormatError::CrcMismatch`]: a needed entry's data fails its CRC-32.
/// - [`FormatError::MissingFile`]: no `manifest.json` or `model.json`.
/// - [`FormatError::Json`] / [`FormatError::InvalidField`]: as for
///   [`read_unpacked`](crate::read_unpacked).
/// - [`FormatError::ExtensionMismatch`]: `path`'s extension doesn't match
///   the manifest's `kind`, which is authoritative.
pub fn read_sltwx(path: &Path) -> Result<Document, FormatError> {
    let bytes = fs::read(path).map_err(|e| io_error(path, &e))?;
    let entries = zip::list(&bytes)?;
    let get = |name: &str| -> Result<Option<String>, FormatError> {
        let Some(entry) = entries.iter().find(|e| e.name() == name) else {
            return Ok(None);
        };
        let data = zip::extract(&bytes, entry)?;
        String::from_utf8(data)
            .map(Some)
            .map_err(|_| FormatError::InvalidField {
                field: name.into(),
                reason: "not valid UTF-8".into(),
            })
    };
    let manifest_text = get(MANIFEST)?.ok_or(FormatError::MissingFile(MANIFEST.into()))?;
    let manifest = read_manifest(&manifest_text).map_err(|e| match e {
        FormatError::Json(m) => FormatError::Json(format!("{MANIFEST}: {m}")),
        other => other,
    })?;
    check_extension(path, manifest.kind)?;
    let model_text = get(MODEL)?.ok_or(FormatError::MissingFile(MODEL.into()))?;
    let model = parse_object(MODEL, &model_text)?;
    let properties = match get(PROPERTIES)? {
        Some(text) => Some(parse_object(PROPERTIES, &text)?),
        None => None,
    };
    Ok(Document {
        manifest,
        model,
        properties,
    })
}

/// Writes `doc` to the ZIP archive `path`
/// (`docs/specs/FILE_FORMAT.md`, "Container"), replacing any file there.
///
/// The entries are `manifest.json`, `model.json`, then `properties.json`
/// when `doc.properties` is `Some`, in that order, each deflated, with the
/// same canonical bytes [`write_unpacked`](crate::write_unpacked) writes.
/// Entry timestamps are fixed (1980-01-01 00:00), so writing the same
/// document twice gives byte-identical files.
///
/// # Errors
///
/// Nothing is written when `path`'s extension doesn't match the manifest's
/// `kind` ([`FormatError::ExtensionMismatch`]) or `model` or `properties`
/// is not a JSON object ([`FormatError::InvalidField`] naming the file).
/// [`FormatError::MalformedArchive`] if an entry is too large for a non-ZIP64
/// archive; [`FormatError::Io`] when the file can't be written.
pub fn write_sltwx(path: &Path, doc: &Document) -> Result<(), FormatError> {
    check_extension(path, doc.manifest.kind)?;
    check_object(MODEL, &doc.model)?;
    if let Some(p) = &doc.properties {
        check_object(PROPERTIES, p)?;
    }
    let manifest = write_manifest(&doc.manifest);
    let model = to_canonical_json(&doc.model);
    let properties = doc.properties.as_ref().map(to_canonical_json);
    let mut entries: Vec<(&str, &[u8])> =
        vec![(MANIFEST, manifest.as_bytes()), (MODEL, model.as_bytes())];
    if let Some(p) = &properties {
        entries.push((PROPERTIES, p.as_bytes()));
    }
    let bytes = zip::write_zip(&entries)?;
    fs::write(path, bytes).map_err(|e| io_error(path, &e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{read_unpacked, DocumentKind};
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A uniquely named file under the system temp dir, removed when dropped
    /// (also when a test fails). `name` keeps its extension.
    struct TempFile(PathBuf);

    impl TempFile {
        fn new(name: &str) -> TempFile {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("swx-io-zip-{}-{n}-{name}", std::process::id()));
            let _ = fs::remove_file(&path);
            TempFile(path)
        }

        fn with_bytes(name: &str, bytes: &[u8]) -> TempFile {
            let f = TempFile::new(name);
            fs::write(&f.0, bytes).unwrap();
            f
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/bracket.sltwx")
    }

    fn fixture_bytes(name: &str) -> Vec<u8> {
        fs::read(fixture().join(name)).unwrap()
    }

    /// An archive entry written by hand, so tests can build archives this
    /// crate's writer never produces.
    struct Raw {
        name: &'static str,
        method: u16,
        crc: u32,
        usize: u32,
        data: Vec<u8>,
        csize: Option<u32>,
    }

    fn stored(name: &'static str, data: &[u8]) -> Raw {
        Raw {
            name,
            method: 0,
            crc: crc32fast::hash(data),
            usize: data.len() as u32,
            data: data.to_vec(),
            csize: None,
        }
    }

    fn p16(out: &mut Vec<u8>, v: u16) {
        out.extend_from_slice(&v.to_le_bytes());
    }

    fn p32(out: &mut Vec<u8>, v: u32) {
        out.extend_from_slice(&v.to_le_bytes());
    }

    fn raw_zip(entries: &[Raw]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut offsets = Vec::new();
        for e in entries {
            offsets.push(out.len() as u32);
            p32(&mut out, 0x0403_4b50);
            p16(&mut out, 20);
            p16(&mut out, 0);
            p16(&mut out, e.method);
            p16(&mut out, 0);
            p16(&mut out, 0x21);
            p32(&mut out, e.crc);
            p32(&mut out, e.csize.unwrap_or(e.data.len() as u32));
            p32(&mut out, e.usize);
            p16(&mut out, e.name.len() as u16);
            p16(&mut out, 0);
            out.extend_from_slice(e.name.as_bytes());
            out.extend_from_slice(&e.data);
        }
        let cd_start = out.len() as u32;
        for (e, offset) in entries.iter().zip(&offsets) {
            p32(&mut out, 0x0201_4b50);
            p16(&mut out, 20);
            p16(&mut out, 20);
            p16(&mut out, 0);
            p16(&mut out, e.method);
            p16(&mut out, 0);
            p16(&mut out, 0x21);
            p32(&mut out, e.crc);
            p32(&mut out, e.csize.unwrap_or(e.data.len() as u32));
            p32(&mut out, e.usize);
            p16(&mut out, e.name.len() as u16);
            p16(&mut out, 0);
            p16(&mut out, 0);
            p16(&mut out, 0);
            p16(&mut out, 0);
            p32(&mut out, 0);
            p32(&mut out, *offset);
            out.extend_from_slice(e.name.as_bytes());
        }
        let cd_size = out.len() as u32 - cd_start;
        p32(&mut out, 0x0605_4b50);
        p16(&mut out, 0);
        p16(&mut out, 0);
        p16(&mut out, entries.len() as u16);
        p16(&mut out, entries.len() as u16);
        p32(&mut out, cd_size);
        p32(&mut out, cd_start);
        p16(&mut out, 0);
        out
    }

    fn golden_doc() -> Document {
        read_unpacked(&fixture()).unwrap()
    }

    fn read_bytes(bytes: &[u8]) -> Result<Document, FormatError> {
        let f = TempFile::with_bytes("case.sltwx", bytes);
        read_sltwx(&f.0)
    }

    #[test]
    fn golden_folder_round_trips_through_zip() {
        let doc = golden_doc();
        let f = TempFile::new("bracket.sltwx");
        write_sltwx(&f.0, &doc).unwrap();
        assert_eq!(read_sltwx(&f.0).unwrap(), doc);
    }

    #[test]
    fn document_without_properties_round_trips() {
        let mut doc = golden_doc();
        doc.properties = None;
        let f = TempFile::new("plain.sltwx");
        write_sltwx(&f.0, &doc).unwrap();
        assert_eq!(read_sltwx(&f.0).unwrap(), doc);
    }

    #[test]
    fn writing_twice_is_byte_identical() {
        let doc = golden_doc();
        let a = TempFile::new("a.sltwx");
        let b = TempFile::new("b.sltwx");
        write_sltwx(&a.0, &doc).unwrap();
        write_sltwx(&b.0, &doc).unwrap();
        assert_eq!(fs::read(&a.0).unwrap(), fs::read(&b.0).unwrap());
    }

    #[test]
    fn entries_are_manifest_model_properties_in_order() {
        let f = TempFile::new("order.sltwx");
        write_sltwx(&f.0, &golden_doc()).unwrap();
        let bytes = fs::read(&f.0).unwrap();
        let names: Vec<String> = zip::list(&bytes)
            .unwrap()
            .iter()
            .map(|e| e.name().to_string())
            .collect();
        assert_eq!(names, ["manifest.json", "model.json", "properties.json"]);
    }

    #[test]
    fn stored_entries_and_extra_entries_are_read() {
        let mut junk = stored("cache/odd.bin", b"x");
        junk.method = 99; // ignored entries are never decoded
        let bytes = raw_zip(&[
            stored("manifest.json", &fixture_bytes("manifest.json")),
            stored("cache/body-b-1.brep", b"DBRep_DrawableShape"),
            stored("model.json", &fixture_bytes("model.json")),
            junk,
            stored("properties.json", &fixture_bytes("properties.json")),
        ]);
        assert_eq!(read_bytes(&bytes).unwrap(), golden_doc());
    }

    #[test]
    fn not_a_zip_is_malformed_archive() {
        assert!(matches!(
            read_bytes(b"not a zip"),
            Err(FormatError::MalformedArchive(_))
        ));
        assert!(matches!(
            read_bytes(b""),
            Err(FormatError::MalformedArchive(_))
        ));
    }

    #[test]
    fn missing_end_record_is_malformed_archive() {
        let f = TempFile::new("cut.sltwx");
        write_sltwx(&f.0, &golden_doc()).unwrap();
        let bytes = fs::read(&f.0).unwrap();
        let err = read_bytes(&bytes[..bytes.len() - 10]).unwrap_err();
        assert_eq!(
            err,
            FormatError::MalformedArchive("no end-of-central-directory record".into())
        );
    }

    #[test]
    fn truncated_entry_is_malformed_archive() {
        let mut model = stored("model.json", &fixture_bytes("model.json"));
        model.csize = Some(1_000_000);
        let bytes = raw_zip(&[
            stored("manifest.json", &fixture_bytes("manifest.json")),
            model,
        ]);
        let err = read_bytes(&bytes).unwrap_err();
        assert!(
            matches!(&err, FormatError::MalformedArchive(m) if m.contains("model.json") && m.contains("truncated")),
            "{err:?}"
        );
    }

    #[test]
    fn unsupported_method_names_the_method() {
        let mut manifest = stored("manifest.json", &fixture_bytes("manifest.json"));
        manifest.method = 99;
        let err = read_bytes(&raw_zip(&[manifest])).unwrap_err();
        assert_eq!(
            err,
            FormatError::UnsupportedMethod {
                entry: "manifest.json".into(),
                method: 99
            }
        );
        assert!(err.to_string().contains("99"));
    }

    #[test]
    fn crc_mismatch_names_the_entry() {
        let mut manifest = stored("manifest.json", &fixture_bytes("manifest.json"));
        manifest.crc ^= 1;
        let err = read_bytes(&raw_zip(&[manifest])).unwrap_err();
        assert_eq!(err, FormatError::CrcMismatch("manifest.json".into()));
        assert!(err.to_string().contains("manifest.json"));
    }

    #[test]
    fn flipped_byte_in_compressed_model_is_an_error_not_a_panic() {
        let f = TempFile::new("flip.sltwx");
        write_sltwx(&f.0, &golden_doc()).unwrap();
        let mut bytes = fs::read(&f.0).unwrap();
        // The first occurrence of the name is the local header; the entry's
        // compressed data follows it directly.
        let at = bytes.windows(10).position(|w| w == b"model.json").unwrap() + 10;
        bytes[at + 3] ^= 0xff;
        let err = read_bytes(&bytes).unwrap_err();
        assert!(
            matches!(
                err,
                FormatError::CrcMismatch(_) | FormatError::MalformedArchive(_)
            ),
            "{err:?}"
        );
    }

    #[test]
    fn missing_required_files() {
        let manifest = stored("manifest.json", &fixture_bytes("manifest.json"));
        let model = stored("model.json", &fixture_bytes("model.json"));
        assert_eq!(
            read_bytes(&raw_zip(&[model])).unwrap_err(),
            FormatError::MissingFile("manifest.json".into())
        );
        assert_eq!(
            read_bytes(&raw_zip(&[manifest])).unwrap_err(),
            FormatError::MissingFile("model.json".into())
        );
    }

    #[test]
    fn non_object_json_files_are_invalid_fields() {
        let manifest = || stored("manifest.json", &fixture_bytes("manifest.json"));
        let err = read_bytes(&raw_zip(&[manifest(), stored("model.json", b"[]")])).unwrap_err();
        assert!(matches!(&err, FormatError::InvalidField { field, .. } if field == "model.json"));
        let err = read_bytes(&raw_zip(&[
            manifest(),
            stored("model.json", b"{}"),
            stored("properties.json", b"3"),
        ]))
        .unwrap_err();
        assert!(
            matches!(&err, FormatError::InvalidField { field, .. } if field == "properties.json")
        );
    }

    #[test]
    fn extension_must_match_the_manifest_kind() {
        let doc = golden_doc();
        let wrong = TempFile::new("bracket.sltwxa");
        assert_eq!(
            write_sltwx(&wrong.0, &doc).unwrap_err(),
            FormatError::ExtensionMismatch {
                extension: Some("sltwxa".into()),
                kind: DocumentKind::Part
            }
        );
        assert!(!wrong.0.exists(), "nothing is written on a mismatch");
        let good = TempFile::new("bracket.sltwx");
        write_sltwx(&good.0, &doc).unwrap();
        fs::copy(&good.0, &wrong.0).unwrap();
        assert!(matches!(
            read_sltwx(&wrong.0),
            Err(FormatError::ExtensionMismatch { .. })
        ));
    }

    #[test]
    fn writing_rejects_non_object_model() {
        let mut doc = golden_doc();
        doc.model = serde_json::json!([1]);
        let f = TempFile::new("bad.sltwx");
        let err = write_sltwx(&f.0, &doc).unwrap_err();
        assert!(matches!(&err, FormatError::InvalidField { field, .. } if field == "model.json"));
        assert!(!f.0.exists());
    }

    #[test]
    fn missing_file_is_io_error() {
        let f = TempFile::new("absent.sltwx");
        assert!(matches!(read_sltwx(&f.0), Err(FormatError::Io { .. })));
    }

    #[test]
    fn system_unzip_accepts_our_archive() {
        let f = TempFile::new("unzip.sltwx");
        write_sltwx(&f.0, &golden_doc()).unwrap();
        match Command::new("unzip").arg("-t").arg(&f.0).output() {
            Ok(out) => assert!(
                out.status.success(),
                "unzip -t failed: {}",
                String::from_utf8_lossy(&out.stdout)
            ),
            Err(e) => println!("skipped: `unzip` is not available ({e})"),
        }
    }
}
