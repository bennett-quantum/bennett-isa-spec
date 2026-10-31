use bennett_isa_ref::{Object,ClassicalMachine};
fn main(){
 let args:Vec<_>=std::env::args().collect(); let text=std::fs::read_to_string(&args[2]).unwrap(); let o=Object::parse(&text).unwrap(); o.check_hashes().unwrap();
 if args[1]=="claim" {let h=o.identity().unwrap().closure_hashes["a"].clone().unwrap();println!("claim={:?}; certificate={:?}",o.verify_claim(&h,0),o.verify_certificate()); return;}
 let mut m=ClassicalMachine::new(&o,"f",&[],&[]).unwrap();let entry=m.snapshot();m.run(3).unwrap();let len=m.outcome().request.unwrap().payload.len();
 println!("request {len}");let reply=vec![0xa5;len];m.reply(&reply).unwrap();drop(reply);println!("reply done");
 m.step_once().unwrap(); let prefix=m.snapshot(); let result=m.step_once();println!("store result={result:?}; prefix_equal={}; difference={:?}",m.snapshot()==prefix,prefix.first_difference(&m.snapshot()));
 drop(prefix);println!("retry={:?}; byte={:?}; outcome={:?}",m.step_once(),m.memory_bytes(1,1),m.outcome());let reverse=m.reverse(); println!("reverse={reverse:?}; entry_equal={}; difference={:?}", m.snapshot()==entry,entry.first_difference(&m.snapshot()));
}