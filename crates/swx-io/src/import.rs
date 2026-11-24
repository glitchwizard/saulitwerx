//! STEP import as an embedded dumb solid feature (`docs/specs/M4-03.md`).
//!
//! [`embed_step`] copies a STEP file's bytes into a part as
//! `attachments/<feature-id>.step` and adds one `import_step` feature
//! ([`ImportStepFeature`]) and one body to `model.json`.
//! [`rebuild_imported`] gets the feature's shape back from the embedded
//! bytes. The embedded bytes are authoritative; `source.sha256` is an
//! integrity check of them.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{json, Map, Value};
use swx_kernel::{Kernel, KernelError};

use crate::unpacked::ATTACHMENTS;
use crate::{sha256, Document, DocumentKind, FormatError};

/// The `type` of an imported STEP feature in `model.json`.
pub const IMPORT_STEP: &str = "import_step";

/// What every STEP exchange structure starts with (ISO 10303-21, clause 5).
const STEP_HEADER: &[u8] = b"ISO-10303-21;";

/// One `import_step` feature entry of a part's `model.json`
/// (`docs/specs/M4-03.md` §1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportStepFeature {
    /// The feature's stable ID, e.g. `"ft-3b10"`.
    pub id: String,
    /// The name shown in the feature history, e.g. `"Imported1"`.
    pub name: String,
    /// The container path of the embedded bytes,
    /// `attachments/<feature-id>.step`.
    pub attachment: String,
    /// SHA-256 of the embedded bytes, as `sha256:<64 lowercase hex>`.
    pub sha256: String,
    /// The file name the user picked, without its directory. Display only.
    pub original_name: String,
    /// Whether the feature is suppressed.
    pub suppressed: bool,
}

impl ImportStepFeature {
    /// The feature as its `model.json` object. `op` is always `new_solid`.
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "type": IMPORT_STEP,
            "name": self.name,
            "source": {
                "attachment": self.attachment,
                "sha256": self.sha256,
                "original_name": self.original_name,
            },
            "op": "new_solid",
            "suppressed": self.suppressed,
        })
    }

    /// Reads an `import_step` feature object from `model.json`.
    ///
    /// # Errors
    ///
    /// [`FormatError::InvalidField`] when `v` is not an `import_step`
    /// feature or `op` is not `new_solid`; [`FormatError::MissingField`]
    /// when `id`, `name` or a `source` field is missing or not a string.
    /// A missing `suppressed` means `false`.
    pub fn from_json(v: &Value) -> Result<ImportStepFeature, FormatError> {
        if v.get("type").and_then(Value::as_str) != Some(IMPORT_STEP) {
            return Err(FormatError::InvalidField {
                field: "features[].type".into(),
                reason: format!("expected `{IMPORT_STEP}`"),
            });
        }
        if let Some(op) = v.get("op") {
            if op.as_str() != Some("new_solid") {
                return Err(FormatError::InvalidField {
                    field: "features[].op".into(),
                    reason: "only `new_solid` is supported for `import_step`".into(),
                });
            }
        }
        let text = |path: &[&str]| -> Result<String, FormatError> {
            path.iter()
                .try_fold(v, |at, key| at.get(key))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| FormatError::MissingField(path.join(".")))
        };
        Ok(ImportStepFeature {
            id: text(&["id"])?,
            name: text(&["name"])?,
            attachment: text(&["source", "attachment"])?,
            sha256: text(&["source", "sha256"])?,
            original_name: text(&["source", "original_name"])?,
            suppressed: v
                .get("suppressed")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        })
    }
}

/// Why a STEP file could not be imported, or an imported feature could not
/// be rebuilt. The messages are those of `docs/specs/M4-03.md` §3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepImportError {
    /// The file is missing or unreadable.
    Unreadable {
        /// The file the user picked.
        path: PathBuf,
        /// The operating system's message.
        message: String,
    },
    /// The file does not start with `ISO-10303-21;`.
    NotStep(PathBuf),
    /// The kernel rejected the file at import.
    KernelRejected {
        /// The file the user picked.
        path: PathBuf,
        /// The kernel's message.
        message: String,
    },
    /// The kernel's shape has zero volume.
    NoSolid(PathBuf),
    /// The document is not a part, or its `model.json` can't take the
    /// feature (a `features` or `bodies` field that is not an array).
    Document(String),
    /// No `import_step` feature has the ID asked for.
    UnknownFeature(String),
    /// The feature's embedded file is missing at rebuild.
    AttachmentMissing {
        /// The feature's name.
        feature: String,
        /// Its `source.attachment`.
        attachment: String,
    },
    /// The embedded file's SHA-256 does not match `source.sha256`.
    AttachmentDamaged {
        /// The feature's name.
        feature: String,
        /// Its `source.attachment`.
        attachment: String,
    },
    /// The kernel rejected the embedded file at rebuild.
    RebuildFailed {
        /// The feature's name.
        feature: String,
        /// The kernel's message, or why the temporary file failed.
        message: String,
    },
}

