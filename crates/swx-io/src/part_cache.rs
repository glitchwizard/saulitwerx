//! Building and loading a part's body cache (`docs/specs/M4-04.md` §3):
//! one `cache/body-<id>.brep` per body and one merged `cache/display.glb`.

use std::collections::BTreeMap;
use std::fmt;

use swx_kernel::{Kernel, KernelError};

use crate::cache::is_entry_path;
use crate::{write_glb, Cache, CacheEntry, Document, GlbError};

/// The display-mesh deflection callers pass to [`build_part_cache`] unless
/// they have a reason not to, in millimetres (spec §3, "Display deflection").
pub const DISPLAY_DEFLECTION_MM: f64 = 0.05;

/// Container path of the merged display mesh.
const DISPLAY_GLB: &str = "cache/display.glb";

/// Why [`build_part_cache`] failed. On any error the document is unchanged.
#[derive(Debug, Clone, PartialEq)]
pub enum PartCacheError {
    /// The deflection is not finite or not greater than 0. Holds the value.
    InvalidDeflection(f64),
    /// `cache/body-<id>.brep` is not a valid cache path for this body ID
    /// (empty, or containing `/` or `\`). Holds the ID.
    InvalidBodyId(String),
    /// The kernel could not write a BRep or tessellate a body.
    Kernel(KernelError),
    /// The merged display mesh could not be written as GLB.
    Mesh(GlbError),
}

impl fmt::Display for PartCacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartCacheError::InvalidDeflection(d) => write!(
                f,
                "deflection {d} mm is invalid: it must be finite and greater than 0"
            ),
            PartCacheError::InvalidBodyId(id) => {
                write!(f, "body ID {id:?} can't name a cache file")
            }
            PartCacheError::Kernel(e) => write!(f, "could not build the body cache: {e}"),
            PartCacheError::Mesh(e) => write!(f, "could not write the display mesh: {e}"),
        }
    }
}

impl std::error::Error for PartCacheError {}

fn body_path(body_id: &str) -> String {
    format!("cache/body-{body_id}.brep")
}

fn check_deflection(deflection_mm: f64) -> Result<f64, PartCacheError> {
    if deflection_mm.is_finite() && deflection_mm > 0.0 {
        Ok(deflection_mm)
    } else {
        Err(PartCacheError::InvalidDeflection(deflection_mm))
    }
}

/// Fills `doc.cache` with the derived files of a part's solved `bodies`,
/// given as `(body ID, shape)` pairs (spec §3):
///
/// - `cache/body-<id>.brep` for each body, from [`Kernel::write_brep`];
/// - `cache/display.glb`: every body tessellated at `deflection_mm` with
///   [`Kernel::tessellate`], merged into one mesh (each body's indices
///   offset by the vertices before it) and written with [`write_glb`].
///
/// Every entry built gets the document's current `model_hash` (the hash
/// [`Document::refresh_model_hash`] records), so it is fresh until the
/// model changes. Other entries already in `doc.cache` are kept; entries at
/// the same paths are replaced.
///
/// # Errors
///
/// `deflection_mm` and the body IDs are checked before any kernel call. On
/// any error `doc` is left unchanged.
pub fn build_part_cache<K: Kernel>(
    kernel: &K,
    doc: &mut Document,
    bodies: &[(String, &K::Shape)],
    deflection_mm: f64,
) -> Result<(), PartCacheError> {
    let deflection = check_deflection(deflection_mm)?;
    for (id, _) in bodies {
        if !is_entry_path(&body_path(id)) {
            return Err(PartCacheError::InvalidBodyId(id.clone()));
        }
    }

    let model_hash = doc.written_model_hash();
    let entry = |bytes: Vec<u8>| CacheEntry {
        model_hash: model_hash.clone(),
        bytes,
    };
    let mut built = BTreeMap::new();
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    for (id, shape) in bodies {
        let brep = kernel.write_brep(shape).map_err(PartCacheError::Kernel)?;
        built.insert(body_path(id), entry(brep.into_bytes()));
        let mesh = kernel
            .tessellate(shape, deflection)
            .map_err(PartCacheError::Kernel)?;
        let offset = positions.len() as u32;
        positions.extend_from_slice(&mesh.positions);
        indices.extend(mesh.indices.iter().map(|i| i + offset));
    }
    let mut glb = Vec::new();
    write_glb(&positions, &indices, &mut glb).map_err(PartCacheError::Mesh)?;
    built.insert(DISPLAY_GLB.to_owned(), entry(glb));

    doc.cache
        .get_or_insert_with(Cache::default)
        .entries
        .extend(built);
    Ok(())
}

