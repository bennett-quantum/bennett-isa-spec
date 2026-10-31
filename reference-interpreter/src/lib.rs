//! Executable reference semantics for the Bennett ISA, version 0.8.
mod action;
mod bits;
mod borrow;
mod call_discipline;
mod certificate;
mod claims;
mod classical;
mod controls;
mod exact;
mod handle_validation;
mod identity;
pub mod json;
mod linking;
mod machine;
mod metadata;
mod metadata_schema;
mod metadata_types;
mod object;
mod oracle;
mod quantum;
mod registers;
mod retained;
pub use retained::Retained;
mod schema;
mod sha256;
mod validation;
pub use bits::BitVector;
pub use borrow::validate_translation_lemma;
pub use claims::{ClaimReport, ClaimVerdict, Xoshiro256};
pub use classical::{
    ClassicalMachine, ClassicalOutcome, ClassicalSnapshot, ClassicalValue, RegisterDelta,
    SnapshotDelta, StepPoint, TapeEntry,
};
pub use classical::{HostExchange, HostRequest, HostRequestData};
pub use identity::Identity;
pub use machine::{
    BasisConfiguration, BasisVectorConfiguration, DerivedWidths, OracleMachine, OracleVectorMachine,
};
pub use metadata::{ScheduleEpsilonShare, ScheduleExpansion};
pub use object::*;
pub use oracle::{
    Argument, Budget, OracleInput, OracleOutcome, OracleVectorInput, OracleVectorOutcome,
};
pub use quantum::{Complex, MeasurementBranch, QuantumBindings, QuantumRegion, QuantumState};
pub use registers::QuantumRegisters;
pub use sha256::{sha256, sha256_hex};

/// A diagnostic always names the normative rule that was violated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub String, pub ErrorKind);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Diagnostic,
    HostResource,
}
impl Error {
    pub fn new(message: String) -> Self {
        Self(message, ErrorKind::Diagnostic)
    }
    pub fn host_resource(message: String) -> Self {
        Self(message, ErrorKind::HostResource)
    }
    pub fn kind(&self) -> ErrorKind {
        self.1
    }
    pub fn context(self, rule: &str) -> Self {
        Self(format!("{rule}: {}", self.0), self.1)
    }
}
pub(crate) fn resource_error(rule: &str, message: impl std::fmt::Display) -> Error {
    Error(
        format!("{rule}: host limit: {message}"),
        ErrorKind::HostResource,
    )
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
pub(crate) fn err<T>(rule: &str, message: impl std::fmt::Display) -> Result<T> {
    Err(Error::new(format!("{rule}: {message}")))
}
pub fn version() -> &'static str {
    "0.8"
}

/// Reserve text-derived storage fallibly before iteration or initialization.
pub(crate) fn reserved<T>(count: usize, rule: &str, what: &str) -> Result<Vec<T>> {
    let mut out = Vec::new();
    reserve_more(&mut out, count, rule, what)?;
    Ok(out)
}

/// Check cumulative storage before a count-driven loop or any visible mutation.
pub(crate) fn reserve_more<T>(
    out: &mut Vec<T>,
    count: usize,
    rule: &str,
    what: &str,
) -> Result<()> {
    out.len()
        .checked_add(count)
        .ok_or_else(|| resource_error(rule, format!("{what} cumulative count exceeds usize")))?;
    out.try_reserve_exact(count).map_err(|e| {
        resource_error(
            rule,
            format!("{what} allocation of {count} elements failed: {e}"),
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod r21_tests {
    use super::*;
    #[test]
    fn r21_derived_consumers_memory_limited() {
        if std::env::var_os("R21_UNIT_CHILD").is_none() {
            let out = std::process::Command::new("python3").args(["-c",
                "import os,resource,sys; resource.setrlimit(resource.RLIMIT_AS,(256*1024*1024,)*2); resource.setrlimit(resource.RLIMIT_CORE,(0,0)); os.execv(sys.argv[1],sys.argv[1:])"])
                .arg(std::env::current_exe().unwrap())
                .args(["--exact", "r21_tests::r21_derived_consumers_memory_limited", "--nocapture"])
                .env("R21_UNIT_CHILD", "1").output().unwrap();
            assert!(out.status.success(), "{out:?}");
            eprintln!("{}", String::from_utf8_lossy(&out.stderr));
            return;
        }
        std::panic::catch_unwind(|| {
            crate::oracle::r21_resource_cases();
            crate::claims::r21_resource_cases();
            crate::classical::r21_resource_cases();
            for count in [0, 1, 8, (1usize << 63) - 1, 1 << 63, usize::MAX] {
                let mut console = vec![0u8; 8];
                let result = reserve_more(&mut console, count, "§7.3", "console bytes");
                if count <= 8 {
                    result.unwrap();
                } else {
                    let e = result.unwrap_err().to_string();
                    assert!(e.contains("§7.3") && e.contains("host limit"), "{e}");
                }
                assert_eq!(console, vec![0; 8]);
            }
            eprintln!("R21 cumulative console storage: 1 path / 6 cases");
        })
        .expect("no derived-cardinality or transfer panic");
    }
}
