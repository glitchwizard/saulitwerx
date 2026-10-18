//! `swx-qa` command-line entry point. See [`swx_qa::run`].

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match swx_qa::run(&args) {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("swx-qa: {e}");
            ExitCode::FAILURE
        }
    }
}
