//! `sltwx`: convert a Saulitwerx document between its unpacked folder form
//! and the single-file `.sltwx` ZIP (`docs/specs/FILE_FORMAT.md`).
//!
//! Usage:
//!
//! ```text
//! sltwx pack <folder> <file>     read the unpacked <folder>, write the ZIP <file>
//! sltwx unpack <file> <folder>   read the ZIP <file>, write the unpacked <folder>
//! sltwx --help | -h              print this usage
//! ```
//!
//! `unpack` creates `<folder>`; its parent must already exist. The output
//! path must not exist: it is never overwritten.
//!
//! Exit codes: 0 on success; 1 with a one-line `sltwx: <error>` on stderr
//! when reading or writing fails or the output path already exists; 2 with
//! the usage on stderr for a missing, extra or unknown argument or
//! subcommand.

use std::path::Path;
use std::process::ExitCode;

use swx_io::{read_sltwx, read_unpacked, write_sltwx, write_unpacked};

const USAGE: &str = "\
usage: sltwx pack <folder> <file>     read the unpacked <folder>, write the ZIP <file>
       sltwx unpack <file> <folder>   read the ZIP <file>, write the unpacked <folder>
       sltwx --help | -h              print this usage

The output path must not exist; it is never overwritten.";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    match argv.as_slice() {
        ["--help"] | ["-h"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        ["pack", folder, file] => report(pack(Path::new(folder), Path::new(file))),
        ["unpack", file, folder] => report(unpack(Path::new(file), Path::new(folder))),
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// Exit 0 on success, or print `sltwx: <error>` to stderr and exit 1.
fn report(result: Result<(), String>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("sltwx: {e}");
            ExitCode::from(1)
        }
    }
}

/// Fails when `out` already exists (as anything, including a dangling
/// symlink), so the tool never overwrites.
fn refuse_existing(out: &Path) -> Result<(), String> {
    if out.symlink_metadata().is_ok() {
        return Err(format!("{}: already exists", out.display()));
    }
    Ok(())
}

fn pack(folder: &Path, file: &Path) -> Result<(), String> {
    refuse_existing(file)?;
    let doc = read_unpacked(folder).map_err(|e| e.to_string())?;
    write_sltwx(file, &doc).map_err(|e| e.to_string())
}

fn unpack(file: &Path, folder: &Path) -> Result<(), String> {
    refuse_existing(folder)?;
    let parent = match folder.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    if !parent.is_dir() {
        return Err(format!(
            "{}: parent folder does not exist",
            folder.display()
        ));
    }
    let doc = read_sltwx(file).map_err(|e| e.to_string())?;
    write_unpacked(folder, &doc).map_err(|e| e.to_string())
}
