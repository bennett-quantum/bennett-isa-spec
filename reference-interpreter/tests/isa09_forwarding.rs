mod support;
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

#[test]
fn source_constructor_binds_lproc_tokens_values_and_region_cells_per_instance() {
 use bennett_isa_conformance::{json,ref_catalog::Machine,ref_language::permutation};
 use bennett_isa_ref::ref_library::OracleInvocation;
 let published=object();
 // Keep the published body and all parameter widths; reduce the scan's address
 // space for this circuit test. The unmodified publication is locked above.
 let layout=support::json(r#"{"A_tab":1,"H":2,"S_cells":4,"globals":{},"heap_base":3,"heap_limit_max":1,"interface_regions":{"plus_db":{"db":{"base":2}},"through":{"db":{"base":1}}},"null_size":1,"ptr_width":32}"#);
 let o=support::rehash(&published.text.replace(&published.section("layout").unwrap().canonical(),&layout.canonical()));
 let args=json::parse(r#"["x","t","f",{"address":0,"cell":8,"role":"in","origin":"interface:db","cells":"db"},["6","0","0","0"],"2"]"#).unwrap();
 let context=json::parse(r#"{"token":["8","0","8","1"]}"#).unwrap();
 let call=OracleInvocation::capture(&o,"through",Some(&context),budget(8,8,1),args.as_arr().unwrap()).unwrap();
 let leaves=call.expand_source(&o.bound_ref_library().unwrap(),&[("x",2),("t",2),("f",4),("db",8)]).unwrap();
 let hash=o.identity().unwrap().closure_hashes["worker"].clone().unwrap();
 assert!(leaves.iter().any(|l|l.source.as_ref().is_some_and(|s|s.terminal.symbol==hash&&s.terminal.instance==[0,1,0])));
 assert!(leaves.iter().any(|l|l.source.as_ref().is_some_and(|s|s.path.iter().any(|e|e.point.symbol==hash&&e.point.instance==[0,1,0]))));
 let m=Machine::compile(&permutation(&leaves).unwrap(),&[("x",2),("t",2),("f",4),("db",8)]).unwrap();
 for x in 0..4 {for d in [0u8,1,2,127,128,255] {let mut state=m.state();m.set(&mut state,"x",x);m.set(&mut state,"t",1);m.set(&mut state,"f",13);m.set(&mut state,"db",d as u64);let entry=state.clone();m.run(&mut state,true,false).unwrap();assert_eq!(m.get(&state,"x"),x);assert_eq!(m.get(&state,"db"),d as u64);assert_eq!(m.get(&state,"t"),(((x+2+(d as u64&3))&3)^1)^1);assert_eq!(m.get(&state,"f"),13);m.run(&mut state,true,true).unwrap();assert_eq!(state,entry);}}
}

fn dual_instance_object()->Object {
 let root=r#"{"abi":"0.9","args":["token","token"],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
 let worker=r#"{"abi":"0.9","args":[],"kind":"lproc","mem":[],"params":[1],"result":1,"tier":"q","tokens":["token"]}"#;
 let id=r#"{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
 support::rehash(&support::object(&[("o",root,"oracle @o (x:wire in) (t:wire) (f:wire[4]) () (left:token, right:token)\nregs { r:reg[1] }\n^entry():\nqbranch x ^left ^right\n^left():\nr = lift.call @w (x) (left) () ()\nqjump ^done\n^right():\nr = lift.call @w (x) (right) () ()\nqjump ^done\n^done():\nlift.retval r"),("w",worker,"lproc @w (x:reg[1]) (r:reg[1]) (tk:token) () ()\nregs { fr:reg[4], bit:reg[1], zero:reg[4], bad:reg[1] }\n^entry():\nlift.oracle_call @id tk (x) r fr () ()\nbit = lift.trunc.i1 fr\nlift.oracle_uncall @id tk (x) r fr () ()\nzero = lift.const.i4 0\nbad = lift.icmp.ne fr zero\nlift.faultif bad 9\nlift.ret bit"),("id",id,"oracle @id (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x")],"{}","\nsection layout\n{\"A_tab\":1,\"H\":4,\"S_cells\":4,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n").replace("=0.8","=0.9"))
}
#[test]
fn coherent_paths_keep_two_lproc_instances_even_when_entry_token_values_are_equal() {
 let o=dual_instance_object();let b=budget(11,9,1);let widths=bennett_isa_ref::DerivedWidths::derive(&o,"o",b).unwrap();assert_eq!((widths.p,widths.did),(5,4));
 for (left,right) in [(0,1),(1,1)] {for x in 0..2 {
  let input=OracleInput{ins:vec![x],args:vec![Argument::Token(Budget::steps(left)),Argument::Token(Budget::steps(right))],..Default::default()};let expected=if x==1 {u128::from(left==0)}else{u128::from(right==0)};
  let out=o.run_oracle("o",&input,b).unwrap();assert_eq!((out.payload,out.code),(expected,0));let mut machine=OracleMachine::new(&o,"o",&input,b).unwrap();let entry=machine.snapshot();for _ in 0..b.steps {machine.forward_step().unwrap();}assert_eq!(machine.commit(1,13).unwrap(),(1^expected,13));for _ in 0..b.steps {machine.inverse_step().unwrap();}assert_eq!(machine.snapshot(),entry);
 }}
}
#[test]
fn recursive_forwarding_closes_over_two_permuted_binding_keys() {
 let root=r#"{"abi":"0.9","args":["token","token"],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
 let worker=r#"{"abi":"0.9","args":[],"kind":"lproc","mem":[],"params":[1],"result":1,"tier":"q","tokens":["token","token"]}"#;
 let id=r#"{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
 let source=support::object(&[("o",root,"oracle @o (x:wire in) (t:wire) (f:wire[4]) () (left:token, right:token)\nregs { r:reg[1] }\n^entry():\nr = lift.call @w (x) (left, right) () ()\nlift.retval r"),("w",worker,"lproc @w (x:reg[1]) (r:reg[1]) (tk:token, other:token) () ()\nregs { zero:reg[1], z:reg[1], n:reg[1], fr:reg[4], bit:reg[1] }\n^entry():\nzero = lift.const.i1 0\nz = lift.icmp.eq x zero\nqbranch z ^base ^recurse\n^recurse():\nn = lift.const.i1 0\nr = lift.call @w (n) (other, tk) () ()\nlift.ret r\n^base():\nlift.oracle_call @id tk (x) r fr () ()\nbit = lift.trunc.i1 fr\nlift.oracle_uncall @id tk (x) r fr () ()\nlift.ret bit"),("id",id,"oracle @id (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x")],"{}","\nsection layout\n{\"A_tab\":1,\"H\":4,\"S_cells\":5,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n").replace("=0.8","=0.9");let o=support::rehash(&source);let b=budget(15,20,2);let widths=bennett_isa_ref::DerivedWidths::derive(&o,"o",b).unwrap();assert_eq!((widths.p,widths.did),(5,4));
 for x in 0..2 {let input=OracleInput{ins:vec![x],args:vec![Argument::Token(Budget::steps(0)),Argument::Token(Budget::steps(1))],..Default::default()};let expected=x^1;let out=o.run_oracle("o",&input,b).unwrap();assert_eq!((out.payload,out.code),(expected,0));let mut machine=OracleMachine::new(&o,"o",&input,b).unwrap();let entry=machine.snapshot();for _ in 0..b.steps {machine.forward_step().unwrap();}assert_eq!(machine.commit(1,7).unwrap(),(1^expected,7));for _ in 0..b.steps {machine.inverse_step().unwrap();}assert_eq!(machine.snapshot(),entry);}
}
