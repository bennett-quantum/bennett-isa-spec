//! Strict JSON and RFC 8785 canonical serialization, without dependencies.
use crate::{err, Result};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Json {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    pub fn parse(text: &str) -> Result<Self> {
        let mut p = Parser { text, pos: 0, depth: 0 };
        let value = p.value()?;
        p.ws();
        if p.pos != text.len() { return err("§16.1", "trailing JSON input"); }
        Ok(value)
    }
    pub fn canonical(&self) -> String {
        match self {
            Self::Null => "null".into(),
            Self::Bool(b) => b.to_string(),
            Self::Number(n) => n.clone(),
            Self::String(s) => quote(s),
            Self::Array(a) => format!("[{}]", a.iter().map(Self::canonical).collect::<Vec<_>>().join(",")),
            Self::Object(o) => {
                let mut entries: Vec<_> = o.iter().collect();
                entries.sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
                format!("{{{}}}", entries.into_iter().map(|(k,v)| format!("{}:{}", quote(k),v.canonical())).collect::<Vec<_>>().join(","))
            }
        }
    }
    pub fn object(&self) -> Result<&BTreeMap<String, Json>> {
        if let Self::Object(o) = self { Ok(o) } else { err("§9.1", "expected JSON object") }
    }
    pub fn array(&self) -> Result<&[Json]> {
        if let Self::Array(a) = self { Ok(a) } else { err("§9.1", "expected JSON array") }
    }
    pub fn string(&self) -> Result<&str> {
        if let Self::String(s) = self { Ok(s) } else { err("§9.1", "expected JSON string") }
    }
    pub fn get(&self, key: &str) -> Result<&Json> {
        self.object()?.get(key).ok_or_else(|| crate::Error(format!("§9.1: missing field {key}")))
    }
    /// Metadata integers may be JSON numbers or decimal strings (§9.1).
    pub fn uint(&self) -> Result<u128> {
        let s = match self { Self::Number(s) | Self::String(s) => s, _ => return err("§9.1", "expected unsigned integer") };
        s.parse().map_err(|_| crate::Error(format!("§9.1: invalid unsigned integer {s}")))
    }
    pub fn fields(&self, required: &[&str], optional: &[&str]) -> Result<()> {
        let obj = self.object()?;
        for key in required { self.get(key)?; }
        for key in obj.keys() {
            if !required.contains(&key.as_str()) && !optional.contains(&key.as_str()) {
                return err("§9.1", format!("unknown field {key}"));
            }
        }
        Ok(())
    }
}

pub fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""), '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"), '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"), '\u{c}' => out.push_str("\\f"), '\r' => out.push_str("\\r"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"'); out
}

fn canonical_number(raw: &str) -> Result<String> {
    let n: f64 = raw.parse().map_err(|_| crate::Error("§16.1: invalid JSON number".into()))?;
    if !n.is_finite() { return err("§16.1", "non-finite JSON number"); }
    if n == 0.0 { return Ok("0".into()); }
    let negative = n < 0.0;
    let display = n.abs().to_string();
    let (mantissa, exponent) = match display.split_once('e') {
        Some((m,e)) => (m, e.parse::<i32>().expect("Rust float exponent")),
        None => (display.as_str(), 0),
    };
    let dot = mantissa.find('.').unwrap_or(mantissa.len()) as i32;
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let leading = digits.bytes().take_while(|b| *b == b'0').count();
    let position = dot + exponent - leading as i32;
    let digits = digits[leading..].trim_end_matches('0');
    let mut out = if negative { "-".to_string() } else { String::new() };
    if position > 0 && position <= 21 {
        if position as usize >= digits.len() {
            out.push_str(digits); out.push_str(&"0".repeat(position as usize - digits.len()));
        } else {
            let (a,b) = digits.split_at(position as usize); out.push_str(a); out.push('.'); out.push_str(b);
        }
    } else if position > -6 && position <= 0 {
        out.push_str("0."); out.push_str(&"0".repeat((-position) as usize)); out.push_str(digits);
    } else {
        out.push_str(&digits[..1]);
        if digits.len() > 1 { out.push('.'); out.push_str(&digits[1..]); }
        out.push('e'); let e = position - 1;
        if e >= 0 { out.push('+'); } out.push_str(&e.to_string());
    }
    Ok(out)
}