impl fmt::Display for StepImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepImportError::Unreadable { path, message } => {
                write!(f, "cannot read STEP file `{}`: {message}", path.display())
            }
            StepImportError::NotStep(path) => write!(
                f,
                "`{}` is not a STEP file (ISO 10303-21 header missing)",
                path.display()
            ),
            StepImportError::KernelRejected { path, message } => {
                write!(f, "STEP import of `{}` failed: {message}", path.display())
            }
            StepImportError::NoSolid(path) => write!(f, "`{}` contains no solid", path.display()),
            StepImportError::Document(why) => write!(f, "cannot import into this document: {why}"),
            StepImportError::UnknownFeature(id) => {
                write!(f, "no `{IMPORT_STEP}` feature with id `{id}`")
            }
            StepImportError::AttachmentMissing {
                feature,
                attachment,
            } => write!(f, "{feature}: embedded STEP file `{attachment}` is missing"),
            StepImportError::AttachmentDamaged {
                feature,
                attachment,
            } => write!(
                f,
                "{feature}: embedded STEP file `{attachment}` is damaged (sha256 does not match)"
            ),
            StepImportError::RebuildFailed { feature, message } => {
                write!(f, "{feature}: STEP import failed: {message}")
            }
        }
    }
}

impl std::error::Error for StepImportError {}

/// Imports the STEP file at `path` into the part `doc` as an embedded dumb
/// solid (`docs/specs/M4-03.md` §1, §3) and returns the new feature.
///
/// The file must start with `ISO-10303-21;` after optional whitespace, and
/// `kernel` must import it as a shape with non-zero volume. Then its bytes
/// are stored unchanged as `attachments/<feature-id>.step`, and `model.json`
/// gets an `import_step` feature named `Imported<N>` (the lowest unused `N`)
/// and a body named after the file without its extension. The feature goes
/// right after the `end_of_part` feature, which then becomes the new
/// feature, or at the end when there is no `end_of_part`.
///
/// # Errors
///
/// Any [`StepImportError`] of spec §3's import table, or
/// [`StepImportError::Document`]. On every error `doc` is left unchanged.
pub fn embed_step<K: Kernel>(
    kernel: &K,
    doc: &mut Document,
    path: &Path,
) -> Result<ImportStepFeature, StepImportError> {
    embed_with(doc, path, |p| {
        let shape = kernel.import_step(p)?;
        kernel.volume(&shape)
    })
}

