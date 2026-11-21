//! Integration tests for the `sltwx` pack / unpack command-line tool.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_sltwx");

fn golden() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/bracket.sltwx")
}

/// A fresh, empty temp folder unique to this process and call; removed on
/// drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(tag: &str) -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("swx-io-sltwx-cli-{}-{tag}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run(args: &[&Path]) -> Output {
    Command::new(BIN).args(args).output().unwrap()
}

fn run_str(args: &[&str]) -> Output {
    Command::new(BIN).args(args).output().unwrap()
}

fn code(out: &Output) -> i32 {
    out.status.code().expect("exited normally")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn pack_then_unpack_reproduces_the_golden_json_files() {
    let tmp = TempDir::new("roundtrip");
    let file = tmp.path("bracket.sltwx");
    let folder = tmp.path("out.sltwx");

    let out = run(&[Path::new("pack"), &golden(), &file]);
    assert_eq!(code(&out), 0, "pack: {}", stderr(&out));
    let out = run(&[Path::new("unpack"), &file, &folder]);
    assert_eq!(code(&out), 0, "unpack: {}", stderr(&out));

    let mut names: Vec<String> = fs::read_dir(golden())
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".json"))
        .collect();
    names.sort();
    assert_eq!(names, ["manifest.json", "model.json", "properties.json"]);
    for name in &names {
        assert_eq!(
            fs::read(folder.join(name)).unwrap(),
            fs::read(golden().join(name)).unwrap(),
            "{name} differs"
        );
    }
}

#[test]
fn packing_twice_gives_identical_files() {
    let tmp = TempDir::new("twice");
    let a = tmp.path("a.sltwx");
    let b = tmp.path("b.sltwx");
    assert_eq!(code(&run(&[Path::new("pack"), &golden(), &a])), 0);
    assert_eq!(code(&run(&[Path::new("pack"), &golden(), &b])), 0);
    assert_eq!(fs::read(&a).unwrap(), fs::read(&b).unwrap());
}

#[test]
fn an_existing_output_is_never_overwritten() {
    let tmp = TempDir::new("exists");
    let file = tmp.path("bracket.sltwx");
    fs::write(&file, b"keep me").unwrap();
    let out = run(&[Path::new("pack"), &golden(), &file]);
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).starts_with("sltwx: "), "{}", stderr(&out));
    assert!(stderr(&out).contains("already exists"), "{}", stderr(&out));
    assert_eq!(fs::read(&file).unwrap(), b"keep me");

    let folder = tmp.path("folder.sltwx");
    fs::create_dir(&folder).unwrap();
    let packed = tmp.path("packed.sltwx");
    assert_eq!(code(&run(&[Path::new("pack"), &golden(), &packed])), 0);
    let out = run(&[Path::new("unpack"), &packed, &folder]);
    assert_eq!(code(&out), 1);
    assert_eq!(fs::read_dir(&folder).unwrap().count(), 0);
}

#[test]
fn bad_arguments_exit_2_with_the_usage() {
    for args in [
        &[][..],
        &["pack", "onlyone"][..],
        &["frob", "a", "b"][..],
        &["pack", "a", "b", "c"][..],
    ] {
        let out = run_str(args);
        assert_eq!(code(&out), 2, "args {args:?}");
        assert!(stderr(&out).contains("usage: sltwx"), "args {args:?}");
    }
}

#[test]
fn help_prints_the_usage_and_exits_0() {
    for flag in ["--help", "-h"] {
        let out = run_str(&[flag]);
        assert_eq!(code(&out), 0);
        assert!(String::from_utf8_lossy(&out.stdout).contains("usage: sltwx"));
    }
}

#[test]
fn unpacking_a_non_zip_reports_the_malformed_archive_error() {
    let tmp = TempDir::new("notzip");
    let file = tmp.path("bad.sltwx");
    fs::write(&file, b"not a zip").unwrap();
    let folder = tmp.path("out.sltwx");
    let out = run(&[Path::new("unpack"), &file, &folder]);
    assert_eq!(code(&out), 1);
    let err = stderr(&out);
    assert!(err.starts_with("sltwx: "), "{err}");
    assert!(err.contains("malformed ZIP archive"), "{err}");
    assert_eq!(err.lines().count(), 1, "{err}");
    assert!(!folder.exists());
}

#[test]
fn an_extension_that_does_not_match_the_kind_exits_1() {
    let tmp = TempDir::new("ext");
    let file = tmp.path("out.sltwxa");
    let out = run(&[Path::new("pack"), &golden(), &file]);
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).starts_with("sltwx: "), "{}", stderr(&out));
    assert!(!file.exists());
}

#[test]
fn unpack_needs_an_existing_parent_folder() {
    let tmp = TempDir::new("parent");
    let file = tmp.path("bracket.sltwx");
    assert_eq!(code(&run(&[Path::new("pack"), &golden(), &file])), 0);
    let folder = tmp.path("missing/out.sltwx");
    let out = run(&[Path::new("unpack"), &file, &folder]);
    assert_eq!(code(&out), 1);
    assert!(!tmp.path("missing").exists());
}
