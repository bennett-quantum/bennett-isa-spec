//! K82's lexical wire interface, independent of the caller's bindings.
use super::*;
struct Scan<'a> {recipes:&'a BTreeMap<String,Json>,cache:BTreeMap<String,Vec<String>>,active:BTreeSet<String>}
impl Scan<'_> {
 fn recipe(&mut self,name:&str)->Result<Vec<String>,String> {
  if let Some(found)=self.cache.get(name) {return Ok(found.clone());}
  let recipe=self.recipes.get(name).ok_or("unknown interface recipe")?.clone();
  let ps=a(f(&recipe,"parameters")?)?;
  if !ps.iter().all(|p|matches!(p,Json::Str(_))) {return Ok(Vec::new());}
  if !self.active.insert(name.into()) {return Err("cyclic named interface".into());}
  let mut bound=ps.iter().map(|p|s(p).map(str::to_string)).collect::<Result<BTreeSet<_>,_>>()?;
  let mut names=Vec::new();self.body(a(f(&recipe,"body")?)?,&mut bound,&mut names)?;
  self.active.remove(name);self.cache.insert(name.into(),names.clone());Ok(names)
 }
 fn wire(j:&Json,bound:&BTreeSet<String>,names:&mut Vec<String>)->Result<(),String> {
  match j {
   Json::Arr(items)=>for item in items {Self::wire(item,bound,names)?;},
   Json::Str(text)=>{let base=text.split_once('[').map_or(text.as_str(),|p|p.0);if !bound.contains(base)&&!names.iter().any(|n|n==base) {names.push(base.into());}},
   Json::Obj(_)=>Self::wire(f(j,"range")?,bound,names)?,
   _=>return Err("invalid interface wire actual".into()),
  } Ok(())
 }
 fn actual(&mut self,formal:&Json,actual:&Json,bound:&BTreeSet<String>,names:&mut Vec<String>)->Result<(),String> {
  let Some(ty)=formal.get("type").and_then(Json::as_str) else{return Ok(())};
  match ty {
   "wire"|"environment"=>Self::wire(actual,bound,names)?,
   "region"=>if let Some(cells)=actual.get("cells") {Self::wire(cells,bound,names)?;},
   "body"=>if let Some(body)=actual.as_arr() {self.body(body,&mut bound.clone(),names)?;},
   "templates"=>if let Some(templates)=actual.as_arr() {for template in templates {let mut local=bound.clone();for p in a(f(template,"parameters")?)? {local.insert(if let Some(n)=p.as_str() {n.to_string()}else{s(f(p,"name")?)?.to_string()});}self.body(a(f(template,"body")?)?,&mut local,names)?;}},
   "records"=>if let Some(records)=actual.as_arr() {for record in records {for field in a(f(formal,"fields")?)? {let name=s(f(field,"name")?)?;if let Some(value)=record.get(name) {self.actual(field,value,bound,names)?;}}}},
   _=>{},
  } Ok(())
 }
 fn body(&mut self,body:&[Json],bound:&mut BTreeSet<String>,names:&mut Vec<String>)->Result<(),String> {
  for node in body {
   if let Some(wires)=node.get("wires") {Self::wire(wires,bound,names)?;}
   if let Some(controls)=node.get("under") {Self::wire(controls,bound,names)?;}
   if let Some(name)=node.get("release") {Self::wire(name,bound,names)?;}
   if let Some(name)=node.get("alloc") {bound.insert(s(name)?.into());}
   if let Some(kind)=node.get("invoke") {
    let kind=s(kind)?;let callee=self.recipes.get(kind).ok_or("unknown interface dependency")?.clone();
    for (formal,actual) in a(f(&callee,"parameters")?)?.iter().zip(a(f(node,"args")?)?) {self.actual(formal,actual,bound,names)?;}
    for name in self.recipe(kind)? {Self::wire(&Json::Str(name),bound,names)?;}
   }
   if let Some(input)=node.get("in") {if node.get("each").is_some() {Self::wire(input,bound,names)?;}}
   if let Some(inner)=node.get("body") {
    let binding=node.get("declare").or_else(||node.get("each")).or_else(||node.get("for"));
    let temporary=binding.map(s).transpose()?.map(str::to_string);
    let old=temporary.as_ref().is_some_and(|n|bound.contains(n));
    if let Some(n)=&temporary {bound.insert(n.clone());}
    if node.get("visit").is_some() {self.body(a(inner)?,&mut bound.clone(),names)?;}else{self.body(a(inner)?,bound,names)?;}
    if let Some(n)=&temporary {if !old {bound.remove(n);}}
   }
  } Ok(())
 }
}
pub(super) fn derive(recipes:&BTreeMap<String,Json>)->Result<BTreeMap<String,Vec<String>>,String> {
 let mut scan=Scan{recipes,cache:BTreeMap::new(),active:BTreeSet::new()};
 for (name,recipe) in recipes {if a(f(recipe,"parameters")?)?.iter().all(|p|matches!(p,Json::Str(_))) {scan.recipe(name)?;}}
 Ok(scan.cache)
}
