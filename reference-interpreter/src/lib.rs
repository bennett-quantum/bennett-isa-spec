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
mod metadata_types;
mod object;
mod oracle;
mod quantum;
mod registers;
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
pub use classical::{HostExchange, HostRequest};
pub use identity::Identity;
pub use machine::{
    BasisConfiguration, BasisVectorConfiguration, DerivedWidths, OracleMachine, OracleVectorMachine,
};
pub use object::*;
pub use oracle::{
    Argument, Budget, OracleInput, OracleOutcome, OracleVectorInput, OracleVectorOutcome,
};
pub use quantum::{Complex, MeasurementBranch, QuantumBindings, QuantumRegion, QuantumState};
pub use registers::QuantumRegisters;
pub use sha256::{sha256, sha256_hex};

/// A diagnostic always names the normative rule that was violated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
pub(crate) fn err<T>(rule: &str, message: impl std::fmt::Display) -> Result<T> {
    Err(Error(format!("{rule}: {message}")))
}
pub fn version() -> &'static str {
    "0.8"
}