struct Parser<'a> { text: &'a str, pos: usize, depth: usize }
impl Parser<'_> {
    fn peek(&self) -> Option<u8> { self.text.as_bytes().get(self.pos).copied() }
    fn ws(&mut self) { while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) { self.pos += 1; } }
    fn take(&mut self, b: u8) -> bool { if self.peek() == Some(b) { self.pos+=1; true } else { false } }
    fn expect(&mut self, b: u8) -> Result<()> { if self.take(b) { Ok(()) } else { err("§16.1", format!("expected JSON '{}' at byte {}", b as char, self.pos)) } }
    fn value(&mut self) -> Result<Json> {
        self.ws(); self.depth += 1;
        if self.depth > 256 { return err("§16.1", "JSON nesting exceeds 256"); }
        let value = match self.peek() {
            Some(b'"') => Json::String(self.string()?),
            Some(b'{') => {
                self.pos+=1; self.ws(); let mut o=BTreeMap::new();
                if !self.take(b'}') { loop {
                    self.ws(); let key=self.string()?; self.ws(); self.expect(b':')?;
                    let v=self.value()?;
                    if o.insert(key.clone(), v).is_some() { return err("§16.1", format!("duplicate JSON key {key}")); }
                    self.ws(); if self.take(b'}') { break; } self.expect(b',')?;
                } } Json::Object(o)
            }
            Some(b'[') => {
                self.pos+=1; self.ws(); let mut a=Vec::new();
                if !self.take(b']') { loop {
                    a.push(self.value()?); self.ws(); if self.take(b']') { break; } self.expect(b',')?;
                } } Json::Array(a)
            }
            Some(b'-' | b'0'..=b'9') => {
                let start=self.pos; self.take(b'-');
                if !self.take(b'0') {
                    if !matches!(self.peek(), Some(b'1'..=b'9')) { return err("§16.1", "invalid JSON integer"); }
                    while matches!(self.peek(), Some(b'0'..=b'9')) { self.pos+=1; }
                }
                if self.take(b'.') { let p=self.pos; while matches!(self.peek(),Some(b'0'..=b'9')) { self.pos+=1; } if p==self.pos { return err("§16.1","empty fraction"); } }
                if self.take(b'e') || self.take(b'E') {
                    if !self.take(b'+') { self.take(b'-'); }
                    let p=self.pos; while matches!(self.peek(),Some(b'0'..=b'9')) { self.pos+=1; } if p==self.pos { return err("§16.1","empty exponent"); }
                }
                Json::Number(canonical_number(&self.text[start..self.pos])?)
            }
            _ => {
                let mut found=None;
                for (s,v) in [("null",Json::Null),("true",Json::Bool(true)),("false",Json::Bool(false))] {
                    if self.text[self.pos..].starts_with(s) { self.pos+=s.len(); found=Some(v); break; }
                }
                found.ok_or_else(|| crate::Error(format!("§16.1: invalid JSON at byte {}",self.pos)))?
            }
        };
        self.depth-=1; Ok(value)
    }
    fn hex4(&mut self) -> Result<u16> {
        let end=self.pos+4;
        let s=self.text.as_bytes().get(self.pos..end).ok_or_else(|| crate::Error("§16.1: short Unicode escape".into()))?;
        let mut n=0;
        for b in s { n=n*16+match b { b'0'..=b'9'=>(b-b'0') as u16,b'a'..=b'f'=>(b-b'a'+10) as u16,b'A'..=b'F'=>(b-b'A'+10) as u16,_=>return err("§16.1","invalid Unicode escape") }; }
        self.pos=end; Ok(n)
    }
    fn string(&mut self) -> Result<String> {
        self.expect(b'"')?; let mut out=String::new();
        loop { match self.peek() {
            None => return err("§16.1","unterminated JSON string"),
            Some(b'"') => { self.pos+=1; return Ok(out); }
            Some(b'\\') => {
                self.pos+=1; let b=self.peek().ok_or_else(||crate::Error("§16.1: short escape".into()))?; self.pos+=1;
                out.push(match b {
                    b'"'=>'"', b'\\'=>'\\',b'/'=>'/',b'b'=>'\u{8}',b'f'=>'\u{c}',b'n'=>'\n',b'r'=>'\r',b't'=>'\t',
                    b'u'=> { let hi=self.hex4()?; let n=if (0xd800..=0xdbff).contains(&hi) {
                        self.expect(b'\\')?; self.expect(b'u')?; let lo=self.hex4()?;
                        if !(0xdc00..=0xdfff).contains(&lo) { return err("§16.1","invalid low surrogate"); }
                        0x10000+(((hi as u32-0xd800)<<10)|(lo as u32-0xdc00))
                    } else { hi as u32 }; char::from_u32(n).ok_or_else(||crate::Error("§16.1: lone surrogate".into()))? },
                    _=>return err("§16.1","invalid JSON escape"),
                });
            }
            Some(0..=31)=>return err("§16.1","unescaped JSON control character"),
            _=> { let c=self.text[self.pos..].chars().next().expect("UTF-8 char"); self.pos+=c.len_utf8(); out.push(c); }
        } }
    }
}
