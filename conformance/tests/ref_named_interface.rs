use bennett_isa_conformance::{json::{self, Json}, ref_language::{Expander, Leaf, Registry}, ref_hash};
use std::collections::BTreeMap;
fn fixture() -> Json {json::parse(include_str!("../fixtures/accepted-k82-09/k82.json")).unwrap()}
fn bundles(v:&Json)->Vec<(String,usize)> {let Json::Obj(m)=v else {panic!("bundle object")};m.iter().map(|(n,w)|{let Json::Num(w)=w else {panic!("width")};(n.clone(),*w as usize)}).collect()}
fn normalized(leaves:&[Leaf])->Json {
 let mut ids=BTreeMap::new();
 let mut name=|n:&str| if n.starts_with('@') {let next=ids.len();format!("#{}",*ids.entry(n.to_string()).or_insert(next))}else{n.to_string()};
 Json::Arr(leaves.iter().map(|l| {let wires=if let Some((n,_,_,_))=&l.allocation {vec![Json::Str(name(n))]}else{l.wires.iter().map(|(n,i)|Json::Str(format!("{}[{i}]",name(n)))).collect()};Json::Arr(vec![Json::Str(l.gate.clone()),Json::Arr(wires)])}).collect())
}
#[test]
fn published_k82_binding_cases() {
 let f=fixture();let mut failures=Vec::new();
 for case in f.get("cases").unwrap().as_arr().unwrap() {
  let id=case.get("id").unwrap().as_str().unwrap();let b=bundles(case.get("bundles").unwrap());let borrowed=b.iter().map(|(n,w)|(n.as_str(),*w)).collect::<Vec<_>>();
  let doc=format!(r#"{{"isa":"0.9","name":"ref","version":"0.9","recipes":{}}}"#,case.get("recipes").unwrap());
  let got=Expander::new(&doc,Registry::new()).and_then(|mut e|e.expand(case.get("root").unwrap().as_str().unwrap(),case.get("args").unwrap().as_arr().unwrap(),&borrowed));
  let expected=case.get("expect").unwrap();
  match (got,expected) {
   (Err(_),Json::Str(s)) if s=="reject"=>{},
   (Ok(leaves),Json::Arr(_)) if normalized(&leaves)==*expected=>{},
   (got,_)=>failures.push(format!("{id}: {got:?}; expected {expected}")),
  }
 }
 assert!(failures.is_empty(),"{}",failures.join("\n"));
}
#[test]
fn published_k82_edges_keep_all_86_leaf_digests() {
 let f=fixture();let mut e=Expander::new(include_str!("../../spec/ref-0.9.json"),Registry::new()).unwrap();let mut count=0;
 for edge in f.get("edges").unwrap().as_arr().unwrap() {
  for case in edge.get("instances").unwrap().as_arr().unwrap() {
   let b=bundles(case.get("bundles").unwrap());let borrowed=b.iter().map(|(n,w)|(n.as_str(),*w)).collect::<Vec<_>>();
   let leaves=e.expand(edge.get("caller").unwrap().as_str().unwrap(),case.get("args").unwrap().as_arr().unwrap(),&borrowed).unwrap();
   let canonical=ref_hash::canonical(&normalized(&leaves)).unwrap();
   assert_eq!(ref_hash::sha256(canonical.as_bytes()),case.get("sha256").unwrap().as_str().unwrap());count+=1;
  }
 }
 assert_eq!(count,86);
}

#[test]
fn published_k82_all_54_interfaces_are_exact() {
 let f=fixture();let e=Expander::new(include_str!("../../spec/ref-0.9.json"),Registry::new()).unwrap();
 let Json::Obj(interfaces)=f.get("interfaces").unwrap() else {panic!("interfaces")};assert_eq!(interfaces.len(),54);
 for (name,expected) in interfaces {let names=e.named_interface(name).unwrap();let got=Json::Arr(names.iter().cloned().map(Json::Str).collect());assert_eq!(got,*expected,"{name}");}
}
