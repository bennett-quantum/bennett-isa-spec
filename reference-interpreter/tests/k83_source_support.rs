mod support;
use bennett_isa_conformance::{json::{self,Json},ref_catalog::Machine,ref_language::{invert_trace,permutation,Leaf}};
use bennett_isa_ref::{Object,QuantumBindings,QuantumState,sha256_hex};
use std::{collections::BTreeMap,path::PathBuf};
const VECTOR:&str=include_str!("../../conformance/fixtures/accepted-k83-09/k83.json");
const VECTOR_HASH:&str="f3e3910d285edfd6d6b756f60aad82eeb1e8b7e70eecd133f730763d880ac41b";
fn count(value:&Json)->usize {let Json::Num(number)=value else {panic!("integer")};assert_eq!(number.fract(),0.0);assert!(*number>=0.0);*number as usize}
fn text(value:&Json)->&str {value.as_str().unwrap()}
fn array(values:impl IntoIterator<Item=Json>)->Json {Json::Arr(values.into_iter().collect())}
fn number(value:usize)->Json {Json::Num(value as f64)}
fn normalized(leaves:&[Leaf],complete:bool)->Json {
 let mut allocations=BTreeMap::new();let mut events=Vec::new();
 for leaf in leaves {
  let source=leaf.source.as_ref().expect("every source leaf has an address");
  if !complete && !matches!(source.terminal.point.as_str(),"$construct"|"$release") {continue;}
  let owner=if let Some((name,_,class,state))=&leaf.allocation {
   assert_eq!((class.as_str(),state.as_str()),("any","clean"));
   if leaf.gate=="alloc" {let alias=format!("a{}",allocations.len());assert!(allocations.insert(name.clone(),alias).is_none(),"separate calls must not alias allocations");}
   allocations.get(name).unwrap().clone()
  } else {let name=&leaf.wires[0].0;allocations.get(name).unwrap_or(name).clone()};
  let support=matches!(source.terminal.point.as_str(),"$construct"|"$release");
  if support {assert!(leaf.controls.is_empty(),"region preparation/restoration is unconditional");assert!(source.terminal.iteration.is_empty());assert!(source.terminal.instance.is_empty());assert!(source.steps.is_empty());}
  for edge in &source.path {assert_eq!(edge.call_index,0);}
  events.push(array([
   Json::Str(leaf.gate.clone()),Json::Str(source.terminal.point.clone()),number(leaf.ordinal),Json::Str(owner),
   if leaf.allocation.is_some() {Json::Null}else{number(leaf.wires[0].1)},
   array(source.terminal.iteration.iter().copied().map(number)),array(source.terminal.instance.iter().copied().map(number)),
   source.path.last().map_or(Json::Null,|edge|array(edge.point.iteration.iter().copied().map(number))),
  ]));
 }
 array(events)
}
fn expect_interface(name:&str,bundle:&str,value:u64)->u64 {if matches!(name,"zero-trip"|"qif-arms") && bundle=="t" {value^1}else{value}}
#[test]
fn accepted_k83_vectors_match_actual_source_coordinates_and_reversible_execution() {
 assert_eq!(sha256_hex(VECTOR.as_bytes()),VECTOR_HASH);
 let vectors=json::parse(VECTOR).unwrap();
 for case in vectors.get("cases").unwrap().as_arr().unwrap() {
  let name=text(case.get("name").unwrap());
  let path=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../conformance/fixtures/accepted-k83-09").join(text(case.get("object").unwrap()));
  let bytes=std::fs::read_to_string(path).unwrap();assert_eq!(sha256_hex(bytes.as_bytes()),text(case.get("sha256").unwrap()));
  let object=Object::parse(&bytes).unwrap();object.check_hashes().unwrap();
  let bundles=case.get("bundles").unwrap().as_arr().unwrap().iter().map(|bundle|{let fields=bundle.as_arr().unwrap();(text(&fields[0]),count(&fields[1]))}).collect::<Vec<_>>();
  let entry=text(case.get("entry").unwrap());let actuals=case.get("actuals").unwrap().as_arr().unwrap();
  let leaves=object.expand_action_source(entry,actuals,&bundles,false).unwrap();
  assert_eq!(normalized(&leaves,case.get("complete_trace").unwrap().as_bool().unwrap()),*case.get("trace").unwrap(),"{name}");
  let mut raw_inverse=leaves.clone();invert_trace(&mut raw_inverse).unwrap();
  for (forward,inverse) in leaves.iter().zip(raw_inverse.iter().rev()) {
   assert_eq!(forward.source,inverse.source);assert_eq!(forward.ordinal,inverse.ordinal);assert_eq!(forward.invocation,inverse.invocation);assert_eq!(forward.allocation,inverse.allocation);assert_eq!(forward.wires,inverse.wires);assert_eq!(forward.controls,inverse.controls);
  }
  for trace in [&leaves,&raw_inverse] {
   let machine=Machine::compile(&permutation(trace).unwrap(),&bundles).unwrap();
   for basis in 0..1u64<<bundles.iter().map(|(_,width)|width).sum::<usize>() {
    let mut state=machine.state();let mut offset=0;
    for (bundle,width) in &bundles {machine.set(&mut state,bundle,(basis>>offset)&((1<<width)-1));offset+=width;}
    let before=state.clone();machine.run(&mut state,true,false).unwrap();offset=0;
    for (bundle,width) in &bundles {let value=(basis>>offset)&((1<<width)-1);assert_eq!(machine.get(&state,bundle),expect_interface(name,bundle,value),"{name}");offset+=width;}
    machine.run(&mut state,true,true).unwrap();assert_eq!(state,before,"{name}");
   }
  }
  for basis in 0..1u64<<bundles.iter().map(|(_,width)|width).sum::<usize>() {
   let mut quantum=QuantumState::new();let mut bindings=QuantumBindings::default();let mut offset=0;
   for (bundle,width) in &bundles {let wires=quantum.allocate(*width).unwrap();for (bit,wire) in wires.iter().enumerate() {if (basis>>(offset+bit))&1!=0 {quantum.gate("x",&[*wire],None,&[]).unwrap();}}offset+=width;bindings.bundles.insert((*bundle).into(),wires);}
   let before=quantum.amplitudes().unwrap();let width=quantum.wire_count();object.execute_action(entry,&mut quantum,&bindings,false,&[]).unwrap();offset=0;
   for (bundle,width) in &bundles {let value=(basis>>offset)&((1<<width)-1);assert_eq!(quantum.read_basis(&bindings.bundles[*bundle]).unwrap(),u128::from(expect_interface(name,bundle,value)));offset+=width;}
   object.execute_action(entry,&mut quantum,&bindings,true,&[]).unwrap();assert_eq!(quantum.wire_count(),width);assert_eq!(quantum.amplitudes().unwrap(),before);
  }
 }
}
