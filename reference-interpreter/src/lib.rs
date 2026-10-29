//! Executable reference semantics for the Bennett ISA, version 0.8.
pub mod json;
mod object;
mod sha256;
mod identity;
mod validation;
mod oracle;
mod machine;
pub use object::*;
pub use sha256::{sha256,sha256_hex};
pub use identity::Identity;
pub use oracle::{Argument,Budget,OracleInput,OracleOutcome};
pub use machine::{BasisConfiguration,OracleMachine};

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
pub fn version() -> &'static str { "0.8" }
