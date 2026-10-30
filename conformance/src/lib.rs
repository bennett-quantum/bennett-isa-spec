//! Bennett ISA conformance suites A-H as data plus a harness.
//!
//! Each case is a directory under `cases/` holding a `case.json`
//! (the spec sentence it tests, a hand derivation of the expected
//! outcome, the CLI command, and the expected outcome) plus the
//! fixture files it references. The harness runs any binary that
//! implements the reference CLI (`check`, `run-oracle`, `step-check`,
//! `run-cc`, `reverse`, `run-action`, `run-program`) and compares the
//! spec-defined outcome fields as a JSON subset.
//!
//! Cases for staged deliverables (spec §15 suites served by
//! reference-interpreter deliverables 8-11) carry an `unavailable`
//! block. They are never skipped: where a CLI probe exists the
//! harness runs it and requires an explicit refusal; the case
//! then reports `UNAVAILABLE: deliverable N`. Where no probe exists
//! the case still reports UNAVAILABLE. A probe that the binary
//! silently accepts is a FAIL.

pub mod json;
pub mod ref_amplitudes;
pub mod ref_catalog;
pub mod ref_hash;
pub mod ref_language;

use json::Json;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Default case root: this crate's `cases/` directory.
pub fn cases_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cases")
}

/// Default binary under test: `$BENNETT_CONFORMANCE_BIN`, else the
/// reference interpreter's debug binary in the sibling crate.
pub fn default_bin() -> PathBuf {
    if let Ok(p) = std::env::var("BENNETT_CONFORMANCE_BIN") {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../reference-interpreter/target/debug/bennett-isa-ref")
}

#[derive(Debug, Clone)]
pub struct Unavailable {
    pub deliverable: u32,
    pub reason: String,
    /// Explicit refusal alternative, used only while this case is staged.
    pub expect: Option<Expect>,
}

#[derive(Debug, Clone, Default)]
pub struct Expect {
    pub exit: Option<i32>,
    pub stdout_json: Option<Json>,
    pub stdout_empty: bool,
    pub stderr_contains: Vec<String>,
    /// Pairs of dotted paths into the stdout JSON that must be equal
    /// (e.g. a reversal's `memory_digest` vs `entry_memory_digest`).
    pub equal_fields: Vec<(String, String)>,
    /// Dotted paths where the `stdout_json` subtree must match
    /// exactly (no extra keys), e.g. a quantum `state`.
    pub exact_at: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub id: String,
    pub suite: String,
    pub spec: String,
    pub derivation: String,
    /// Resolved absolute path of the referenced object, if any.
    pub object: Option<PathBuf>,
    pub command: Option<Vec<String>>,
    pub expect: Expect,
    pub unavailable: Option<Unavailable>,
    pub dir: PathBuf,
}

#[derive(Debug)]
pub enum Outcome {
    Pass,
    Fail(String),
    Unavailable {
        deliverable: u32,
        reason: String,
        /// `true` when a probe command ran and confirmed the loud
        /// refusal the case expects (precise error or explicit staging).
        probe_confirmed: bool,
    },
}

/// Load every `case.json` under `root` (recursively), sorted by id.
pub fn load_cases(root: &Path) -> Result<Vec<Case>, String> {
    let mut files = Vec::new();
    collect_case_files(root, &mut files).map_err(|e| format!("{}: {e}", root.display()))?;
    files.sort();
    let mut cases = Vec::new();
    for f in files {
        cases.push(load_case(&f)?);
    }
    cases.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(cases)
}

fn collect_case_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_case_files(&path, out)?;
        } else if path.file_name().map(|n| n == "case.json").unwrap_or(false) {
            out.push(path);
        }
    }
    Ok(())
}

