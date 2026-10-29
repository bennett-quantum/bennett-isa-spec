//! Minimal JSON (std only): parse, and compare an expected document
//! as a *subset* of an actual one. Objects match field-wise with
//! extra actual fields ignored (the spec defines outcome fields; an
//! implementation may report more). Arrays match exactly in length
//! and element-wise. Numbers compare as `f64` values (integers above
//! 2^53-1 travel as decimal strings in this CLI, spec §9.1, so they
//! compare as strings instead).

use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    /// Every JSON number; equality is exact `f64` equality after
    /// parsing, so a shortest-round-trip decimal in a case file
    /// matches the identical binary64 a binary prints.
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(BTreeMap<String, Json>),
}

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(m) => m.get(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_arr(&self) -> Option<&[Json]> {
        match self {
            Json::Arr(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Look up a dotted path (`a.b.0.c`) through objects and arrays.
    pub fn at_path(&self, path: &str) -> Option<&Json> {
        let mut cur = self;
        for part in path.split('.') {
            cur = match cur {
                Json::Obj(m) => m.get(part)?,
                Json::Arr(a) => a.get(part.parse::<usize>().ok()?)?,
                _ => return None,
            };
        }
        Some(cur)
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Json::Null => write!(f, "null"),
            Json::Bool(b) => write!(f, "{b}"),
            Json::Num(n) => write!(f, "{n}"),
            Json::Str(s) => write!(f, "{s:?}"),
            Json::Arr(a) => {
                write!(f, "[")?;
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Json::Obj(m) => {
                write!(f, "{{")?;
                for (i, (k, v)) in m.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{k:?}:{v}")?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// First difference between `expected` (subset) and `actual`, as a
/// dotted path plus both values; `None` when `expected` is a subset.
pub fn subset_diff(expected: &Json, actual: &Json) -> Option<String> {
    diff_at(expected, actual, "$")
}

/// First difference under *exact* comparison (no extra object keys
/// allowed anywhere in the subtree); for fields like a quantum
/// `state`, where an extra component is a wrong answer, not extra
/// reporting.
pub fn exact_diff(expected: &Json, actual: &Json) -> Option<String> {
    if expected == actual {
        None
    } else {
        Some(format!("expected exactly {expected}, got {actual}"))
    }
}

fn diff_at(expected: &Json, actual: &Json, path: &str) -> Option<String> {
    match (expected, actual) {
        (Json::Obj(e), Json::Obj(a)) => {
            for (k, ev) in e {
                match a.get(k) {
                    None => return Some(format!("{path}.{k}: expected {ev}, field missing")),
                    Some(av) => {
                        if let Some(d) = diff_at(ev, av, &format!("{path}.{k}")) {
                            return Some(d);
                        }
                    }
                }
            }
            None
        }
        (Json::Arr(e), Json::Arr(a)) => {
            if e.len() != a.len() {
                return Some(format!(
                    "{path}: expected array of {} elements, got {}",
                    e.len(),
                    a.len()
                ));
            }
            for (i, (ev, av)) in e.iter().zip(a).enumerate() {
                if let Some(d) = diff_at(ev, av, &format!("{path}.{i}")) {
                    return Some(d);
                }
            }
            None
        }
        (Json::Num(e), Json::Num(a)) => {
            if e == a {
                None
            } else {
                Some(format!("{path}: expected {e}, got {a}"))
            }
        }
        _ => {
            if expected == actual {
                None
            } else {
                Some(format!("{path}: expected {expected}, got {actual}"))
            }
        }
    }
}

pub fn parse(text: &str) -> Result<Json, String> {
    let mut p = Parser {
        bytes: text.as_bytes(),
        pos: 0,
    };
    p.ws();
    let v = p.value()?;
    p.ws();
    if p.pos != p.bytes.len() {
        return Err(format!("trailing input at byte {}", p.pos));
    }
    Ok(v)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.pos < self.bytes.len()
            && matches!(self.bytes[self.pos], b' ' | b'\t' | b'\n' | b'\r')
        {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Result<u8, String> {
        self.bytes
            .get(self.pos)
            .copied()
            .ok_or_else(|| "unexpected end of input".to_string())
    }

    fn value(&mut self) -> Result<Json, String> {
        match self.peek()? {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => Ok(Json::Str(self.string()?)),
            b't' => self.literal("true", Json::Bool(true)),
            b'f' => self.literal("false", Json::Bool(false)),
            b'n' => self.literal("null", Json::Null),
            b'-' | b'0'..=b'9' => self.number(),
            c => Err(format!(
                "unexpected character {:?} at byte {}",
                c as char, self.pos
            )),
        }
    }

    fn literal(&mut self, word: &str, v: Json) -> Result<Json, String> {
        if self.bytes[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(v)
        } else {
            Err(format!("bad literal at byte {}", self.pos))
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.pos += 1;
        let mut map = BTreeMap::new();
        self.ws();
        if self.peek()? == b'}' {
            self.pos += 1;
            return Ok(Json::Obj(map));
        }
        loop {
            self.ws();
            let key = self.string()?;
            self.ws();
            if self.peek()? != b':' {
                return Err(format!("expected ':' at byte {}", self.pos));
            }
            self.pos += 1;
            self.ws();
            let val = self.value()?;
            if map.insert(key.clone(), val).is_some() {
                return Err(format!("duplicate key {key:?}"));
            }
            self.ws();
            match self.peek()? {
                b',' => self.pos += 1,
                b'}' => {
                    self.pos += 1;
                    return Ok(Json::Obj(map));
                }
                _ => return Err(format!("expected ',' or '}}' at byte {}", self.pos)),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.pos += 1;
        let mut items = Vec::new();
        self.ws();
        if self.peek()? == b']' {
            self.pos += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            self.ws();
            items.push(self.value()?);
            self.ws();
            match self.peek()? {
                b',' => self.pos += 1,
                b']' => {
                    self.pos += 1;
                    return Ok(Json::Arr(items));
                }
                _ => return Err(format!("expected ',' or ']' at byte {}", self.pos)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        if self.peek()? != b'"' {
            return Err(format!("expected string at byte {}", self.pos));
        }
        self.pos += 1;
        let mut out = String::new();
        loop {
            let c = self.peek()?;
            self.pos += 1;
            match c {
                b'"' => return Ok(out),
                b'\\' => {
                    let e = self.peek()?;
                    self.pos += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let cp = self.hex4()?;
                            if (0xD800..0xDC00).contains(&cp) {
                                // High surrogate: a low surrogate must follow.
                                if self.bytes[self.pos..].starts_with(b"\\u") {
                                    self.pos += 2;
                                    let lo = self.hex4()?;
                                    if !(0xDC00..=0xDFFF).contains(&lo) {
                                        return Err("bad low surrogate".into());
                                    }
                                    let c = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                    out.push(char::from_u32(c).ok_or("bad surrogate pair")?);
                                } else {
                                    return Err("lone high surrogate".into());
                                }
                            } else {
                                out.push(char::from_u32(cp).ok_or("bad \\u escape")?);
                            }
                        }
                        _ => return Err(format!("bad escape at byte {}", self.pos)),
                    }
                }
                0..=0x1f => return Err("unescaped control character in string".into()),
                _ => {
                    // Copy one UTF-8 scalar starting at the previous byte.
                    let start = self.pos - 1;
                    let rest = std::str::from_utf8(&self.bytes[start..])
                        .map_err(|_| "invalid UTF-8 in string".to_string())?;
                    let ch = rest.chars().next().ok_or("empty string remainder")?;
                    self.pos = start + ch.len_utf8();
                    out.push(ch);
                }
            }
        }
    }

    fn hex4(&mut self) -> Result<u32, String> {
        if self.pos + 4 > self.bytes.len() {
            return Err("short \\u escape".into());
        }
        let s = std::str::from_utf8(&self.bytes[self.pos..self.pos + 4])
            .map_err(|_| "bad \\u escape".to_string())?;
        let v = u32::from_str_radix(s, 16).map_err(|_| "bad \\u escape".to_string())?;
        self.pos += 4;
        Ok(v)
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.pos;
        if self.peek()? == b'-' {
            self.pos += 1;
        }
        match self.peek()? {
            b'0' => self.pos += 1,
            b'1'..=b'9' => self.digits()?,
            _ => return Err(format!("expected integer at byte {}", self.pos)),
        }
        if self.bytes.get(self.pos) == Some(&b'.') {
            self.pos += 1;
            self.digits()?;
        }
        if matches!(self.bytes.get(self.pos), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.bytes.get(self.pos), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            self.digits()?;
        }
        let s = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| "bad number".to_string())?;
        let n = s.parse::<f64>().map_err(|_| format!("bad number {s:?}"))?;
        if !n.is_finite() {
            return Err(format!("number outside finite binary64 range: {s:?}"));
        }
        Ok(Json::Num(n))
    }

    fn digits(&mut self) -> Result<(), String> {
        let start = self.pos;
        while matches!(self.bytes.get(self.pos), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(format!("expected digit at byte {}", self.pos));
        }
        Ok(())
    }
}
