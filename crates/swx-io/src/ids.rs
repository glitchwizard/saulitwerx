//! Stable IDs in a part's `model.json` (`docs/specs/FILE_FORMAT.md`,
//! "`model.json` rules"): every sketch, entity, constraint, feature and body
//! has an `id` that never changes, and every array except `features` is a
//! set, written sorted by `id`.
//!
//! The "ID arrays" of a part are the top-level `sketches`, `features` and
//! `bodies`, and each sketch's `entities` and `constraints`. An absent array
//! is fine. Each element must be an object with a non-empty string `id`,
//! unique within its array. Assemblies and drawings are not checked yet.

use std::borrow::Cow;
use std::collections::HashSet;

use serde_json::Value;

use crate::{DocumentKind, FormatError};

/// Top-level ID arrays, and whether each is a set (sorted on write).
const TOP: [(&str, bool); 3] = [("sketches", true), ("features", false), ("bodies", true)];
/// ID arrays inside each sketch; both are sets.
const IN_SKETCH: [&str; 2] = ["entities", "constraints"];

/// Checks the IDs of a part's `model`. Other kinds, and a `model` that is not
/// an object, pass unchanged (the object check is done elsewhere).
pub(crate) fn check_ids(kind: DocumentKind, model: &Value) -> Result<(), FormatError> {
    if kind != DocumentKind::Part {
        return Ok(());
    }
    let Some(obj) = model.as_object() else {
        return Ok(());
    };
    for (name, _) in TOP {
        let Some(items) = id_array(obj.get(name), name)? else {
            continue;
        };
        check_array(items, name)?;
        if name == "sketches" {
            for (i, sketch) in items.iter().enumerate() {
                for inner in IN_SKETCH {
                    let path = format!("sketches[{i}].{inner}");
                    if let Some(list) = id_array(sketch.get(inner), &path)? {
                        check_array(list, &path)?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// The `model` to write: checked by [`check_ids`], with a part's set arrays
/// sorted by `id` in byte order. `features` keep their order. Returns the
/// input unchanged (borrowed) for other kinds.
pub(crate) fn model_for_write(
    kind: DocumentKind,
    model: &Value,
) -> Result<Cow<'_, Value>, FormatError> {
    check_ids(kind, model)?;
    if kind != DocumentKind::Part {
        return Ok(Cow::Borrowed(model));
    }
    let Some(obj) = model.as_object() else {
        return Ok(Cow::Borrowed(model));
    };
    let mut obj = obj.clone();
    for (name, is_set) in TOP {
        let Some(Value::Array(items)) = obj.get_mut(name) else {
            continue;
        };
        if is_set {
            sort_by_id(items);
        }
        if name == "sketches" {
            for sketch in items.iter_mut() {
                for inner in IN_SKETCH {
                    if let Some(Value::Array(list)) = sketch.get_mut(inner) {
                        sort_by_id(list);
                    }
                }
            }
        }
    }
    Ok(Cow::Owned(Value::Object(obj)))
}

/// `value` as an ID array: `None` when absent, an error when it is not an
/// array.
fn id_array<'a>(value: Option<&'a Value>, path: &str) -> Result<Option<&'a [Value]>, FormatError> {
    match value {
        None => Ok(None),
        Some(Value::Array(items)) => Ok(Some(items)),
        Some(_) => Err(FormatError::InvalidField {
            field: path.into(),
            reason: "must be an array".into(),
        }),
    }
}

/// Every element has a non-empty string `id`, and no `id` repeats.
fn check_array(items: &[Value], path: &str) -> Result<(), FormatError> {
    let mut seen = HashSet::new();
    for (index, item) in items.iter().enumerate() {
        let id = item
            .get("id")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| FormatError::InvalidId {
                array: path.into(),
                index,
            })?;
        if !seen.insert(id) {
            return Err(FormatError::DuplicateId {
                array: path.into(),
                id: id.into(),
            });
        }
    }
    Ok(())
}

/// Sorts checked elements by their `id` string in byte order (`str`'s
/// `Ord`), the same order the canonical writer uses for keys.
fn sort_by_id(items: &mut [Value]) {
    items.sort_by(|a, b| id_of(a).cmp(id_of(b)));
}

fn id_of(v: &Value) -> &str {
    v.get("id").and_then(Value::as_str).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zip;
    use crate::{read_sltwx, read_unpacked, write_sltwx, write_unpacked, Document};
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A fresh, uniquely named directory under the system temp dir, removed
    /// when dropped.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(tag: &str) -> TempDir {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let n = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("swx-io-ids-test-{}-{n}-{tag}", std::process::id()));
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

    fn golden() -> Document {
        read_unpacked(&fixture()).unwrap()
    }

    fn ids(v: &Value) -> Vec<&str> {
        v.as_array().unwrap().iter().map(id_of).collect()
    }

    /// The golden part with unsorted `bodies`, `entities` and `features`.
    fn unsorted() -> Document {
        let mut doc = golden();
        let m = &mut doc.model;
        m["bodies"] = json!([{"id": "b-2"}, {"id": "b-1"}]);
        m["sketches"][0]["entities"] = json!([{"id": "e2"}, {"id": "e10"}, {"id": "e1"}]);
        m["features"] = json!([{"id": "ft-b"}, {"id": "ft-a"}]);
        doc
    }

    /// Reads the model bytes stored in a written `.sltwx` file.
    fn zipped_model(path: &Path) -> Vec<u8> {
        let bytes = fs::read(path).unwrap();
        let entries = zip::list(&bytes).unwrap();
        let e = entries.iter().find(|e| e.name() == "model.json").unwrap();
        zip::extract(&bytes, e).unwrap()
    }

    #[test]
    fn golden_round_trips_byte_identically_in_both_forms() {
        let doc = golden();
        let golden_model = fs::read(fixture().join("model.json")).unwrap();
        let tmp = TempDir::new("golden");
        let dir = tmp.0.join("bracket.sltwx");
        write_unpacked(&dir, &doc).unwrap();
        assert_eq!(fs::read(dir.join("model.json")).unwrap(), golden_model);
        assert_eq!(read_unpacked(&dir).unwrap(), doc);
        let file = tmp.0.join("packed.sltwx");
        write_sltwx(&file, &doc).unwrap();
        assert_eq!(zipped_model(&file), golden_model);
        assert_eq!(read_sltwx(&file).unwrap(), doc);
    }

    #[test]
    fn sets_are_sorted_by_id_on_write_and_features_keep_their_order() {
        let doc = unsorted();
        let before = doc.clone();
        let tmp = TempDir::new("sorted");
        let dir = tmp.0.join("u.sltwx");
        let file = tmp.0.join("p.sltwx");
        write_unpacked(&dir, &doc).unwrap();
        write_sltwx(&file, &doc).unwrap();
        assert_eq!(doc, before, "the caller's document is not modified");
        for back in [read_unpacked(&dir).unwrap(), read_sltwx(&file).unwrap()] {
            assert_eq!(ids(&back.model["bodies"]), ["b-1", "b-2"]);
            assert_eq!(
                ids(&back.model["sketches"][0]["entities"]),
                ["e1", "e10", "e2"]
            );
            assert_eq!(ids(&back.model["sketches"][0]["constraints"]), ["c1", "c2"]);
            assert_eq!(ids(&back.model["features"]), ["ft-b", "ft-a"]);
        }
        assert_eq!(
            fs::read(dir.join("model.json")).unwrap(),
            zipped_model(&file)
        );
    }

    #[test]
    fn reading_does_not_reorder() {
        let doc = unsorted();
        let text = crate::to_canonical_json(&doc.model);
        let tmp = TempDir::new("noreorder");
        let dir = tmp.0.join("r.sltwx");
        write_unpacked(&dir, &golden()).unwrap();
        fs::write(dir.join("model.json"), &text).unwrap();
        let back = read_unpacked(&dir).unwrap();
        assert_eq!(ids(&back.model["bodies"]), ["b-2", "b-1"]);
        assert_eq!(
            ids(&back.model["sketches"][0]["entities"]),
            ["e2", "e10", "e1"]
        );
    }

    /// The three bad cases of the issue and the error each must give.
    fn bad_cases() -> Vec<(Document, FormatError)> {
        let mut dup = golden();
        dup.model["bodies"] = json!([{"id": "b-1"}, {"id": "b-1"}]);
        let mut no_id = golden();
        no_id.model["sketches"][0]["entities"] = json!([{"id": "e1"}, {"kind": "line"}]);
        let mut numeric = golden();
        numeric.model["sketches"][0]["constraints"] = json!([{"id": 5}]);
        vec![
            (
                dup,
                FormatError::DuplicateId {
                    array: "bodies".into(),
                    id: "b-1".into(),
                },
            ),
            (
                no_id,
                FormatError::InvalidId {
                    array: "sketches[0].entities".into(),
                    index: 1,
                },
            ),
            (
                numeric,
                FormatError::InvalidId {
                    array: "sketches[0].constraints".into(),
                    index: 0,
                },
            ),
        ]
    }

    #[test]
    fn bad_ids_are_rejected_by_both_writers_with_nothing_written() {
        let tmp = TempDir::new("badwrite");
        for (i, (doc, want)) in bad_cases().into_iter().enumerate() {
            let dir = tmp.0.join(format!("w{i}.sltwx"));
            let file = tmp.0.join(format!("f{i}.sltwx"));
            assert_eq!(write_unpacked(&dir, &doc), Err(want.clone()));
            assert_eq!(write_sltwx(&file, &doc), Err(want));
            assert!(!dir.exists() && !file.exists());
        }
    }

    #[test]
    fn bad_ids_are_rejected_by_both_readers() {
        let tmp = TempDir::new("badread");
        let manifest = fs::read(fixture().join("manifest.json")).unwrap();
        for (i, (doc, want)) in bad_cases().into_iter().enumerate() {
            let model = crate::to_canonical_json(&doc.model);
            let dir = tmp.0.join(format!("r{i}.sltwx"));
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("manifest.json"), &manifest).unwrap();
            fs::write(dir.join("model.json"), &model).unwrap();
            assert_eq!(read_unpacked(&dir), Err(want.clone()));
            let file = tmp.0.join(format!("z{i}.sltwx"));
            let bytes = zip::write_zip(&[
                ("manifest.json", &manifest),
                ("model.json", model.as_bytes()),
            ])
            .unwrap();
            fs::write(&file, bytes).unwrap();
            assert_eq!(read_sltwx(&file), Err(want));
        }
    }

    #[test]
    fn empty_id_and_non_object_elements_are_invalid() {
        let part = DocumentKind::Part;
        let empty = json!({"bodies": [{"id": ""}]});
        let scalar = json!({"sketches": [{"id": "s"}, 7]});
        let want = |array: &str, index| FormatError::InvalidId {
            array: array.into(),
            index,
        };
        assert_eq!(check_ids(part, &empty), Err(want("bodies", 0)));
        assert_eq!(check_ids(part, &scalar), Err(want("sketches", 1)));
        // Absent arrays are fine.
        assert_eq!(check_ids(part, &json!({})), Ok(()));
    }

    #[test]
    fn an_id_array_that_is_not_an_array_is_an_invalid_field() {
        let part = DocumentKind::Part;
        assert_eq!(
            check_ids(part, &json!({"bodies": {"id": "b-1"}})),
            Err(FormatError::InvalidField {
                field: "bodies".into(),
                reason: "must be an array".into(),
            })
        );
        assert_eq!(
            check_ids(part, &json!({"sketches": [{"id": "s", "constraints": 3}]})),
            Err(FormatError::InvalidField {
                field: "sketches[0].constraints".into(),
                reason: "must be an array".into(),
            })
        );
    }

    #[test]
    fn errors_display_the_array_and_the_id() {
        assert_eq!(
            FormatError::InvalidId {
                array: "sketches[0].constraints".into(),
                index: 3,
            }
            .to_string(),
            "`sketches[0].constraints[3]` needs a non-empty string `id`"
        );
        assert_eq!(
            FormatError::DuplicateId {
                array: "bodies".into(),
                id: "b-1".into(),
            }
            .to_string(),
            "duplicate id `b-1` in `bodies`"
        );
    }

    #[test]
    fn assemblies_and_drawings_are_not_checked_or_sorted() {
        let tmp = TempDir::new("asm");
        let model = json!({"bodies": [{"id": "b-2"}, {"id": "b-2"}, {"x": 1}]});
        for (kind, ext) in [
            (DocumentKind::Assembly, "sltwxa"),
            (DocumentKind::Drawing, "sltwxd"),
        ] {
            let mut doc = golden();
            doc.manifest.kind = kind;
            doc.model = model.clone();
            let dir = tmp.0.join(format!("u.{ext}"));
            let file = tmp.0.join(format!("p.{ext}"));
            write_unpacked(&dir, &doc).unwrap();
            write_sltwx(&file, &doc).unwrap();
            assert_eq!(read_unpacked(&dir).unwrap(), doc);
            assert_eq!(read_sltwx(&file).unwrap(), doc);
        }
    }
}
