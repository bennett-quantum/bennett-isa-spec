mod support;
use bennett_isa_ref::{validate_translation_lemma,Instruction,Object,QuantumBindings,QuantumState};
use std::collections::BTreeSet;
use support::object;
fn gate(op:&str,operands:&[&str])->Instruction{Instruction{lhs:Vec::new(),op:op.into(),operands:operands.iter().map(|s|(*s).into()).collect(),bodies:Vec::new(),line:1}}
#[test]
fn translation_lemma_checks_all_four_required_negative_patterns(){
    let b=BTreeSet::from(["b".into(),"b2".into()]);let x=BTreeSet::from(["x".into()]);let v=[gate("cx",&["x","b"])];let w=[gate("cx",&["b","t"])];validate_translation_lemma(&v,&w,&x,&b).unwrap();
    // A V;W;V^-1 prefix lacks the required final W and is checked as a full frame below.
    assert!(validate_translation_lemma(&[gate("cx",&["b2","b"])],&w,&BTreeSet::from(["b2".into()]),&b).is_err());
    assert!(validate_translation_lemma(&[gate("cx",&["b","x"])],&w,&BTreeSet::from(["b".into()]),&b).is_err());
    assert!(validate_translation_lemma(&v,&[gate("swap",&["y","z"]),gate("cx",&["b","y"])],&x,&b).is_err());
    assert!(validate_translation_lemma(&v,&[gate("cx",&["b","y"]),gate("cx",&["y","z"])],&x,&b).is_err());
}
fn borrowed(body:&str)->String {
    let signature=r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[],"mut":[1,1,1],"tier":"q"}"#;
    object(&[("a",signature,&format!("perm_action @a (t:wire mut, y:wire mut, z:wire mut) (x:wire in) () () ()\n^entry():\nb = qborrow any 1\n{body}\nqreturn b"))],"{}","")
}
#[test]
fn borrow_frame_exhaustion_rejects_hidden_dependence_on_borrowed_bits(){
    let raw=borrowed("cx b y\ncx y z\ncx b y");let error=Object::parse(&raw).expect_err("borrowed-dependent output accepted");assert!(error.to_string().contains("§5.6"));
}
#[test]
fn translation_pattern_returns_an_entangled_borrowed_system(){
    let o=Object::parse(&borrowed("cx x b\ncx b t\ncx x b\ncx b t")).unwrap();
    let mut q=QuantumState::new();let t=q.allocate(1).unwrap();let y=q.allocate(1).unwrap();let z=q.allocate(1).unwrap();let x=q.allocate(1).unwrap();let b=q.allocate(1).unwrap();let reference=q.allocate(1).unwrap();
    q.gate("h",&[b[0]],None,&[]).unwrap();q.gate("cx",&[b[0],reference[0]],None,&[]).unwrap();q.gate("x",&x,None,&[]).unwrap();
    let mut bindings=QuantumBindings::default();for(n,w)in[("t",t.clone()),("y",y),("z",z),("x",x)]{bindings.bundles.insert(n.into(),w);}bindings.borrowed.push(b.clone());
    o.execute_action("a",&mut q,&bindings,false,&[]).unwrap();
    // The output is t=1 tensor the original Bell pair on borrowed b/reference.
    q.gate("x",&t,None,&[]).unwrap();q.gate("cx",&[b[0],reference[0]],None,&[]).unwrap();q.gate("h",&b,None,&[]).unwrap();q.release(&b).unwrap();q.release(&reference).unwrap();
}