fn load_case(file: &Path) -> Result<Case, String> {
    let text = fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let doc = json::parse(&text).map_err(|e| format!("{}: {e}", file.display()))?;
    let dir = file
        .parent()
        .ok_or("case.json has no parent")?
        .to_path_buf();
    let req_str = |key: &str| -> Result<String, String> {
        doc.get(key)
            .and_then(Json::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("{}: missing string field {key:?}", file.display()))
    };
    let id = req_str("id")?;
    let object = match doc.get("object") {
        None | Some(Json::Null) => None,
        Some(v) => {
            let rel = v
                .as_str()
                .ok_or_else(|| format!("{id}: object must be a string path"))?;
            Some(dir.join(rel))
        }
    };
    let command = match doc.get("command") {
        None | Some(Json::Null) => None,
        Some(v) => {
            let arr = v
                .as_arr()
                .ok_or_else(|| format!("{id}: command must be an array of strings"))?;
            let mut cmd = Vec::new();
            for a in arr {
                cmd.push(
                    a.as_str()
                        .ok_or_else(|| format!("{id}: command entries must be strings"))?
                        .to_string(),
                );
            }
            if cmd.is_empty() {
                return Err(format!("{id}: empty command array"));
            }
            Some(cmd)
        }
    };
    let expect = parse_expect(&id, doc.get("expect"))?;
    let unavailable = match doc.get("unavailable") {
        None | Some(Json::Null) => None,
        Some(u) => {
            let deliverable = match u.get("deliverable") {
                Some(Json::Num(n)) if n.fract() == 0.0 && *n >= 0.0 => *n as u32,
                _ => return Err(format!("{id}: unavailable.deliverable must be an integer")),
            };
            let reason = u
                .get("reason")
                .and_then(Json::as_str)
                .ok_or_else(|| format!("{id}: unavailable.reason is required"))?
                .to_string();
            let staged_expect = u
                .get("expect")
                .map(|v| parse_expect(&id, Some(v)))
                .transpose()?;
            if let Some(ref staged) = staged_expect {
                // An alternative is permission to report missing mechanics,
                // never permission to accept success or an arbitrary error.
                let marker = format!("staged deliverable {deliverable}");
                if command.is_none()
                    || expect.exit != Some(1)
                    || !expect.stdout_empty
                    || expect.stderr_contains.is_empty()
                    || staged.exit != Some(1)
                    || !staged.stdout_empty
                    || staged.stdout_json.is_some()
                    || !staged.stderr_contains.contains(&marker)
                {
                    return Err(format!(
                        "{id}: unavailable.expect requires an explicit {marker:?} refusal (exit 1, empty stdout) and a precise primary error"
                    ));
                }
            }
            Some(Unavailable {
                deliverable,
                reason,
                expect: staged_expect,
            })
        }
    };
    Ok(Case {
        id,
        suite: req_str("suite")?,
        spec: req_str("spec")?,
        derivation: req_str("derivation")?,
        object,
        command,
        expect,
        unavailable,
        dir,
    })
}

fn parse_expect(id: &str, v: Option<&Json>) -> Result<Expect, String> {
    let mut e = Expect::default();
    let Some(v) = v else { return Ok(e) };
    if let Some(x) = v.get("exit") {
        match x {
            Json::Num(n) if n.fract() == 0.0 => e.exit = Some(*n as i32),
            _ => return Err(format!("{id}: expect.exit must be an integer")),
        }
    }
    if let Some(x) = v.get("stdout_json") {
        e.stdout_json = Some(x.clone());
    }
    if let Some(x) = v.get("stdout_empty") {
        e.stdout_empty = x
            .as_bool()
            .ok_or_else(|| format!("{id}: expect.stdout_empty must be a boolean"))?;
    }
    if let Some(x) = v.get("stderr_contains") {
        for s in x
            .as_arr()
            .ok_or_else(|| format!("{id}: expect.stderr_contains must be an array"))?
        {
            e.stderr_contains.push(
                s.as_str()
                    .ok_or_else(|| format!("{id}: stderr_contains entries must be strings"))?
                    .to_string(),
            );
        }
    }
    if let Some(x) = v.get("stdout_json_exact_at") {
        for s in x
            .as_arr()
            .ok_or_else(|| format!("{id}: stdout_json_exact_at must be an array"))?
        {
            e.exact_at.push(
                s.as_str()
                    .ok_or_else(|| format!("{id}: exact_at entries must be strings"))?
                    .to_string(),
            );
        }
    }
    if let Some(x) = v.get("stdout_json_equal_fields") {
        for pair in x
            .as_arr()
            .ok_or_else(|| format!("{id}: stdout_json_equal_fields must be an array"))?
        {
            let p = pair
                .as_arr()
                .filter(|a| a.len() == 2)
                .ok_or_else(|| format!("{id}: equal-field entries must be [path, path]"))?;
            let a = p[0]
                .as_str()
                .ok_or_else(|| format!("{id}: equal-field paths must be strings"))?;
            let b = p[1]
                .as_str()
                .ok_or_else(|| format!("{id}: equal-field paths must be strings"))?;
            e.equal_fields.push((a.to_string(), b.to_string()));
        }
    }
    Ok(e)
}

