//! `swx-qa thumbnail` run as a process, so exit status and stderr are real.

use std::path::Path;
use std::process::{Command, Output};

#[cfg(feature = "occt")]
use std::path::PathBuf;
#[cfg(feature = "occt")]
use std::sync::atomic::{AtomicUsize, Ordering};

/// A fresh, uniquely named directory under the system temp dir, removed
/// when dropped.
#[cfg(feature = "occt")]
struct TempDir(PathBuf);

#[cfg(feature = "occt")]
impl TempDir {
    fn new(tag: &str) -> TempDir {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("swx-qa-thumbnail-{}-{n}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        TempDir(path)
    }
}

#[cfg(feature = "occt")]
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn thumbnail(doc: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_swx-qa"))
        .arg("thumbnail")
        .arg(doc)
        .output()
        .unwrap()
}

#[cfg(not(feature = "occt"))]
#[test]
fn without_occt_says_to_rebuild() {
    let out = thumbnail(Path::new("part.sltwx"));
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("--features occt"), "{err}");
}

#[cfg(feature = "occt")]
mod occt {
    use super::*;
    use swx_io::{
        build_part_cache, read_sltwx, read_unpacked, write_sltwx, write_unpacked, Document,
        DISPLAY_DEFLECTION_MM,
    };
    use swx_kernel::{Kernel, OcctKernel};
    use swx_qa::render::BACKGROUND;

    fn golden_part() -> Document {
        let dir =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../swx-io/tests/golden/bracket.sltwx");
        read_unpacked(&dir).unwrap()
    }

    /// The golden part with a 40 × 25 × 10 mm box cached as body `b-1`.
    fn cached_box_doc() -> Document {
        let k = OcctKernel;
        let shape = k.make_box([0.0; 3], [40.0, 25.0, 10.0]).unwrap();
        let mut doc = golden_part();
        build_part_cache(
            &k,
            &mut doc,
            &[("b-1".to_owned(), &shape)],
            DISPLAY_DEFLECTION_MM,
        )
        .unwrap();
        doc
    }

    /// Writes `doc` in both forms under `tmp`: (unpacked folder, `.sltwx` file).
    fn write_both(tmp: &TempDir, doc: &Document) -> [PathBuf; 2] {
        let dir = tmp.0.join("box.sltwx");
        write_unpacked(&dir, doc).unwrap();
        let file = tmp.0.join("packed").join("box.sltwx");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        write_sltwx(&file, doc).unwrap();
        [dir, file]
    }

    fn read(path: &Path) -> Document {
        if path.is_dir() {
            read_unpacked(path).unwrap()
        } else {
            read_sltwx(path).unwrap()
        }
    }

    /// Every file under `path` (or `path` itself), by path, with its bytes.
    fn snapshot(path: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut files = Vec::new();
        if path.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(path)
                .unwrap()
                .map(|e| e.unwrap().path())
                .collect();
            entries.sort();
            for e in entries {
                files.extend(snapshot(&e));
            }
        } else {
            files.push((path.to_owned(), std::fs::read(path).unwrap()));
        }
        files
    }

    /// (width, height, RGB pixels) of a PNG.
    fn decode(png: &[u8]) -> (u32, u32, Vec<[u8; 3]>) {
        let mut reader = png::Decoder::new(png).read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        assert_eq!(info.color_type, png::ColorType::Rgb);
        let px = buf[..info.buffer_size()]
            .chunks_exact(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();
        (info.width, info.height, px)
    }

    #[test]
    fn writes_fresh_thumbnail_in_both_forms() {
        let tmp = TempDir::new("both");
        for path in write_both(&tmp, &cached_box_doc()) {
            let out = thumbnail(&path);
            assert!(out.status.success(), "{out:?}");
            let doc = read(&path);
            let png = doc
                .fresh_cache_file("thumbnail.png")
                .expect("fresh thumbnail");
            let (w, h, px) = decode(png);
            assert_eq!((w, h), (256, 256));
            let drawn = px.iter().filter(|&&p| p != BACKGROUND).count();
            assert!(drawn * 10 >= px.len(), "{drawn} of {} drawn", px.len());
            assert!(doc.fresh_cache_file("cache/body-b-1.brep").is_some());
        }
    }

    #[test]
    fn running_twice_gives_identical_bytes() {
        let tmp = TempDir::new("twice");
        for path in write_both(&tmp, &cached_box_doc()) {
            assert!(thumbnail(&path).status.success());
            let first = snapshot(&path);
            let png = read(&path).cache.unwrap().entries["thumbnail.png"].clone();
            assert!(thumbnail(&path).status.success());
            assert_eq!(snapshot(&path), first, "{}", path.display());
            assert_eq!(
                read(&path).cache.unwrap().entries["thumbnail.png"],
                png,
                "{}",
                path.display()
            );
        }
    }

    #[test]
    fn no_fresh_body_brep_exits_1_and_writes_nothing() {
        let mut stale = cached_box_doc();
        stale.model["parameters"][0]["expr"] = "41 mm".into();
        for (tag, doc) in [("uncached", golden_part()), ("stale", stale)] {
            let tmp = TempDir::new(tag);
            for path in write_both(&tmp, &doc) {
                let before = snapshot(&path);
                let out = thumbnail(&path);
                assert_eq!(out.status.code(), Some(1), "{tag}");
                let err = String::from_utf8_lossy(&out.stderr);
                assert!(
                    err.contains("the cache must be built first"),
                    "{tag}: {err}"
                );
                assert_eq!(snapshot(&path), before, "{tag}: {}", path.display());
            }
        }
    }
}
