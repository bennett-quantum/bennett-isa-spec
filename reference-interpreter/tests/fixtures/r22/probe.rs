//! Executable reference semantics for the Bennett ISA, version 0.8.
#[path = "src/action.rs"]
mod action;
#[path = "src/bits.rs"]
mod bits;
#[path = "src/borrow.rs"]
mod borrow;
#[path = "src/call_discipline.rs"]
mod call_discipline;
#[path = "src/certificate.rs"]
mod certificate;
#[path = "src/claims.rs"]
mod claims;
#[path = "src/classical.rs"]
mod classical;
#[path = "src/controls.rs"]
mod controls;
#[path = "src/exact.rs"]
mod exact;
#[path = "src/handle_validation.rs"]
mod handle_validation;
#[path = "src/identity.rs"]
mod identity;
#[path = "src/json.rs"]
pub mod json;
#[path = "src/linking.rs"]
mod linking;
#[path = "src/machine.rs"]
mod machine;
#[path = "src/metadata.rs"]
mod metadata;
#[path = "src/metadata_schema.rs"]
mod metadata_schema;
#[path = "src/metadata_types.rs"]
mod metadata_types;
#[path = "src/object.rs"]
mod object;
#[path = "src/oracle.rs"]
mod oracle;
#[path = "src/quantum.rs"]
mod quantum;
#[path = "src/registers.rs"]
mod registers;
#[path = "src/schema.rs"]
mod schema;
#[path = "src/sha256.rs"]
mod sha256;
#[path = "src/validation.rs"]
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
    out.len().checked_add(count).ok_or_else(|| {
        Error(format!(
            "{rule}: host limit: {what} cumulative count exceeds usize"
        ))
    })?;
    out.try_reserve_exact(count).map_err(|e| {
        Error(format!(
            "{rule}: host limit: {what} allocation of {count} elements failed: {e}"
        ))
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

fn repin(s: &str) -> Object {
 let o=Object::parse(s).unwrap(); let id=o.identity().unwrap(); let mut text=s.to_owned();
 for sym in o.symbols().unwrap().values() { let name=sym.get("name").unwrap().string().unwrap(); if let Some(new)=id.interface_hashes.get(name) {let old=sym.get("interface_hash").unwrap().string().unwrap();text=text.replace(old,new);} }
 let out=Object::parse(&text).unwrap();out.check_hashes().unwrap();out
}
fn main() {
 let args:Vec<String>=std::env::args().collect(); let mode=&args[1];
 let root="/home/tobias/Projects/bennett-workspace/bennett-isa-spec-review/reference-interpreter";
 if mode=="claim" {
  let s=std::fs::read_to_string(format!("{root}/tests/fixtures/r21/{}",args[2])).unwrap();
  let o=Object::parse(&s).unwrap();o.check_hashes().unwrap();
  let h=o.identity().unwrap().closure_hashes["a"].clone().unwrap();
  println!("parse/hash ok; claim {:?}; certificate {:?}",o.verify_claim(&h,0),o.verify_certificate());
 } else if mode=="transfer" {
  let host=args[2]=="host"; let len:u64=args[3].parse().unwrap();
  let s=std::fs::read_to_string(format!("{root}/tests/fixtures/r21/{}-length.bisa",if host{"cc"}else{"r"})).unwrap();
  let s=s.replace("const.i64 9223372036854775808",&format!("const.i64 {len}"));
  let o=repin(&s);let mut m=ClassicalMachine::new(&o,"f",&[],&[]).unwrap();let entry=m.snapshot();
  m.step_once().unwrap();m.step_once().unwrap(); let before=m.snapshot();
  let result=m.step_once();println!("transfer {len}: {result:?}");
  if len>8 {let e=result.unwrap_err().to_string();assert!(e.contains("host limit")&&e.contains(if host{"§10"}else{"§7.3"}));assert_eq!(m.snapshot(),before);assert!(m.host_transcript().is_empty());assert!(m.console().is_empty());assert_eq!(m.effect_tape_len(),0);assert!(m.outcome().request.is_none());println!("unchanged full snapshot; no request, transcript, console or effect tape");}
  else {result.unwrap();if host {assert_eq!(m.outcome().request.unwrap().payload,vec![0;len as usize]);m.reply(&[7,8]).unwrap();} else {assert_eq!(m.console(),vec![0;len as usize]);assert_eq!(m.effect_tape_len(),1);}}
  m.reverse().unwrap();assert_eq!(m.snapshot(),entry);println!("entry restored");
 } else if mode=="region" {
  let a:u64=args[2].parse().unwrap();
  let m=oracle::Memory{bytes:std::collections::BTreeMap::from([(35,0xa5)]),allocations:vec![],heap_base:1,heap_used:0,table_capacity:None,addresses:std::collections::BTreeMap::from([("review".into(),32)])};
  let r=oracle::region_bytes(&m,"review",a); if a<10 {let v=r.unwrap();assert_eq!(v.len(),1<<a);assert_eq!(v[3],0xa5);println!("region {a}: {} bytes; cell 3=165",v.len());}else {let e=r.unwrap_err().to_string();assert!(e.contains("§4.2")&&e.contains("host limit"));println!("region {a}: {e}");}
 } else if mode=="dense" {
  let n:usize=args[2].parse().unwrap();let mut q=QuantumState::new();let w=q.allocate(n).unwrap();q.gate("x",&[w[n-1]],None,&[]).unwrap();let before=q.clone();let r=q.amplitudes();
  if n<10 {let a=r.unwrap();assert_eq!(a.len(),1<<n);assert_eq!(a[1<<(n-1)].re,1.0);println!("dense {n}: {} entries; expected basis amplitude 1",a.len());}else {let e=r.unwrap_err().to_string();assert!(e.contains("§4.3")&&e.contains("host limit"));println!("dense {n}: {e}");}assert_eq!(q,before);
 } else if mode=="hand" {
  let s=std::fs::read_to_string(format!("{root}/tests/fixtures/inc8.bisa")).unwrap();let o=Object::parse(&s).unwrap();
  for b in [0,2,3,4] {let r=o.run_oracle("inc8",&OracleInput::bits(&[255]),Budget::steps(b)).unwrap(); println!("inc8 x255 B{b}: {r:?}");assert_eq!((r.payload,r.code,r.steps),(0,if b<3{1}else{0},b.min(3)));}
 }
}
