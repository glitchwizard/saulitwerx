//! `swx-qa thumbnail`: renders a part's `thumbnail.png` cache entry from
//! its fresh body BReps (`docs/specs/M4-04.md` §3, "Thumbnail before M5").

use std::path::Path;

/// Container path of the thumbnail.
pub const THUMBNAIL: &str = "thumbnail.png";

/// Thumbnail width and height in pixels (FILE_FORMAT.md).
pub const THUMBNAIL_SIZE: u32 = 256;

/// Reads the part at `path` (an unpacked folder or a `.sltwx` file), loads
/// every body with a fresh `cache/body-<id>.brep` through
/// [`swx_io::load_body`], renders them together with [`crate::render`]'s
/// fixed isometric view at 256 × 256 px, stores the PNG as the
/// `thumbnail.png` cache entry with the document's current `model_hash`,
/// and writes the document back in the form it was read from.
///
/// # Errors
///
/// The document can't be read or written, no body BRep is fresh (the cache
/// must be built first; nothing is written then), or rendering fails.
#[cfg(feature = "occt")]
pub fn thumbnail(path: &Path) -> Result<String, String> {
    use swx_io::{load_body, read_sltwx, read_unpacked, write_sltwx, write_unpacked, CacheEntry};
    use swx_kernel::{Mesh, OcctKernel};

    let unpacked = path.is_dir();
    let read = if unpacked {
        read_unpacked(path)
    } else {
        read_sltwx(path)
    };
    let mut doc = read.map_err(|e| format!("cannot read {}: {e}", path.display()))?;

    let k = OcctKernel;
    let mut mesh = Mesh::default();
    let mut model_hash = None;
    let mut bodies = 0;
    let entries = doc.cache.as_ref().map(|c| &c.entries);
    for (entry_path, entry) in entries.into_iter().flatten() {
        let Some(id) = entry_path
            .strip_prefix("cache/body-")
            .and_then(|r| r.strip_suffix(".brep"))
        else {
            continue;
        };
        // `load_body` returns a shape only for a fresh entry, whose
        // `model_hash` is therefore the document's current one.
        let Some(shape) = load_body(&k, &doc, id) else {
            continue;
        };
        let body = crate::fit_mesh(&k, &shape)?;
        let offset = mesh.positions.len() as u32;
        mesh.positions.extend_from_slice(&body.positions);
        mesh.normals.extend_from_slice(&body.normals);
        mesh.indices.extend(body.indices.iter().map(|i| i + offset));
        model_hash = Some(entry.model_hash.clone());
        bodies += 1;
    }
    let model_hash = model_hash.ok_or_else(|| {
        format!(
            "{} has no fresh body BRep (cache/body-<id>.brep): the cache must be built first",
            path.display()
        )
    })?;

    let opts = crate::render::Options {
        width: THUMBNAIL_SIZE,
        height: THUMBNAIL_SIZE,
        ..crate::render::Options::default()
    };
    let (img, _) = crate::render::render(&mesh, &opts)?;
    let png = img.to_png()?;
    let bytes = png.len();
    doc.cache
        .get_or_insert_with(Default::default)
        .entries
        .insert(
            THUMBNAIL.to_owned(),
            CacheEntry {
                model_hash,
                bytes: png,
            },
        );

    let written = if unpacked {
        write_unpacked(path, &doc)
    } else {
        write_sltwx(path, &doc)
    };
    written.map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(format!(
        "wrote {THUMBNAIL} into {} ({THUMBNAIL_SIZE}x{THUMBNAIL_SIZE}, {bytes} bytes, {bodies} bodies)",
        path.display()
    ))
}

/// Without the `occt` feature, body BReps can't be read.
#[cfg(not(feature = "occt"))]
pub fn thumbnail(path: &Path) -> Result<String, String> {
    Err(format!(
        "cannot make a thumbnail of {}: reading body BReps needs OCCT; rebuild with `cargo build -p swx-qa --features occt`",
        path.display()
    ))
}
