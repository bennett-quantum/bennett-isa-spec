use bennett_isa_ref::{Object,Argument,Budget,OracleInput,OracleMachine};
use std::collections::BTreeMap;
fn object()->Object {let spec=include_str!("../../spec/Bennett-ISA-Spec-0.9.md");let body=spec.split_once("**Example 10**").unwrap().1.split_once("```\n").unwrap().1.split_once("\n```").unwrap().0;let o=Object::parse(&format!("{body}\n")).unwrap();o.check_hashes().unwrap();o}
fn budget(steps:u64,stack:u128,depth:u128)->Budget {Budget{steps,heap:0,stack,depth}}
fn input(x:u128,a:u128,byte:u8,nested:u64)->OracleInput {OracleInput{ins:vec![x],args:vec![Argument::Token(Budget::steps(nested)),Argument::Bits(a)],regions:BTreeMap::from([("db".into(),vec![byte])])}}
#[test]
fn published_example10_forwards_every_input_argument_and_cell_without_extra_charge() {
 let o=object();
 for x in 0..4 {for a in 0..4 {for d in 0..=255 {
  let input=input(x,a,d,6);let want=((x+a+u128::from(d&3))&3)^1;
  let out=o.run_oracle("through",&input,budget(8,8,1)).unwrap();assert_eq!((out.payload,out.code),(want,0),"x={x} a={a} d={d}");
  let mut machine=OracleMachine::new(&o,"through",&input,budget(8,8,1)).unwrap();let entry=machine.snapshot();
  for _ in 0..8 {machine.forward_step().unwrap();}
  assert_eq!(machine.commit(2,13).unwrap(),(want^2,13));
  for _ in 0..8 {machine.inverse_step().unwrap();}assert_eq!(machine.snapshot(),entry);
 }}}
}
#[test]
fn forwarded_nested_faults_and_outer_boundaries_reverse_the_complete_prefix() {
 let o=object();
 for (outer,nested,code) in [(budget(7,8,1),6,1),(budget(8,7,1),6,3),(budget(8,8,0),6,3),(budget(5,8,1),5,9)] {
  let input=input(3,2,255,nested);let out=o.run_oracle("through",&input,outer).unwrap();assert_eq!((out.payload,out.code),(0,code));
  let mut machine=OracleMachine::new(&o,"through",&input,outer).unwrap();let entry=machine.snapshot();for _ in 0..outer.steps {machine.forward_step().unwrap();}assert_eq!(machine.commit(3,10).unwrap(),(3,10^code));for _ in 0..outer.steps {machine.inverse_step().unwrap();}assert_eq!(machine.snapshot(),entry);
 }
}