/// The shape of body `body_id` from its cached `cache/body-<id>.brep`, if
/// that entry is fresh ([`Document::fresh_cache_file`]) and reads back with
/// [`Kernel::read_brep`]. `None` on any failure: a missing, stale or
/// unreadable entry means the caller rebuilds instead (spec §4).
pub fn load_body<K: Kernel>(kernel: &K, doc: &Document, body_id: &str) -> Option<K::Shape> {
    let bytes = doc.fresh_cache_file(&body_path(body_id))?;
    let text = std::str::from_utf8(bytes).ok()?;
    kernel.read_brep(text).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deflection_check() {
        assert_eq!(check_deflection(0.05), Ok(0.05));
        for bad in [0.0, -1.0, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                check_deflection(bad),
                Err(PartCacheError::InvalidDeflection(bad))
            );
        }
        assert!(matches!(
            check_deflection(f64::NAN),
            Err(PartCacheError::InvalidDeflection(d)) if d.is_nan()
        ));
        assert!(PartCacheError::InvalidDeflection(0.0)
            .to_string()
            .contains("greater than 0"));
    }

    #[test]
    fn body_paths() {
        assert_eq!(body_path("b-1"), "cache/body-b-1.brep");
        assert!(is_entry_path(&body_path("b-1")));
        assert!(!is_entry_path(&body_path("a/b")));
        assert!(!is_entry_path(&body_path("a\\b")));
    }

    #[cfg(feature = "occt")]
    mod occt {
        use super::super::*;
        use crate::{read_sltwx, read_unpacked, write_sltwx, write_unpacked};
        use std::path::{Path, PathBuf};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use swx_kernel::OcctKernel;

        /// A fresh, uniquely named directory under the system temp dir,
        /// removed when dropped.
        struct TempDir(PathBuf);

        impl TempDir {
            fn new(tag: &str) -> TempDir {
                static NEXT: AtomicUsize = AtomicUsize::new(0);
                let n = NEXT.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!(
                    "swx-io-part-cache-{}-{n}-{tag}",
                    std::process::id()
                ));
                let _ = std::fs::remove_dir_all(&path);
                std::fs::create_dir_all(&path).unwrap();
                TempDir(path)
            }
        }

        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }

        fn golden_part() -> Document {
            let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/bracket.sltwx");
            read_unpacked(&dir).unwrap()
        }

        fn box_shape(k: &OcctKernel) -> <OcctKernel as Kernel>::Shape {
            k.make_box([0.0; 3], [40.0, 25.0, 10.0]).unwrap()
        }

        fn cached_box_doc(k: &OcctKernel) -> Document {
            let shape = box_shape(k);
            let mut doc = golden_part();
            build_part_cache(
                k,
                &mut doc,
                &[("b-1".to_owned(), &shape)],
                DISPLAY_DEFLECTION_MM,
            )
            .unwrap();
            doc
        }

        /// The POSITION accessor's `min` and `max` from a GLB's JSON chunk.
        fn glb_bounds(glb: &[u8]) -> (Vec<f64>, Vec<f64>) {
            let len = u32::from_le_bytes(glb[12..16].try_into().unwrap()) as usize;
            let text = std::str::from_utf8(&glb[20..20 + len]).unwrap();
            let json: serde_json::Value = serde_json::from_str(text.trim_end()).unwrap();
            let pos = &json["accessors"][0];
            let read = |k: &str| {
                pos[k]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap())
                    .collect()
            };
            (read("min"), read("max"))
        }

        fn check_box_cache(k: &OcctKernel, doc: &Document) {
            let body = load_body(k, doc, "b-1").expect("fresh body");
            let v = k.volume(&body).unwrap();
            assert!((v - 10_000.0).abs() <= 1e-9 * 10_000.0, "volume {v}");
            let glb = doc
                .fresh_cache_file("cache/display.glb")
                .expect("fresh glb");
            let (min, max) = glb_bounds(glb);
            for (got, want) in min.iter().zip([0.0, 0.0, 0.0]) {
                assert!((got - want).abs() <= 1e-4, "min {min:?}");
            }
            for (got, want) in max.iter().zip([40.0, 25.0, 10.0]) {
                assert!((got - want).abs() <= 1e-4, "max {max:?}");
            }
        }

        #[test]
        fn box_cache_round_trips_in_both_forms() {
            let k = OcctKernel;
            let doc = cached_box_doc(&k);
            let entries = &doc.cache.as_ref().unwrap().entries;
            assert!(entries.contains_key("cache/body-b-1.brep"));
            assert!(entries.contains_key("cache/display.glb"));
            check_box_cache(&k, &doc);

            let tmp = TempDir::new("forms");
            let dir = tmp.0.join("box.sltwx");
            write_unpacked(&dir, &doc).unwrap();
            check_box_cache(&k, &read_unpacked(&dir).unwrap());

            let file = tmp.0.join("box-file.sltwx");
            write_sltwx(&file, &doc).unwrap();
            check_box_cache(&k, &read_sltwx(&file).unwrap());
        }

        #[test]
        fn garbage_brep_loads_as_none() {
            let k = OcctKernel;
            let mut doc = cached_box_doc(&k);
            let tmp = TempDir::new("garbage");
            let dir = tmp.0.join("box.sltwx");
            doc.cache
                .as_mut()
                .unwrap()
                .entries
                .get_mut("cache/body-b-1.brep")
                .unwrap()
                .bytes = b"garbage".to_vec();
            // Written and read back, so the index's sha256 matches the new bytes.
            write_unpacked(&dir, &doc).unwrap();
            let back = read_unpacked(&dir).unwrap();
            assert_eq!(
                back.fresh_cache_file("cache/body-b-1.brep"),
                Some(&b"garbage"[..])
            );
            assert!(load_body(&k, &back, "b-1").is_none());
        }

        #[test]
        fn model_edit_makes_body_stale() {
            let k = OcctKernel;
            let mut doc = cached_box_doc(&k);
            assert!(load_body(&k, &doc, "b-1").is_some());
            doc.model["parameters"][0]["expr"] = serde_json::Value::String("41 mm".into());
            assert!(load_body(&k, &doc, "b-1").is_none());
            assert!(load_body(&k, &doc, "b-2").is_none());
        }

        #[test]
        fn invalid_deflection_or_body_id_leaves_cache_unchanged() {
            let k = OcctKernel;
            let shape = box_shape(&k);
            let bodies = [("b-1".to_owned(), &shape)];
            for doc in [golden_part(), cached_box_doc(&k)] {
                for bad in [0.0, -0.05, f64::NAN, f64::INFINITY] {
                    let mut d = doc.clone();
                    let err = build_part_cache(&k, &mut d, &bodies, bad).unwrap_err();
                    assert!(matches!(err, PartCacheError::InvalidDeflection(_)));
                    assert_eq!(d.cache, doc.cache);
                }
                let mut d = doc.clone();
                let err = build_part_cache(
                    &k,
                    &mut d,
                    &[("a/b".to_owned(), &shape)],
                    DISPLAY_DEFLECTION_MM,
                )
                .unwrap_err();
                assert_eq!(err, PartCacheError::InvalidBodyId("a/b".into()));
                assert_eq!(d.cache, doc.cache);
            }
        }
    }
}
