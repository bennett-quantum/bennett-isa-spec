use bennett_isa_ref::{Object, OracleMachine, OracleInput, Budget, json::Json};
fn main() {
 let args:Vec<_>=std::env::args().collect();
 let o=Object::parse(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
 let s=o.section("schedule").unwrap();
 let op=s.get("entries").unwrap().array().unwrap()[0].clone();
 let mut op=op.object().unwrap().clone(); op.insert("generated".into(),Json::Number("0".into()));
 println!("admitted but concrete resolution: {:?}",o.resolve_schedule(&Json::Object(op),"gate"));
 let o=Object::parse(&std::fs::read_to_string(&args[2]).unwrap()).unwrap();
 let mut input=OracleInput::bits(&[0]);input.regions.insert("db".into(),vec![0;16]);
 let mut m=OracleMachine::new(&o,"sum_db",&input,Budget{steps:10,heap:4080,stack:16,depth:64}).unwrap();
 for t in 1..=10 {
  let before=m.snapshot();m.forward_step().unwrap();let after=m.snapshot();
  m.inverse_step().unwrap();assert_eq!(m.snapshot(),before);
  if t==3 || t==9 {
   println!("step {t}: pc {} -> {}, counters {:?} -> {:?}; inverse restored full tuple",before.pc,after.pc,before.counter_values().map(|v|v.to_decimal()),after.counter_values().map(|v|v.to_decimal()));
  }
  m.forward_step().unwrap();assert_eq!(m.snapshot(),after);
 }
 for _ in 0..10 {m.inverse_step().unwrap();} assert!(m.frames_clean());println!("all ten inverse frames clean");
}