/// Run one case against `bin`. Child processes run headless
/// (CLAUDE.md rule 11): display/browser variables removed and
/// `TMPDIR` set to `tmpdir` (created by the caller).
pub fn run_case(bin: &Path, case: &Case, tmpdir: &Path) -> Outcome {
    let Some(command) = &case.command else {
        return match &case.unavailable {
            Some(u) => Outcome::Unavailable {
                deliverable: u.deliverable,
                reason: u.reason.clone(),
                probe_confirmed: false,
            },
            None => Outcome::Fail(format!(
                "{}: available case has no command (case data error)",
                case.id
            )),
        };
    };
    let object_str = case
        .object
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let dir_str = case.dir.to_string_lossy().into_owned();
    let args: Vec<String> = command
        .iter()
        .map(|a| {
            a.replace("{object}", &object_str)
                .replace("{dir}", &dir_str)
        })
        .collect();
    let output = match Command::new(bin)
        .args(&args)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("DBUS_SESSION_BUS_ADDRESS")
        .env_remove("BROWSER")
        .env("TMPDIR", tmpdir)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return Outcome::Fail(format!(
                "{}: cannot run {} {}: {e}",
                case.id,
                bin.display(),
                args.join(" ")
            ))
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let exit = output.status.code().unwrap_or(-1);
    if let Err(msg) = check_expect(&case.expect, exit, &stdout, &stderr) {
        let staged = case.unavailable.as_ref().and_then(|u| u.expect.as_ref());
        match staged {
            Some(expect) => {
                if let Err(refusal) = check_expect(expect, exit, &stdout, &stderr) {
                    return Outcome::Fail(format!(
                        "{}: {msg}; staged alternative: {refusal}",
                        case.id
                    ));
                }
            }
            None => return Outcome::Fail(format!("{}: {msg}", case.id)),
        }
    }
    match &case.unavailable {
        Some(u) => Outcome::Unavailable {
            deliverable: u.deliverable,
            reason: u.reason.clone(),
            probe_confirmed: true,
        },
        None => Outcome::Pass,
    }
}

fn check_expect(exp: &Expect, exit: i32, stdout: &str, stderr: &str) -> Result<(), String> {
    if let Some(want) = exp.exit {
        if exit != want {
            return Err(format!(
                "exit {exit}, expected {want}; stdout={stdout:?} stderr={stderr:?}"
            ));
        }
    }
    if exp.stdout_empty && !stdout.trim().is_empty() {
        return Err(format!("stdout not empty: {stdout:?}"));
    }
    for needle in &exp.stderr_contains {
        if !stderr.contains(needle.as_str()) {
            return Err(format!(
                "stderr does not contain {needle:?}; stderr={stderr:?} stdout={stdout:?}"
            ));
        }
    }
    let mut parsed: Option<Json> = None;
    if exp.stdout_json.is_some() || !exp.equal_fields.is_empty() {
        parsed = Some(
            json::parse(stdout.trim())
                .map_err(|e| format!("stdout is not one JSON document ({e}): {stdout:?}"))?,
        );
    }
    if let (Some(want), Some(got)) = (&exp.stdout_json, &parsed) {
        if let Some(d) = json::subset_diff(want, got) {
            return Err(format!(
                "stdout JSON mismatch at {d}; full stdout={stdout:?}"
            ));
        }
        for path in &exp.exact_at {
            let (w, g) = (want.at_path(path), got.at_path(path));
            match (w, g) {
                (Some(w), Some(g)) => {
                    if let Some(d) = json::exact_diff(w, g) {
                        return Err(format!(
                            "stdout JSON at {path} must match exactly: {d}; full stdout={stdout:?}"
                        ));
                    }
                }
                _ => {
                    return Err(format!(
                        "exact_at path {path} missing in expected or actual JSON"
                    ))
                }
            }
        }
    }
    if let Some(got) = &parsed {
        for (a, b) in &exp.equal_fields {
            let va = got.at_path(a);
            let vb = got.at_path(b);
            match (va, vb) {
                (Some(x), Some(y)) if x == y => {}
                (x, y) => {
                    return Err(format!(
                        "fields {a} and {b} must be equal, got {x:?} vs {y:?}"
                    ))
                }
            }
        }
    }
    Ok(())
}

/// Per-suite tallies plus one human-readable line per case.
#[derive(Debug, Default)]
pub struct Report {
    pub lines: Vec<String>,
    /// suite -> (pass, fail, unavailable)
    pub per_suite: BTreeMap<String, (usize, usize, usize)>,
}

impl Report {
    pub fn passed(&self) -> usize {
        self.per_suite.values().map(|v| v.0).sum()
    }
    pub fn failed(&self) -> usize {
        self.per_suite.values().map(|v| v.1).sum()
    }
    pub fn unavailable(&self) -> usize {
        self.per_suite.values().map(|v| v.2).sum()
    }
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}

pub fn run_all(bin: &Path, cases: &[Case], tmpdir: &Path) -> Report {
    let mut report = Report::default();
    for case in cases {
        let tally = report.per_suite.entry(case.suite.clone()).or_default();
        match run_case(bin, case, tmpdir) {
            Outcome::Pass => {
                tally.0 += 1;
                report.lines.push(format!("PASS {}", case.id));
            }
            Outcome::Fail(msg) => {
                tally.1 += 1;
                report.lines.push(format!("FAIL {msg}"));
            }
            Outcome::Unavailable {
                deliverable,
                reason,
                probe_confirmed,
            } => {
                tally.2 += 1;
                report.lines.push(format!(
                    "UNAVAILABLE {}: unavailable: deliverable {} — {}{}",
                    case.id,
                    deliverable,
                    reason,
                    if probe_confirmed {
                        " (probe confirmed explicit refusal)"
                    } else {
                        " (no CLI probe exists)"
                    }
                ));
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_draft() {
        assert_eq!(super::version(), "0.0.0");
    }
}
