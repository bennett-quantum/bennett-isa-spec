//! Harness CLI: run the conformance cases against any binary that
//! implements the reference CLI.
//!
//! Usage:
//!   bennett-isa-conformance [--bin PATH] [--cases DIR] [--suite X] [--list]
//!
//! Exit status: 0 when every case passes and none is unavailable;
//! 1 when any case fails; 2 when no case fails but at least one is
//! UNAVAILABLE (a staged deliverable: loud, never a silent skip).

use bennett_isa_conformance::{cases_dir, default_bin, load_cases, run_all};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut bin = default_bin();
    let mut cases_root = cases_dir();
    let mut suite: Option<String> = None;
    let mut list = false;

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--bin" => match args.next() {
                Some(p) => bin = PathBuf::from(p),
                None => return usage("--bin needs a PATH"),
            },
            "--cases" => match args.next() {
                Some(p) => cases_root = PathBuf::from(p),
                None => return usage("--cases needs a DIR"),
            },
            "--suite" => match args.next() {
                Some(s) => suite = Some(s),
                None => return usage("--suite needs a letter A-H"),
            },
            "--list" => list = true,
            "-h" | "--help" => {
                println!(
                    "usage: bennett-isa-conformance [--bin PATH] [--cases DIR] [--suite X] [--list]"
                );
                return ExitCode::SUCCESS;
            }
            other => return usage(&format!("unknown argument {other:?}")),
        }
    }

    let mut cases = match load_cases(&cases_root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("cannot load cases: {e}");
            return ExitCode::from(1);
        }
    };
    if let Some(s) = &suite {
        cases.retain(|c| &c.suite == s);
        if cases.is_empty() {
            eprintln!("no cases in suite {s}");
            return ExitCode::from(1);
        }
    }
    if list {
        for c in &cases {
            println!("{} [{}] {}", c.id, c.suite, c.spec);
        }
        return ExitCode::SUCCESS;
    }
    if !bin.is_file() {
        eprintln!(
            "binary under test not found at {} — build the reference interpreter or pass --bin PATH",
            bin.display()
        );
        return ExitCode::from(1);
    }
    let tmpdir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp/run");
    if let Err(e) = std::fs::create_dir_all(&tmpdir) {
        eprintln!("cannot create {}: {e}", tmpdir.display());
        return ExitCode::from(1);
    }

    println!("binary under test: {}", bin.display());
    let report = run_all(&bin, &cases, &tmpdir);
    for line in report.lines() {
        println!("{line}");
    }
    println!();
    println!("| suite | pass | fail | unavailable | total |");
    for (s, (p, f, u)) in &report.per_suite {
        println!("| {s} | {p} | {f} | {u} | {} |", p + f + u);
    }
    println!(
        "| all | {} | {} | {} | {} |",
        report.passed(),
        report.failed(),
        report.unavailable(),
        cases.len()
    );

    if report.failed() > 0 {
        ExitCode::from(1)
    } else if report.unavailable() > 0 {
        ExitCode::from(2)
    } else {
        ExitCode::SUCCESS
    }
}

fn usage(msg: &str) -> ExitCode {
    eprintln!("{msg}");
    eprintln!("usage: bennett-isa-conformance [--bin PATH] [--cases DIR] [--suite X] [--list]");
    ExitCode::from(1)
}
