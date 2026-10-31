use bennett_isa_ref::Object;
fn main() {
 let text=std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
 let object=Object::parse(&text).unwrap(); object.check_hashes().unwrap();
 let h=object.identity().unwrap().closure_hashes["a"].clone().unwrap();
 eprintln!("parsed and hash-checked; verifying claim");
 match object.verify_claim(&h,0) { Ok(v)=>println!("{v:?}"), Err(e)=>{eprintln!("{e}");std::process::exit(1)} }
}
