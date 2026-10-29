mod support;
use bennett_isa_ref::{BitVector,Budget,Object,OracleVectorInput};
fn object(body:&str)->Object{
    let signature=r#"{"abi":"0.8","args":[],"fault":4,"in":[257,257],"kind":"oracle","mem":[],"tgt":257,"tier":"q"}"#;
    Object::parse(&support::object(&[("wide",signature,&format!("oracle @wide (x:wire[257] in, y:wire[257] in) (t:wire[257]) (f:wire[4]) () ()\nregs {{ r:reg[257] }}\n^entry():\n{body}\nlift.retval r"))],"{}","")).unwrap()
}
#[test]
fn arbitrary_width_oracle_arithmetic_and_total_faults(){
    // x=2^200+3, y=2^100+5. Quotient=2^100-5, remainder=28.
    let x=BitVector::parse_decimal(257,"1606938044258990275541962092341162602522202993782792835301379").unwrap();
    let y=BitVector::parse_decimal(257,"1267650600228229401496703205381").unwrap();
    let input=OracleVectorInput{ins:vec![x.clone(),y.clone()],..Default::default()};
    for (op,expected)in [("add",x.binary("add",&y).unwrap()),("udiv",BitVector::parse_decimal(257,"1267650600228229401496703205371").unwrap()),("urem",BitVector::from_u128(257,28).unwrap())]{
        let o=object(&format!("r = lift.{op} x y"));let r=o.run_oracle_vectors("wide",&input,Budget::steps(2)).unwrap();assert_eq!(r.payload,expected);assert!(r.code.is_zero());assert_eq!(r.steps,2);
        let r=o.run_oracle_vectors("wide",&input,Budget::steps(1)).unwrap();assert!(r.payload.is_zero());assert_eq!(r.code.to_u128().unwrap(),1);
    }
    let o=object("r = lift.udiv x y");let input=OracleVectorInput{ins:vec![x,BitVector::zero(257).unwrap()],..Default::default()};let r=o.run_oracle_vectors("wide",&input,Budget::steps(2)).unwrap();assert_eq!((r.code.to_u128().unwrap(),r.steps),(5,1));assert!(r.payload.is_zero());
}
