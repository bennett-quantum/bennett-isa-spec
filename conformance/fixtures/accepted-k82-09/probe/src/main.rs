use bennett_isa_conformance::{json::{self, Json}, ref_language::{Expander, Leaf, Registry}};
use std::collections::BTreeMap;
fn sha(s:&str)->String{bennett_isa_conformance::ref_hash::sha256(s.as_bytes())}
fn norm(leaves:&[Leaf])->Vec<(String,Vec<String>)>{
    let mut ids=BTreeMap::new();
    let mut name=|n:&str|->String{ if n.starts_with('@'){let k=ids.len();format!("#{}",*ids.entry(n.to_string()).or_insert(k))}else{n.to_string()}};
    leaves.iter().map(|l|{
        if let Some((n,_,_,_))=&l.allocation{ if l.gate=="alloc"||l.gate=="release"{ return (l.gate.clone(),vec![name(n)]);} }
        (l.gate.clone(),l.wires.iter().map(|(n,i)|format!("{}[{}]",name(n),i)).collect())
    }).collect()
}
fn render(v:&[(String,Vec<String>)])->String{
    let x:Vec<String>=v.iter().map(|(g,w)|{let ws:Vec<String>=w.iter().map(|s|format!("\"{s}\"")).collect();format!("[\"{g}\",[{}]]",ws.join(","))}).collect();
    format!("[{}]",x.join(","))
}
fn main(){
    let t=std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();
    let catalog=std::fs::read_to_string(std::env::args().nth(2).unwrap()).unwrap();
    let v=json::parse(&t).unwrap();
    let (mut ok,mut bad)=(0,0);
    let mut full=Expander::new(&catalog,Registry::new()).unwrap();
    for e in v.get("edges").unwrap().as_arr().unwrap(){
        let caller=e.get("caller").unwrap().as_str().unwrap();
        for c in e.get("instances").unwrap().as_arr().unwrap(){
            let mut bundles:Vec<(String,usize)>=vec![];
            if let Json::Obj(m)=c.get("bundles").unwrap(){for(k,w) in m.iter(){ if let Json::Num(n)=w{bundles.push((k.clone(),*n as usize));}}}
            let b:Vec<(&str,usize)>=bundles.iter().map(|(a,n)|(a.as_str(),*n)).collect();
            let r=full.expand(caller,c.get("args").unwrap().as_arr().unwrap(),&b);
            match r{
                Ok(l)=>{let n=norm(&l);let got=sha(&render(&n));
                    if got==c.get("sha256").unwrap().as_str().unwrap(){ok+=1}else{bad+=1;println!("EDGE DIFF {caller} {} leaves={} want={}",c.get("args").unwrap(),n.len(),c.get("leaves").unwrap());}}
                Err(x)=>{bad+=1;println!("EDGE ERR {caller} {x}");}
            }
        }
    }
    println!("edges: {ok} match, {bad} differ");
    for c in v.get("cases").unwrap().as_arr().unwrap(){
        let id=c.get("id").unwrap().as_str().unwrap();
        let doc=format!(r#"{{"isa":"0.9","name":"ref","version":"0.9","recipes":{}}}"#,c.get("recipes").unwrap());
        let mut bundles:Vec<(String,usize)>=vec![];
        if let Json::Obj(m)=c.get("bundles").unwrap(){for(k,w) in m.iter(){ if let Json::Num(n)=w{bundles.push((k.clone(),*n as usize));}}}
        let b:Vec<(&str,usize)>=bundles.iter().map(|(a,n)|(a.as_str(),*n)).collect();
        let got=Expander::new(&doc,Registry::new()).and_then(|mut e|e.expand(c.get("root").unwrap().as_str().unwrap(),c.get("args").unwrap().as_arr().unwrap(),&b));
        let want=c.get("expect").unwrap();
        let verdict=match(&got,want){
            (Err(x),Json::Str(_))=>format!("PASS rejected: {x}"),
            (Ok(l),Json::Str(_))=>format!("FAIL accepted {}",render(&norm(l))),
            (Err(x),_)=>format!("FAIL rejected: {x}"),
            (Ok(l),w)=>{let g=render(&norm(l));let w=w.to_string().replace(' ',"");if g==w{"PASS".into()}else{format!("FAIL got {g} want {w}")}}
        };
        println!("{id}: {verdict}");
    }
}