/// [`embed_step`] with the kernel's part as a function from the file to its
/// volume, so the checks around it can be tested without a kernel.
fn embed_with(
    doc: &mut Document,
    path: &Path,
    import_volume: impl FnOnce(&Path) -> Result<f64, KernelError>,
) -> Result<ImportStepFeature, StepImportError> {
    if doc.manifest.kind != DocumentKind::Part {
        return Err(StepImportError::Document(
            "only a part can import a STEP file".into(),
        ));
    }
    let model = doc
        .model
        .as_object()
        .ok_or_else(|| StepImportError::Document("`model.json` is not an object".into()))?;
    for key in ["features", "bodies"] {
        if model.get(key).is_some_and(|v| !v.is_array()) {
            return Err(StepImportError::Document(format!(
                "`{key}` in `model.json` is not an array"
            )));
        }
    }
    let bytes = fs::read(path).map_err(|e| StepImportError::Unreadable {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    if !has_step_header(&bytes) {
        return Err(StepImportError::NotStep(path.to_path_buf()));
    }
    let volume = import_volume(path).map_err(|e| StepImportError::KernelRejected {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    if volume.is_nan() || volume == 0.0 {
        return Err(StepImportError::NoSolid(path.to_path_buf()));
    }

    let hex = hex(&sha256(&bytes));
    let features = model.get("features").and_then(Value::as_array);
    let bodies = model.get("bodies").and_then(Value::as_array);
    let id = unique(&format!("ft-{}", &hex[..8]), |c| {
        has_id(features, c) || doc.attachments.contains_key(&format!("{c}.step"))
    });
    let body_id = unique(&format!("b-{}", &hex[..8]), |c| has_id(bodies, c));
    let name = (1..)
        .map(|n| format!("Imported{n}"))
        .find(|n| {
            !features
                .into_iter()
                .flatten()
                .any(|f| f.get("name").and_then(Value::as_str) == Some(n.as_str()))
        })
        .unwrap_or_default();
    let original_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let body_name = Path::new(&original_name)
        .file_stem()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let feature = ImportStepFeature {
        attachment: format!("{ATTACHMENTS}/{id}.step"),
        sha256: format!("sha256:{hex}"),
        id,
        name,
        original_name,
        suppressed: false,
    };

    // Every check is done; from here on `doc` changes.
    let model: &mut Map<String, Value> = doc.model.as_object_mut().expect("checked above");
    let end = model
        .get("end_of_part")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let features = model
        .entry("features")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .expect("checked above");
    let at = end
        .as_deref()
        .and_then(|e| {
            features
                .iter()
                .position(|f| f.get("id").and_then(Value::as_str) == Some(e))
        })
        .map_or(features.len(), |i| i + 1);
    features.insert(at, feature.to_json());
    if end.is_some() {
        model.insert("end_of_part".into(), json!(feature.id));
    }
    model
        .entry("bodies")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .expect("checked above")
        .push(json!({ "id": body_id, "created_by": feature.id, "name": body_name }));
    doc.attachments
        .insert(format!("{}.step", feature.id), bytes);
    Ok(feature)
}

/// Rebuilds the `import_step` feature `id` of `doc` from its embedded bytes
/// (`docs/specs/M4-03.md` §2): checks them against `source.sha256`, writes
/// them to a temporary file under [`std::env::temp_dir`] (removed
/// afterwards) and imports that with `kernel`.
///
/// # Errors
///
/// [`StepImportError::UnknownFeature`] when `doc` has no such feature (or
/// its entry is malformed), and otherwise the errors of spec §3's rebuild
/// table: [`StepImportError::AttachmentMissing`],
/// [`StepImportError::AttachmentDamaged`],
/// [`StepImportError::RebuildFailed`].
pub fn rebuild_imported<K: Kernel>(
    kernel: &K,
    doc: &Document,
    id: &str,
) -> Result<K::Shape, StepImportError> {
    rebuild_with(doc, id, |p| kernel.import_step(p))
}

/// [`rebuild_imported`] with the kernel's import as a function, so the
/// checks before it can be tested without a kernel.
fn rebuild_with<S>(
    doc: &Document,
    id: &str,
    import: impl FnOnce(&Path) -> Result<S, KernelError>,
) -> Result<S, StepImportError> {
    let feature = doc
        .model
        .get("features")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|f| f.get("id").and_then(Value::as_str) == Some(id))
        .and_then(|f| ImportStepFeature::from_json(f).ok())
        .ok_or_else(|| StepImportError::UnknownFeature(id.to_owned()))?;
    let missing = || StepImportError::AttachmentMissing {
        feature: feature.name.clone(),
        attachment: feature.attachment.clone(),
    };
    let key = feature
        .attachment
        .strip_prefix(ATTACHMENTS)
        .and_then(|rest| rest.strip_prefix('/'))
        .ok_or_else(missing)?;
    let bytes = doc.attachments.get(key).ok_or_else(missing)?;
    if format!("sha256:{}", hex(&sha256(bytes))) != feature.sha256 {
        return Err(StepImportError::AttachmentDamaged {
            feature: feature.name,
            attachment: feature.attachment,
        });
    }
    let failed = |message: String| StepImportError::RebuildFailed {
        feature: feature.name.clone(),
        message,
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let tmp = std::env::temp_dir().join(format!(
        "swx-import-{}-{}.step",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&tmp, bytes).map_err(|e| failed(format!("{}: {e}", tmp.display())))?;
    let result = import(&tmp);
    let _ = fs::remove_file(&tmp);
    result.map_err(|e| failed(e.to_string()))
}

/// Whether `bytes` start with `ISO-10303-21;` after optional ASCII
/// whitespace.
fn has_step_header(bytes: &[u8]) -> bool {
    let start = bytes
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    bytes[start..].starts_with(STEP_HEADER)
}

fn hex(digest: &[u8]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn has_id(array: Option<&Vec<Value>>, id: &str) -> bool {
    array
        .into_iter()
        .flatten()
        .any(|v| v.get("id").and_then(Value::as_str) == Some(id))
}

/// `base`, or `base-2`, `base-3`, … : the first that `taken` rejects not.
fn unique(base: &str, taken: impl Fn(&str) -> bool) -> String {
    std::iter::once(base.to_owned())
        .chain((2..).map(|n| format!("{base}-{n}")))
        .find(|c| !taken(c))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{read_unpacked, write_unpacked};
    use std::sync::atomic::AtomicUsize;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> TempDir {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("swx-io-import-{}-{n}-{tag}", std::process::id()));
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

    fn golden() -> Document {
        read_unpacked(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/bracket.sltwx"))
            .unwrap()
    }

    const STEP: &[u8] = b"\n  ISO-10303-21;\nHEADER;\nENDSEC;\nDATA;\nENDSEC;\nEND-ISO-10303-21;\n";

    fn embedded(tmp: &TempDir) -> (Document, ImportStepFeature) {
        let src = tmp.0.join("bracket part.step");
        fs::write(&src, STEP).unwrap();
        let mut doc = golden();
        let f = embed_with(&mut doc, &src, |p| {
            assert_eq!(p, src);
            Ok(10_000.0)
        })
        .unwrap();
        (doc, f)
    }

    #[test]
    fn embed_adds_attachment_feature_and_body() {
        let tmp = TempDir::new("embed");
        let (doc, f) = embedded(&tmp);
        let hex = hex(&sha256(STEP));
        assert_eq!(f.id, format!("ft-{}", &hex[..8]));
        assert_eq!(f.name, "Imported1");
        assert_eq!(f.attachment, format!("attachments/{}.step", f.id));
        assert_eq!(f.sha256, format!("sha256:{hex}"));
        assert_eq!(f.original_name, "bracket part.step");
        assert_eq!(doc.attachments[&format!("{}.step", f.id)], STEP);
        let features = doc.model["features"].as_array().unwrap();
        assert_eq!(features.len(), 2);
        assert_eq!(features[1], f.to_json());
        assert_eq!(doc.model["end_of_part"], json!(f.id));
        let body = doc.model["bodies"]
            .as_array()
            .unwrap()
            .last()
            .unwrap()
            .clone();
        assert_eq!(body["created_by"], json!(f.id));
        assert_eq!(body["name"], "bracket part");
        // A second import of the same file gets fresh IDs and name.
        let mut doc2 = doc.clone();
        let src = tmp.0.join("bracket part.step");
        let g = embed_with(&mut doc2, &src, |_| Ok(1.0)).unwrap();
        assert_eq!(g.id, format!("{}-2", f.id));
        assert_eq!(g.name, "Imported2");
        assert_eq!(doc2.attachments.len(), 2);
    }

    /// Spec §6 test 2.
    #[test]
    fn import_step_feature_json_round_trips_and_rewrites_identically() {
        let tmp = TempDir::new("json");
        let (doc, f) = embedded(&tmp);
        let out = tmp.0.join("part.sltwx");
        write_unpacked(&out, &doc).unwrap();
        let first = fs::read(out.join("model.json")).unwrap();
        let back = read_unpacked(&out).unwrap();
        let entry = back.model["features"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["id"] == json!(f.id))
            .unwrap();
        assert_eq!(ImportStepFeature::from_json(entry).unwrap(), f);
        assert_eq!(entry["source"]["attachment"], json!(f.attachment));
        assert_eq!(entry["source"]["sha256"], json!(f.sha256));
        assert_eq!(entry["source"]["original_name"], "bracket part.step");
        assert_eq!(back, doc);
        write_unpacked(&out, &back).unwrap();
        assert_eq!(fs::read(out.join("model.json")).unwrap(), first);
    }

    #[test]
    fn from_json_rejects_other_features_and_missing_fields() {
        let golden = golden();
        let extrude = &golden.model["features"][0];
        assert!(ImportStepFeature::from_json(extrude).is_err());
        let mut v = ImportStepFeature {
            id: "ft-1".into(),
            name: "Imported1".into(),
            attachment: "attachments/ft-1.step".into(),
            sha256: "sha256:00".into(),
            original_name: "a.step".into(),
            suppressed: true,
        }
        .to_json();
        assert!(ImportStepFeature::from_json(&v).unwrap().suppressed);
        v["source"].as_object_mut().unwrap().remove("sha256");
        assert_eq!(
            ImportStepFeature::from_json(&v).unwrap_err(),
            FormatError::MissingField("source.sha256".into())
        );
    }

    fn no_kernel<T>(_: &Path) -> Result<T, KernelError> {
        panic!("the kernel must not be reached")
    }

    /// Spec §6 test 3, and the import-time messages of §3 that need no kernel.
    #[test]
    fn errors_without_a_kernel_have_the_spec_messages() {
        let tmp = TempDir::new("errors");
        let (doc, f) = embedded(&tmp);
        let key = format!("{}.step", f.id);

        let mut gone = doc.clone();
        gone.attachments.remove(&key);
        assert_eq!(
            rebuild_with(&gone, &f.id, no_kernel::<()>)
                .unwrap_err()
                .to_string(),
            format!("Imported1: embedded STEP file `attachments/{key}` is missing")
        );

        let mut flipped = doc.clone();
        flipped.attachments.get_mut(&key).unwrap()[5] ^= 1;
        assert_eq!(
            rebuild_with(&flipped, &f.id, no_kernel::<()>)
                .unwrap_err()
                .to_string(),
            format!(
                "Imported1: embedded STEP file `attachments/{key}` is damaged (sha256 does not match)"
            )
        );

        let hello = tmp.0.join("hello.txt");
        fs::write(&hello, "hello").unwrap();
        let mut d = golden();
        assert_eq!(
            embed_with(&mut d, &hello, no_kernel)
                .unwrap_err()
                .to_string(),
            format!(
                "`{}` is not a STEP file (ISO 10303-21 header missing)",
                hello.display()
            )
        );
        let missing = tmp.0.join("nope.step");
        let err = embed_with(&mut d, &missing, no_kernel)
            .unwrap_err()
            .to_string();
        assert!(
            err.starts_with(&format!("cannot read STEP file `{}`: ", missing.display())),
            "{err}"
        );
        let step = tmp.0.join("flat.step");
        fs::write(&step, STEP).unwrap();
        assert_eq!(
            embed_with(&mut d, &step, |_| Ok(0.0))
                .unwrap_err()
                .to_string(),
            format!("`{}` contains no solid", step.display())
        );
        assert_eq!(
            embed_with(&mut d, &step, |_| Err(KernelError::Backend("bad".into())))
                .unwrap_err()
                .to_string(),
            format!(
                "STEP import of `{}` failed: kernel failure: bad",
                step.display()
            )
        );
        assert_eq!(d, golden(), "a failed import adds nothing");
    }

    #[test]
    fn rebuild_passes_the_embedded_bytes_and_removes_the_temp_file() {
        let tmp = TempDir::new("rebuild");
        let (doc, f) = embedded(&tmp);
        let mut seen = None;
        let bytes = rebuild_with(&doc, &f.id, |p| {
            seen = Some(p.to_path_buf());
            Ok(fs::read(p).unwrap())
        })
        .unwrap();
        assert_eq!(bytes, STEP);
        assert!(!seen.unwrap().exists());
        let err = rebuild_with(&doc, &f.id, |_| -> Result<(), _> {
            Err(KernelError::Backend("no".into()))
        })
        .unwrap_err();
        assert_eq!(
            err.to_string(),
            "Imported1: STEP import failed: kernel failure: no"
        );
        assert_eq!(
            rebuild_with(&doc, "ft-91c2", no_kernel::<()>).unwrap_err(),
            StepImportError::UnknownFeature("ft-91c2".into())
        );
    }

    #[test]
    fn header_may_follow_whitespace_only() {
        assert!(has_step_header(b"ISO-10303-21;"));
        assert!(has_step_header(b" \r\n\tISO-10303-21;\n"));
        assert!(!has_step_header(b"x ISO-10303-21;"));
        assert!(!has_step_header(b""));
    }

    /// Spec §6 tests 5–7, with the OCCT kernel.
    #[cfg(feature = "occt")]
    mod occt {
        use super::*;
        use crate::{read_sltwx, write_sltwx};
        use std::collections::BTreeMap;
        use swx_kernel::OcctKernel;

        fn new_part() -> Document {
            Document {
                manifest: golden().manifest,
                model: json!({}),
                properties: None,
                attachments: BTreeMap::new(),
            }
        }

        fn close(a: f64, b: f64) -> bool {
            (a - b).abs() <= 1e-6 * b.abs().max(1.0)
        }

        /// Exports a 40 × 25 × 10 mm box to `dir/box.step`.
        fn exported_box(dir: &Path) -> PathBuf {
            let k = OcctKernel;
            let shape = k.make_box([0.0; 3], [40.0, 25.0, 10.0]).unwrap();
            let path = dir.join("box.step");
            k.export_step(&shape, &path).unwrap();
            path
        }

        fn check_box(doc: &Document, id: &str, size: [f64; 3]) {
            let k = OcctKernel;
            let shape = rebuild_imported(&k, doc, id).unwrap();
            let v = k.volume(&shape).unwrap();
            assert!(close(v, size[0] * size[1] * size[2]), "volume {v}");
            let bb = k.bounding_box(&shape).unwrap();
            for ((lo, hi), want) in bb.min.iter().zip(bb.max).zip(size) {
                assert!(close(*lo, 0.0), "min {:?}", bb.min);
                assert!(close(hi, want), "max {:?}", bb.max);
            }
        }

        /// Test 5: round trip through both container forms.
        #[test]
        fn box_round_trips_through_both_forms() {
            let tmp = TempDir::new("occt-rt");
            let src = exported_box(&tmp.0);
            let exported = fs::read(&src).unwrap();
            let mut doc = new_part();
            let f = embed_step(&OcctKernel, &mut doc, &src).unwrap();
            assert_eq!(doc.attachments[&format!("{}.step", f.id)], exported);
            assert_eq!(f.sha256, format!("sha256:{}", hex(&sha256(&exported))));

            let packed = tmp.0.join("part.sltwx");
            write_sltwx(&packed, &doc).unwrap();
            let back = read_sltwx(&packed).unwrap();
            assert_eq!(back, doc);
            check_box(&back, &f.id, [40.0, 25.0, 10.0]);

            let folder = tmp.0.join("folder").join("part.sltwx");
            write_unpacked(&folder, &doc).unwrap();
            let back = read_unpacked(&folder).unwrap();
            assert_eq!(back, doc);
            assert_eq!(fs::read(folder.join(&f.attachment)).unwrap(), exported);
            check_box(&back, &f.id, [40.0, 25.0, 10.0]);
        }

        /// Test 6: a file in metres imports at its true size in millimetres.
        #[test]
        fn metre_file_is_converted_to_millimetres() {
            let tmp = TempDir::new("occt-units");
            let src = exported_box(&tmp.0);
            let text = fs::read_to_string(&src).unwrap();
            assert!(text.contains("SI_UNIT(.MILLI.,.METRE.)"));
            let metres = tmp.0.join("box-m.step");
            fs::write(
                &metres,
                text.replace("SI_UNIT(.MILLI.,.METRE.)", "SI_UNIT($,.METRE.)"),
            )
            .unwrap();
            let mut doc = new_part();
            let f = embed_step(&OcctKernel, &mut doc, &metres).unwrap();
            check_box(&doc, &f.id, [40_000.0, 25_000.0, 10_000.0]);
        }

        /// Test 7: a valid header with a truncated body fails at import.
        #[test]
        fn truncated_file_fails_and_adds_nothing() {
            let tmp = TempDir::new("occt-trunc");
            let src = exported_box(&tmp.0);
            let text = fs::read_to_string(&src).unwrap();
            let cut = tmp.0.join("cut.step");
            fs::write(&cut, &text[..text.find("DATA;").unwrap() + 40]).unwrap();
            let mut doc = new_part();
            let err = embed_step(&OcctKernel, &mut doc, &cut).unwrap_err();
            assert!(
                err.to_string()
                    .starts_with(&format!("STEP import of `{}` failed: ", cut.display())),
                "{err}"
            );
            assert_eq!(doc, new_part());
        }
    }
}
