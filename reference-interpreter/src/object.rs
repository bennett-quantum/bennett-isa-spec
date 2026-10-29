use crate::{err, json::Json, Error, Result};
use std::collections::{BTreeMap, BTreeSet};

pub const SECTIONS: [&str; 9] = [
    "symbols",
    "imports",
    "functions",
    "data",
    "devices",
    "layout",
    "schedule",
    "certificate",
    "coverage",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    /// iN, 1 <= N <= 128 (§3.3).
    Int(u16),
    Ptr,
    F64,
    Token,
    Qreg(u64),
    /// wire / wire[N]: no upper bound in §3.3; N is held exactly as u64.
    Wire(u64),
    /// reg[N]: no upper bound in §3.3.
    Reg(u64),
    /// region[A,W]: 2^A cells of wire[W]; neither dimension is bounded by §3.3.
    Region(u64, u64),
}
/// A canonical unsigned INT (§16.1) held exactly in a host u64. A larger INT is
/// well-formed text but names more bits or cells than any host can address;
/// it is reported as a host-resource limit, never silently reduced.
pub(crate) fn host_count(s: &str, what: &str) -> Result<u64> {
    if !integer(s) || s.starts_with('-') {
        return err("§16.1", format!("{what} must be a canonical unsigned INT"));
    }
    s.parse::<u64>().map_err(|_| {
        Error(format!(
            "host limit: {what} {s} exceeds the host's 64-bit descriptor range (not an ISA rule)"
        ))
    })
}
impl Type {
    pub fn parse(s: &str) -> Result<Self> {
        let width = |s: &str| -> Result<u16> {
            let n = host_count(s, "integer width")?;
            if !(1..=128).contains(&n) {
                return err("§3.3", "iN requires 1 <= N <= 128");
            }
            Ok(n as u16)
        };
        let bundle_width = |s: &str| -> Result<u64> {
            let n = host_count(s, "bundle width")?;
            if n == 0 {
                return err("§3.3", "bundle width must be positive");
            }
            Ok(n)
        };
        Ok(match s {
            "ptr" => Self::Ptr,
            "f64" => Self::F64,
            "token" => Self::Token,
            "wire" => Self::Wire(1),
            _ if s.starts_with('i') => Self::Int(width(&s[1..])?),
            _ if s.starts_with("wire[") && s.ends_with(']') => {
                Self::Wire(bundle_width(&s[5..s.len() - 1])?)
            }
            _ if s.starts_with("reg[") && s.ends_with(']') => {
                Self::Reg(bundle_width(&s[4..s.len() - 1])?)
            }
            _ if s.starts_with("qreg[") && s.ends_with(']') => {
                Self::Qreg(bundle_width(&s[5..s.len() - 1])?)
            }
            _ if s.starts_with("region[") && s.ends_with(']') => {
                let (a, w) = s[7..s.len() - 1]
                    .split_once(',')
                    .ok_or_else(|| Error("§16.2: malformed region type".into()))?;
                Self::Region(host_count(a, "region address width")?, bundle_width(w)?)
            }
            _ => return err("§3.3", format!("unknown type {s}")),
        })
    }
    /// The bit width of a bit-vector type; ptr has width ptr_width.
    pub fn width(&self, ptr: u16) -> Result<u64> {
        match self {
            Self::Int(n) => Ok(*n as u64),
            Self::Wire(n) | Self::Reg(n) => Ok(*n),
            Self::Ptr => Ok(ptr as u64),
            _ => err("§3.3", "expected bit-vector type"),
        }
    }
    /// Host index form of a width, for storage allocation.
    pub fn bits(&self, ptr: u16) -> Result<usize> {
        let w = self.width(ptr)?;
        usize::try_from(w).map_err(|_| {
            Error(format!(
                "host limit: width {w} exceeds the host address space (not an ISA rule)"
            ))
        })
    }
    pub fn text(&self) -> String {
        match self {
            Self::Int(n) => format!("i{n}"),
            Self::Wire(n) => format!("wire[{n}]"),
            Self::Reg(n) => format!("reg[{n}]"),
            Self::Region(a, w) => format!("region[{a},{w}]"),
            Self::Qreg(n) => format!("qreg[{n}]"),
            Self::Ptr => "ptr".into(),
            Self::F64 => "f64".into(),
            Self::Token => "token".into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub role: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    pub lhs: Vec<String>,
    pub op: String,
    pub operands: Vec<String>,
    pub bodies: Vec<Vec<Instruction>>,
    pub line: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub name: String,
    pub params: Vec<Param>,
    pub instructions: Vec<Instruction>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    pub kind: String,
    pub name: String,
    pub params: [Vec<Param>; 5],
    pub regs: Vec<Param>,
    pub bounds: Vec<u64>,
    pub blocks: Vec<Block>,
    pub lines: Vec<String>,
}
impl Function {
    /// An imported declaration has the same operand schema as a definition.
    /// Its synthetic parameter names are local to validation, never executable.
    pub(crate) fn from_signature(name: &str, signature: &Json) -> Result<Self> {
        let kind = signature.get("kind")?.string()?;
        let fields: &[&str] = match kind {
            "oracle" => &["abi", "tier", "kind", "in", "tgt", "fault", "mem", "args"],
            "action" | "perm_action" => {
                &["abi", "tier", "kind", "mut", "in", "mem", "args", "bounds"]
            }
            "lproc" => &["abi", "tier", "kind", "params", "result"],
            "cc" | "r" => &["abi", "tier", "kind", "params", "results"],
            _ => return err("§9.1", "unknown function signature kind"),
        };
        signature.fields(fields, &[])?;
        let mut f = Self {
            kind: kind.into(),
            name: name.into(),
            params: Default::default(),
            regs: Vec::new(),
            bounds: Vec::new(),
            blocks: Vec::new(),
            lines: Vec::new(),
        };
        let widths = |key: &str, reg: bool, role: Option<&str>| -> Result<Vec<Param>> {
            signature
                .get(key)?
                .array()?
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let w = v.uint()?;
                    let ty = Type::parse(&format!("{}[{w}]", if reg { "reg" } else { "wire" }))?;
                    Ok(Param {
                        name: format!("{key}_{i}"),
                        ty,
                        role: role.map(str::to_string),
                    })
                })
                .collect()
        };
        let scalar = |key: &str, reg: bool| -> Result<Vec<Param>> {
            let w = signature.get(key)?.uint()?;
            Ok(vec![Param {
                name: key.into(),
                ty: Type::parse(&format!("{}[{w}]", if reg { "reg" } else { "wire" }))?,
                role: None,
            }])
        };
        let types = |key: &str| -> Result<Vec<Param>> {
            signature
                .get(key)?
                .array()?
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    Ok(Param {
                        name: format!("{key}_{i}"),
                        ty: Type::parse(v.string()?)?,
                        role: None,
                    })
                })
                .collect()
        };
        let memory = || -> Result<Vec<Param>> {
            signature
                .get("mem")?
                .array()?
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    v.fields(&["A", "W", "role"], &[])?;
                    let a = v.get("A")?.uint()?;
                    let w = v.get("W")?.uint()?;
                    let role = v.get("role")?.string()?;
                    if !["in", "mut"].contains(&role) {
                        return err("§5.1", "invalid interface region role");
                    }
                    Ok(Param {
                        name: format!("mem_{i}"),
                        ty: Type::parse(&format!("region[{a},{w}]"))?,
                        role: Some(role.into()),
                    })
                })
                .collect()
        };
        match kind {
            "oracle" => {
                f.params[0] = widths("in", false, Some("in"))?;
                f.params[1] = scalar("tgt", false)?;
                f.params[2] = scalar("fault", false)?;
                f.params[3] = memory()?;
                f.params[4] = types("args")?;
            }
            "action" | "perm_action" => {
                f.params[0] = widths("mut", false, Some("mut"))?;
                f.params[1] = widths("in", false, Some("in"))?;
                f.params[2] = memory()?;
                f.params[3] = types("args")?;
                f.bounds = signature
                    .get("bounds")?
                    .array()?
                    .iter()
                    .map(|n| {
                        u64::try_from(n.uint()?).map_err(|_| {
                            Error("§5.3: loop bound exceeds implementation counter".into())
                        })
                    })
                    .collect::<Result<_>>()?;
            }
            "lproc" => {
                f.params[0] = widths("params", true, None)?;
                f.params[1] = scalar("result", true)?;
            }
            _ => {
                f.params[0] = types("params")?;
                f.params[1] = types("results")?;
            }
        }
        validate_header(&f)?;
        if &f.signature() != signature {
            return err(
                "§9.1",
                "invalid signature version, tier, or field representation",
            );
        }
        Ok(f)
    }
    pub fn locations(&self) -> Vec<&Param> {
        let mut out = Vec::new();
        if self.kind == "lproc" {
            out.extend(&self.params[0]);
            out.extend(&self.params[1]);
        }
        out.extend(&self.regs);
        out
    }
    pub fn frame_cells(&self) -> usize {
        3 + self.locations().len()
    }
    pub fn instructions(&self) -> impl Iterator<Item = &Instruction> {
        self.blocks.iter().flat_map(|b| &b.instructions)
    }
    pub fn signature(&self) -> Json {
        let widths = |p: &[Param]| {
            Json::Array(
                p.iter()
                    .map(|p| {
                        Json::Number(match &p.ty {
                            Type::Wire(n) | Type::Reg(n) => n.to_string(),
                            _ => "0".into(),
                        })
                    })
                    .collect(),
            )
        };
        let types =
            |p: &[Param]| Json::Array(p.iter().map(|p| Json::String(p.ty.text())).collect());
        let mem = |p: &[Param]| {
            Json::Array(
                p.iter()
                    .map(|p| {
                        let (a, w) = match p.ty {
                            Type::Region(a, w) => (a, w),
                            _ => (0, 0),
                        };
                        Json::Object(
                            [
                                ("A".into(), Json::Number(a.to_string())),
                                ("W".into(), Json::Number(w.to_string())),
                                (
                                    "role".into(),
                                    Json::String(p.role.clone().unwrap_or_default()),
                                ),
                            ]
                            .into(),
                        )
                    })
                    .collect(),
            )
        };
        let mut m = BTreeMap::from([
            ("abi".into(), Json::String("0.8".into())),
            ("kind".into(), Json::String(self.kind.clone())),
            (
                "tier".into(),
                Json::String(if matches!(self.kind.as_str(), "cc" | "r") {
                    self.kind.clone()
                } else {
                    "q".into()
                }),
            ),
        ]);
        match self.kind.as_str() {
            "oracle" => {
                m.insert("in".into(), widths(&self.params[0]));
                m.insert(
                    "tgt".into(),
                    widths(&self.params[1]).array().expect("array")[0].clone(),
                );
                m.insert(
                    "fault".into(),
                    widths(&self.params[2]).array().expect("array")[0].clone(),
                );
                m.insert("mem".into(), mem(&self.params[3]));
                m.insert("args".into(), types(&self.params[4]));
            }
            "lproc" => {
                m.insert("params".into(), widths(&self.params[0]));
                m.insert(
                    "result".into(),
                    widths(&self.params[1]).array().expect("array")[0].clone(),
                );
            }
            "action" | "perm_action" => {
                m.insert("mut".into(), widths(&self.params[0]));
                m.insert("in".into(), widths(&self.params[1]));
                m.insert("mem".into(), mem(&self.params[2]));
                m.insert("args".into(), types(&self.params[3]));
                m.insert(
                    "bounds".into(),
                    Json::Array(
                        self.bounds
                            .iter()
                            .map(|n| {
                                if *n > 9_007_199_254_740_991 {
                                    Json::String(n.to_string())
                                } else {
                                    Json::Number(n.to_string())
                                }
                            })
                            .collect(),
                    ),
                );
            }
            _ => {
                m.insert("params".into(), types(&self.params[0]));
                m.insert("results".into(), types(&self.params[1]));
            }
        }
        Json::Object(m)
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    pub text: String,
    pub ptr_width: u16,
    pub metadata: BTreeMap<String, Json>,
    pub functions: BTreeMap<String, Function>,
    pub function_order: Vec<String>,
    pub data: BTreeMap<String, Vec<u8>>,
}

pub(crate) fn name(s: &str) -> bool {
    let mut b = s.bytes();
    matches!(b.next(), Some(b'A'..=b'Z' | b'a'..=b'z' | b'_'))
        && b.all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'.')
}
pub(crate) fn integer(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    s == "0" || (!s.starts_with('0') && !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
}
pub(crate) fn literal_name(s: &str) -> Result<String> {
    if !s.starts_with('"') {
        return if name(s) {
            Ok(s.into())
        } else {
            err("§16.1", "expected NAME or STRING literal")
        };
    }
    let mut escaped = false;
    for c in s[1..].chars() {
        if escaped {
            if !['"', '\\', 'n'].contains(&c) {
                return err(
                    "§16.1",
                    "instruction strings admit only quote, backslash and newline escapes",
                );
            }
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        }
    }
    if escaped {
        return err("§16.1", "incomplete STRING escape");
    }
    let value = Json::parse(s)?;
    Ok(value.string()?.into())
}
fn operand_lexical(s: &str) -> Result<()> {
    if name(s) || integer(s) {
        return Ok(());
    }
    if s.starts_with('"') {
        literal_name(s)?;
        return Ok(());
    }
    if s.starts_with('(') {
        list(s)?;
        return Ok(());
    }
    if let Some(n) = s.strip_prefix('@') {
        if name(n) {
            return Ok(());
        }
    }
    if let Some(label) = s.strip_prefix('^') {
        let (n, args) = label
            .split_once('(')
            .map_or((label, None), |(n, args)| (n, Some(args)));
        if name(n) {
            if let Some(args) = args {
                list(&format!("({args}"))?;
            }
            return Ok(());
        }
    }
    if let Some((base, index)) = s.split_once('[') {
        if name(base)
            && index
                .strip_suffix(']')
                .is_some_and(|s| integer(s) && !s.starts_with('-'))
        {
            return Ok(());
        }
    }
    if s.contains(['.', 'e', 'E']) && matches!(s.as_bytes().first(), Some(b'0'..=b'9' | b'-')) {
        let value = Json::parse(s)?;
        if let Json::Number(number) = value {
            let mut canonical = if s.starts_with('-') && s.parse::<f64>() == Ok(0.0) {
                "-0".into()
            } else {
                number
            };
            if !canonical.contains(['.', 'e']) {
                canonical.push_str(".0");
            }
            if canonical == s {
                return Ok(());
            }
        }
    }
    err("§16.1", format!("invalid or noncanonical operand {s}"))
}
pub(crate) fn list(s: &str) -> Result<Vec<String>> {
    let s = s
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| Error(format!("§16.2: expected operand list {s}")))?;
    if s.is_empty() {
        return Ok(Vec::new());
    }
    let v: Vec<_> = s.split(", ").map(str::to_string).collect();
    if v.iter().any(|s| !name(s)) {
        return err(
            "§16.2",
            "operand lists require names separated by comma and space",
        );
    }
    Ok(v)
}
fn params(s: &str) -> Result<Vec<Param>> {
    if s.is_empty() {
        return Ok(Vec::new());
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for p in s.split(", ") {
        let (n, rest) = p
            .split_once(':')
            .ok_or_else(|| Error(format!("§16.2: invalid parameter {p}")))?;
        if !name(n) || !seen.insert(n) {
            return err("§16.2", "invalid or duplicate parameter name");
        }
        let (t, role) = match rest.split_once(' ') {
            Some((t, r)) if matches!(r, "in" | "mut") => (t, Some(r.to_string())),
            Some(_) => return err("§16.2", "invalid parameter role"),
            None => (rest, None),
        };
        out.push(Param {
            name: n.into(),
            ty: Type::parse(t)?,
            role,
        });
    }
    Ok(out)
}

/// Split instruction operands without splitting tuple or string operands.
pub(crate) fn tokens(s: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut nesting = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
            }
            continue;
        }
        match c {
            '"' => quoted = true,
            '(' => nesting += 1,
            ')' => {
                nesting -= 1;
                if nesting < 0 {
                    return err("§16.2", "unmatched close parenthesis");
                }
            }
            ' ' if nesting == 0 => {
                if start == i {
                    return err("§16.1", "multiple spaces between tokens");
                }
                out.push(s[start..i].into());
                start = i + 1;
            }
            _ => {}
        }
    }
    if quoted || nesting != 0 {
        return err("§16.2", "unterminated operand");
    }
    if start < s.len() {
        out.push(s[start..].into());
    } else if !s.is_empty() {
        return err("§16.1", "trailing space");
    }
    Ok(out)
}
fn instruction(lines: &[(usize, &str)], pos: &mut usize) -> Result<Instruction> {
    let (line, raw) = lines[*pos];
    *pos += 1;
    let (lhs, rhs) = if let Some((a, b)) = raw.split_once(" = ") {
        let lhs = if a.starts_with('(') {
            list(a)?
        } else if name(a) {
            vec![a.into()]
        } else {
            return err("§16.2", "invalid result list");
        };
        (lhs, b)
    } else {
        (Vec::new(), raw)
    };
    let ts = tokens(rhs)?;
    if ts.is_empty() {
        return err("§16.2", "empty instruction");
    }
    if !name(&ts[0]) {
        return err("§16.1", "invalid opcode token");
    }
    for (index, operand) in ts.iter().enumerate().skip(1) {
        if operand == "{" && index + 1 == ts.len() {
            continue;
        }
        operand_lexical(operand)?;
    }
    let mut ins = Instruction {
        lhs,
        op: ts[0].clone(),
        operands: ts[1..].to_vec(),
        bodies: Vec::new(),
        line,
    };
    if ins.operands.last().is_some_and(|s| s == "{") {
        ins.operands.pop();
        loop {
            let mut body = Vec::new();
            while *pos < lines.len() && !lines[*pos].1.starts_with('}') {
                body.push(instruction(lines, pos)?);
            }
            if body.is_empty() || *pos == lines.len() {
                return err("§16.2", "empty or unterminated structured body");
            }
            let close = lines[*pos].1;
            *pos += 1;
            ins.bodies.push(body);
            match close {
                "}" => break,
                "} {" if ins.bodies.len() == 1 => {}
                _ => return err("§16.2", "invalid structured body close"),
            }
        }
    }
    Ok(ins)
}

impl Object {
    pub(crate) fn declaration(&self, name: &str) -> Result<Function> {
        if let Some(f) = self.functions.get(name) {
            return Ok(f.clone());
        }
        let import = self
            .section("imports")
            .and_then(|s| s.object().ok())
            .and_then(|s| s.get(name))
            .ok_or_else(|| Error(format!("§9.3: undefined symbol @{name}")))?;
        Function::from_signature(name, import.get("signature")?)
    }
    pub(crate) fn data_size(&self, name: &str) -> Result<u128> {
        if let Some(bytes) = self.data.get(name) {
            return Ok(bytes.len() as u128);
        }
        let import = self
            .section("imports")
            .and_then(|v| v.object().ok())
            .and_then(|v| v.get(name))
            .ok_or_else(|| Error(format!("§9.3: unknown data symbol @{name}")))?;
        let sig = import.get("signature")?;
        if sig.get("kind")?.string()? != "data" {
            return err("§4.6", "qregion image reference must have data kind");
        }
        sig.get("size")?.uint()
    }
    pub(crate) fn ensure_linked(&self) -> Result<()> {
        if self
            .section("imports")
            .map(|s| s.object().map(|s| !s.is_empty()))
            .transpose()?
            .unwrap_or(false)
        {
            return err(
                "§9.3",
                "object has unresolved imports; link before execution",
            );
        }
        Ok(())
    }
    pub fn parse(text: &str) -> Result<Self> {
        if !text.ends_with('\n') || text.contains('\r') || text.contains('\t') {
            return err(
                "§16.1",
                "object requires LF lines, no tabs or trailing spaces, and final LF",
            );
        }
        let mut data_section = false;
        for line in text.lines() {
            if let Some(section) = line.strip_prefix("section ") {
                data_section = section == "data";
            }
            if line.ends_with(' ') && !(data_section && line.strip_suffix(' ').is_some_and(name)) {
                return err("§16.1", "trailing space outside an empty HEX data line");
            }
        }
        let lines: Vec<_> = text.lines().collect();
        let header = tokens(lines.first().copied().unwrap_or(""))?;
        if header.len() != 9 || header[0] != "bennett-object" {
            return err("§16.2", "malformed object header");
        }
        for (i, k) in [
            "isa", "object", "launch", "sched", "cert", "coverage", "target",
        ]
        .iter()
        .enumerate()
        {
            if header[i + 1] != format!("{k}=0.8") {
                return err("§1", format!("expected {k}=0.8"));
            }
        }
        let raw = header[8]
            .strip_prefix("ptr_width=")
            .ok_or_else(|| Error("§16.2: header requires ptr_width=INT".into()))?;
        let ptr_width = host_count(raw, "ptr_width")?;
        // SPEC-QUESTIONS 14: hostcall's rlen is iN with N = ptr_width (§4.8), and
        // iN exists only for 1 <= N <= 128 (§3.3).
        if !(1..=128).contains(&ptr_width) {
            return err(
                "§3.3",
                "ptr_width must satisfy 1 <= ptr_width <= 128 (iN rlen, §4.8)",
            );
        }
        let ptr_width = ptr_width as u16;
        let mut obj = Object {
            text: text.into(),
            ptr_width,
            metadata: BTreeMap::new(),
            functions: BTreeMap::new(),
            function_order: Vec::new(),
            data: BTreeMap::new(),
        };
        let mut pos = 1;
        let mut previous = None;
        while pos < lines.len() {
            if !lines[pos].is_empty()
                || pos + 1 >= lines.len()
                || !lines[pos + 1].starts_with("section ")
            {
                return err(
                    "§16.1",
                    format!(
                        "exactly one blank required before section at line {}",
                        pos + 1
                    ),
                );
            }
            pos += 1;
            let section = &lines[pos][8..];
            pos += 1;
            let order = SECTIONS
                .iter()
                .position(|s| *s == section)
                .ok_or_else(|| Error(format!("§9.1: unknown section {section}")))?;
            if previous.is_some_and(|n| n >= order) {
                return err("§9.1", "duplicate or out-of-order section");
            }
            previous = Some(order);
            let start = pos;
            while pos < lines.len() && !lines[pos].is_empty() {
                if lines[pos].starts_with("section ") {
                    return err("§16.1", "missing blank line before section");
                }
                pos += 1;
            }
            if pos < lines.len()
                && (pos + 1 == lines.len() || !lines[pos + 1].starts_with("section "))
            {
                return err("§16.1", "blank line outside section boundary");
            }
            match section {
                "functions" => obj.parse_functions(&lines[start..pos], start + 1)?,
                "data" => {
                    for line in &lines[start..pos] {
                        let t = if let Some(n) = line.strip_suffix(' ').filter(|n| name(n)) {
                            vec![n.to_string(), String::new()]
                        } else {
                            tokens(line)?
                        };
                        // Empty data images have an empty HEX token, represented by NAME SP.
                        if t.len() != 2
                            || !name(&t[0])
                            || t[1].len() % 2 != 0
                            || !t[1]
                                .bytes()
                                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                        {
                            return err("§16.2", "invalid data line");
                        }
                        let bytes = (0..t[1].len())
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&t[1][i..i + 2], 16).expect("hex checked"))
                            .collect();
                        if obj.data.insert(t[0].clone(), bytes).is_some() {
                            return err("§9.1", "duplicate data name");
                        }
                    }
                }
                _ => {
                    if pos - start != 1 {
                        return err(
                            "§9.1",
                            format!("section {section} requires one JSON object line"),
                        );
                    }
                    let v = Json::parse(lines[start])?;
                    v.object()?;
                    if v.canonical() != lines[start] {
                        return err(
                            "§16.1",
                            format!("section {section} is not RFC 8785 canonical"),
                        );
                    }
                    obj.metadata.insert(section.into(), v);
                }
            }
        }
        obj.validate()?;
        Ok(obj)
    }
    fn parse_functions(&mut self, lines: &[&str], base: usize) -> Result<()> {
        let mut pos = 0;
        while pos < lines.len() {
            let start = pos;
            let ts = tokens(lines[pos])?;
            pos += 1;
            let (kind, offset) = if ts.first().is_some_and(|s| s == "fn") {
                (ts.get(1).map(String::as_str).unwrap_or(""), 2)
            } else {
                (ts.first().map(String::as_str).unwrap_or(""), 1)
            };
            if !matches!(
                kind,
                "cc" | "r" | "oracle" | "action" | "perm_action" | "lproc"
            ) || ts.len() != offset + 6
            {
                return err(
                    "§16.2",
                    "invalid function header or number of parameter lists",
                );
            }
            let n = ts[offset]
                .strip_prefix('@')
                .filter(|s| name(s))
                .ok_or_else(|| Error("§16.2: invalid function name".into()))?;
            let mut ps: [Vec<Param>; 5] = Default::default();
            for (i, s) in ts[offset + 1..].iter().enumerate() {
                let inner = s
                    .strip_prefix('(')
                    .and_then(|s| s.strip_suffix(')'))
                    .ok_or_else(|| Error("§16.2: invalid parameter list".into()))?;
                ps[i] = params(inner)?;
            }
            let mut f = Function {
                kind: kind.into(),
                name: n.into(),
                params: ps,
                regs: Vec::new(),
                bounds: Vec::new(),
                blocks: Vec::new(),
                lines: Vec::new(),
            };
            if pos < lines.len() && lines[pos].starts_with("regs ") {
                let body = lines[pos]
                    .strip_prefix("regs { ")
                    .ok_or_else(|| Error("§16.2: malformed regs".into()))?;
                let (rs, rest) = body
                    .split_once(" }")
                    .ok_or_else(|| Error("§16.2: malformed regs close".into()))?;
                f.regs = params(rs)?;
                if !rest.is_empty() {
                    let bs = rest
                        .strip_prefix(" bounds {")
                        .and_then(|s| s.strip_suffix('}'))
                        .ok_or_else(|| Error("§16.2: malformed bounds".into()))?;
                    if !bs.is_empty() {
                        f.bounds = bs
                            .split(", ")
                            .map(|s| host_count(s, "loop bound"))
                            .collect::<Result<_>>()?;
                    }
                }
                pos += 1;
            }
            while pos < lines.len() && lines[pos].starts_with('^') {
                let b = lines[pos];
                pos += 1;
                let (bn, ps) = b[1..]
                    .split_once('(')
                    .ok_or_else(|| Error("§16.2: malformed block".into()))?;
                if !name(bn) {
                    return err("§16.2", "invalid block name");
                }
                let ps = ps
                    .strip_suffix("):")
                    .ok_or_else(|| Error("§16.2: malformed block parameter list".into()))?;
                let mut block = Block {
                    name: bn.into(),
                    params: params(ps)?,
                    instructions: Vec::new(),
                };
                let end = (pos..lines.len())
                    .find(|i| lines[*i].starts_with('^') || is_head(lines[*i]))
                    .unwrap_or(lines.len());
                let numbered: Vec<_> = lines[pos..end]
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (base + pos + i, *s))
                    .collect();
                let mut ip = 0;
                while ip < numbered.len() {
                    block.instructions.push(instruction(&numbered, &mut ip)?);
                }
                if block.instructions.is_empty() {
                    return err("§16.2", "empty block");
                }
                f.blocks.push(block);
                pos = end;
            }
            if f.blocks.is_empty() {
                return err("§16.2", "function requires blocks");
            }
            f.lines = lines[start..pos].iter().map(|s| s.to_string()).collect();
            self.function_order.push(n.into());
            if self.functions.insert(n.into(), f).is_some() {
                return err("§9.1", "duplicate function name");
            }
        }
        Ok(())
    }
    pub fn section(&self, key: &str) -> Option<&Json> {
        self.metadata.get(key)
    }
    pub fn symbols(&self) -> Result<&BTreeMap<String, Json>> {
        self.metadata
            .get("symbols")
            .ok_or_else(|| Error("§9.1: missing symbols section".into()))?
            .object()
    }
    fn validate(&self) -> Result<()> {
        self.validate_symbols()?;
        for f in self.functions.values() {
            self.validate_function(f)?;
        }
        self.validate_layout()?;
        self.validate_metadata()?;
        self.validate_call_graph()?;
        Ok(())
    }
    fn validate_symbols(&self) -> Result<()> {
        let mut names = BTreeSet::new();
        for (index, s) in self.symbols()? {
            if !integer(index) || index.starts_with('-') {
                return err("§9.1", "invalid local symbol index");
            }
            s.fields(&["name", "kind", "signature", "interface_hash"], &[])?;
            let n = s.get("name")?.string()?;
            let k = s.get("kind")?.string()?;
            if !name(n) || !names.insert(n) {
                return err("§9.1", "invalid or duplicate symbol name");
            }
            let h = s.get("interface_hash")?.string()?;
            if h.len() != 64
                || !h
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return err("§9.2", "interface_hash must be 64 lowercase hex digits");
            }
            let sig = s.get("signature")?;
            if k == "data" {
                sig.fields(&["abi", "kind", "size", "tier"], &[])?;
                if sig.get("kind")?.string()? != "data"
                    || sig.get("tier")?.string()? != "-"
                    || sig.get("abi")?.string()? != "0.8"
                {
                    return err("§9.1", "invalid data signature");
                }
                let bytes = self
                    .data
                    .get(n)
                    .ok_or_else(|| Error(format!("§9.1: missing data definition {n}")))?;
                if sig.get("size")?.uint()? != bytes.len() as u128 {
                    return err("§7.2", "data size differs from signature");
                }
            } else {
                let f = self
                    .functions
                    .get(n)
                    .ok_or_else(|| Error(format!("§9.1: missing function definition {n}")))?;
                validate_header(f)?;
                if k != f.kind || sig != &f.signature() {
                    return err("§9.1", format!("signature mismatch for {n}"));
                }
            }
        }
        if self
            .functions
            .keys()
            .chain(self.data.keys())
            .any(|n| !names.contains(n.as_str()))
        {
            return err("§9.1", "definition without symbol");
        }
        if let Some(imports) = self.section("imports") {
            for (n, im) in imports.object()? {
                if !name(n) || names.contains(n.as_str()) {
                    return err(
                        "§9.3",
                        "invalid import name or import conflicts with definition",
                    );
                }
                im.fields(&["signature"], &["pinned_closure_hash"])?;
                let signature = im.get("signature")?;
                if signature.get("kind")?.string()? == "data" {
                    signature.fields(&["abi", "tier", "kind", "size"], &[])?;
                    if signature.get("abi")?.string()? != "0.8"
                        || signature.get("tier")?.string()? != "-"
                    {
                        return err("§9.1", "invalid imported data signature");
                    }
                    signature.get("size")?.uint()?;
                } else {
                    Function::from_signature(n, signature)?;
                }
                if let Some(p) = im.object()?.get("pinned_closure_hash") {
                    let h = p.string()?;
                    if h.len() != 64
                        || !h
                            .bytes()
                            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                    {
                        return err("§9.3", "invalid pinned hash");
                    }
                }
            }
        }
        Ok(())
    }
    fn validate_function(&self, f: &Function) -> Result<()> {
        let lifted = matches!(f.kind.as_str(), "oracle" | "lproc");
        let mut types = BTreeMap::new();
        for p in f
            .params
            .iter()
            .enumerate()
            .filter(|(i, _)| !matches!(f.kind.as_str(), "cc" | "r") || *i == 0)
            .flat_map(|(_, ps)| ps)
            .chain(&f.regs)
        {
            if types.insert(p.name.clone(), p.ty.clone()).is_some() {
                return err("§3.7", "duplicate function location or parameter");
            }
        }
        let mut labels = BTreeSet::new();
        for b in &f.blocks {
            if !labels.insert(&b.name) {
                return err("§3.6", "duplicate block label");
            }
            if lifted && !b.params.is_empty() {
                return err("§3.7", "lifted blocks have no parameters");
            }
        }
        if lifted {
            for b in &f.blocks {
                for (i, ins) in b.instructions.iter().enumerate() {
                    let term = matches!(
                        ins.op.as_str(),
                        "qjump" | "qbranch" | "qjumpr" | "lift.ret" | "lift.retval" | "fault"
                    );
                    if term != (i + 1 == b.instructions.len()) {
                        return err(
                            "§3.6",
                            format!("block {} must end in exactly one terminator", b.name),
                        );
                    }
                    self.validate_row(f, ins, &types, &labels)?;
                }
            }
            self.validate_matched_oracle_calls(f)?;
        } else {
            self.validate_structured(f, &mut types, &labels)?;
        }
        Ok(())
    }
    fn index_space_size(&self, containing: &Function) -> Result<u128> {
        let mut maximum = containing.instructions().count() as u128;
        for oracle in self.functions.values().filter(|f| f.kind == "oracle") {
            let mut pending = vec![oracle.name.clone()];
            let mut seen = BTreeSet::new();
            let mut count = 0u128;
            while let Some(name) = pending.pop() {
                if !seen.insert(name.clone()) {
                    continue;
                }
                let function = self.declaration(&name)?;
                count += function.instructions().count() as u128;
                for call in function.instructions().filter(|i| i.op == "lift.call") {
                    let target = call
                        .operands
                        .first()
                        .and_then(|n| n.strip_prefix('@'))
                        .ok_or_else(|| Error("§4.8: malformed lift.call".into()))?;
                    pending.push(target.to_string());
                }
            }
            if seen.contains(&containing.name) {
                maximum = maximum.max(count);
            }
        }
        Ok(maximum)
    }
    fn fault_width(&self, containing: &Function) -> Result<u64> {
        if containing.kind == "oracle" {
            return containing.params[2][0].ty.width(self.ptr_width);
        }
        let mut width = u64::MAX;
        for oracle in self.functions.values().filter(|f| f.kind == "oracle") {
            let mut pending = vec![oracle.name.clone()];
            let mut seen = BTreeSet::new();
            while let Some(name) = pending.pop() {
                if !seen.insert(name.clone()) {
                    continue;
                }
                let f = self.declaration(&name)?;
                for call in f.instructions().filter(|i| i.op == "lift.call") {
                    let target = call
                        .operands
                        .first()
                        .and_then(|s| s.strip_prefix('@'))
                        .ok_or_else(|| Error("§4.8: malformed lift.call".into()))?;
                    pending.push(target.into());
                }
            }
            if seen.contains(&containing.name) {
                width = width.min(oracle.params[2][0].ty.width(self.ptr_width)?);
            }
        }
        Ok(width)
    }
    fn validate_row(
        &self,
        f: &Function,
        i: &Instruction,
        types: &BTreeMap<String, Type>,
        labels: &BTreeSet<&String>,
    ) -> Result<()> {
        let op = i.op.as_str();
        let args = &i.operands;
        if !i.bodies.is_empty() {
            return err("§4.8", "structured instructions are not lifted rows");
        }
        let rd = |n: &str| -> Result<u64> {
            let p = if f.kind == "oracle" {
                f.regs
                    .iter()
                    .chain(&f.params[0])
                    .chain(&f.params[4])
                    .find(|p| p.name == n)
            } else {
                f.params[0]
                    .iter()
                    .chain(&f.params[1])
                    .chain(&f.regs)
                    .find(|p| p.name == n)
            };
            p.ok_or_else(|| Error(format!("§3.7: unreadable location {n}")))?
                .ty
                .width(self.ptr_width)
        };
        let dest = |n: &str| -> Result<u64> {
            if !f.locations().iter().any(|p| p.name == n) {
                return err("§3.7", format!("not a writable location: {n}"));
            }
            types[n].width(self.ptr_width)
        };
        let arity = |n: usize, lhs: usize| -> Result<()> {
            if args.len() != n || i.lhs.len() != lhs {
                err(
                    "§4.8",
                    format!(
                        "{} expects {lhs} results and {n} operands at line {}",
                        op, i.line
                    ),
                )
            } else {
                Ok(())
            }
        };
        let eq = |a: u64, b: u64| -> Result<()> {
            if a != b {
                err("§3.3", format!("width mismatch in {op}: {a} != {b}"))
            } else {
                Ok(())
            }
        };
        let label = |s: &str| -> Result<()> {
            if s.strip_prefix('^')
                .is_none_or(|s| !labels.iter().any(|x| x.as_str() == s))
            {
                err("§3.6", format!("unknown label {s}"))
            } else {
                Ok(())
            }
        };
        let addr = |s: &str| -> Result<()> { eq(rd(s)?, self.ptr_width as u64) };
        let length = |s: &str| -> Result<()> {
            if integer(s) {
                let n = s
                    .parse::<u128>()
                    .map_err(|_| Error("§3.3: invalid length".into()))?;
                if self.ptr_width < 128 && n >= 1u128 << self.ptr_width {
                    return err("§3.3", "length does not fit pointer");
                }
                Ok(())
            } else {
                rd(s).map(|_| ())
            }
        };
        let code = |s: &str| -> Result<()> {
            let fw = self.fault_width(f)?;
            crate::BitVector::parse_unsigned_exact(
                usize::try_from(fw).map_err(|_| Error("host limit: fault width".into()))?,
                s,
            )?;
            Ok(())
        };
        if let Some(n) = op.strip_prefix("lift.const.i") {
            arity(1, 1)?;
            // SPEC-QUESTIONS 15: a lifted suffix names the reg[N] width; it is
            // not bounded by iN's 128.
            let w = lifted_suffix(n)?;
            eq(dest(&i.lhs[0])?, w)?;
            const_literal(&args[0], w)?;
        } else if let Some(base) = op
            .strip_prefix("lift.")
            .filter(|s| is_binop(s) || is_div(s))
        {
            let _ = base;
            arity(2, 1)?;
            eq(rd(&args[0])?, rd(&args[1])?)?;
            eq(dest(&i.lhs[0])?, rd(&args[0])?)?;
        } else if let Some(pred) = op.strip_prefix("lift.icmp.") {
            arity(2, 1)?;
            if !["eq", "ne", "ult", "ule", "slt", "sle"].contains(&pred) {
                return err("§4.8", "unknown comparison predicate");
            }
            eq(rd(&args[0])?, rd(&args[1])?)?;
            eq(dest(&i.lhs[0])?, 1)?;
        } else if op == "lift.select" {
            arity(3, 1)?;
            eq(rd(&args[0])?, 1)?;
            eq(rd(&args[1])?, rd(&args[2])?)?;
            eq(dest(&i.lhs[0])?, rd(&args[1])?)?;
        } else if let Some((cast, n)) = op
            .strip_prefix("lift.")
            .and_then(|s| s.split_once(".i"))
            .filter(|(c, _)| ["zext", "sext", "trunc"].contains(c))
        {
            arity(1, 1)?;
            let out = lifted_suffix(n)?;
            eq(dest(&i.lhs[0])?, out)?;
            let input = rd(&args[0])?;
            if if cast == "trunc" {
                out >= input
            } else {
                out <= input
            } {
                return err("§3.3", "cast has wrong width direction");
            }
        } else {
            match op {
                "lift.mov" => {
                    arity(1, 1)?;
                    eq(dest(&i.lhs[0])?, rd(&args[0])?)?;
                }
                "lift.parmov" => {
                    if args.len() != 1 {
                        return err("§4.8", "parmov needs one source tuple");
                    }
                    let src = list(&args[0])?;
                    if src.len() != i.lhs.len() || src.is_empty() {
                        return err("§4.8", "parmov source/destination arity");
                    }
                    let mut seen = BTreeSet::new();
                    for (d, s) in i.lhs.iter().zip(src) {
                        if !seen.insert(d) {
                            return err("§3.3", "parmov destinations alias");
                        }
                        eq(dest(d)?, rd(&s)?)?;
                    }
                }
                "lift.label" => {
                    arity(1, 1)?;
                    label(&args[0])?;
                    let w = dest(&i.lhs[0])?;
                    if w < 128 && (1u128 << w) <= self.index_space_size(f)? {
                        return err("§3.3", "label location cannot represent index space");
                    }
                }
                "lift.addr" => {
                    arity(1, 1)?;
                    eq(dest(&i.lhs[0])?, self.ptr_width as u64)?;
                    let n = args[0]
                        .strip_prefix('@')
                        .ok_or_else(|| Error("§4.8: addr requires @name".into()))?;
                    let global = self
                        .section("layout")
                        .and_then(|l| l.get("globals").ok())
                        .and_then(|g| g.object().ok())
                        .is_some_and(|g| g.contains_key(n));
                    if !global && !(f.kind == "oracle" && f.params[3].iter().any(|p| p.name == n)) {
                        return err("§4.2", format!("addr cannot name {n} in {}", f.name));
                    }
                }
                "lift.boundscheck" => {
                    arity(2, 0)?;
                    addr(&args[0])?;
                    length(&args[1])?;
                }
                "lift.alloc" => {
                    arity(1, 1)?;
                    length(&args[0])?;
                    eq(dest(&i.lhs[0])?, self.ptr_width as u64)?;
                }
                "lift.free" => {
                    arity(1, 0)?;
                    addr(&args[0])?;
                }
                "lift.faultif" => {
                    arity(2, 0)?;
                    eq(rd(&args[0])?, 1)?;
                    code(&args[1])?;
                }
                "lift.call" => {
                    arity(2, 1)?;
                    let callee = self.declaration(
                        args[0]
                            .strip_prefix('@')
                            .ok_or_else(|| Error("§4.8: lift.call requires @symbol".into()))?,
                    )?;
                    if callee.kind != "lproc" {
                        return err("§3.7", "lift.call requires lproc");
                    }
                    let src = list(&args[1])?;
                    if src.len() != callee.params[0].len() {
                        return err("§3.3", "lproc argument count");
                    }
                    for (s, p) in src.iter().zip(&callee.params[0]) {
                        eq(rd(s)?, p.ty.width(self.ptr_width)?)?;
                    }
                    eq(
                        dest(&i.lhs[0])?,
                        callee.params[1][0].ty.width(self.ptr_width)?,
                    )?;
                }
                "lift.ret" | "lift.retval" => {
                    arity(1, 0)?;
                    if (op == "lift.ret") != (f.kind == "lproc") {
                        return err("§4.2", "ret/retval in wrong function kind");
                    }
                    eq(rd(&args[0])?, f.params[1][0].ty.width(self.ptr_width)?)?;
                }
                "qjump" => {
                    arity(1, 0)?;
                    label(&args[0])?;
                }
                "qbranch" => {
                    arity(3, 0)?;
                    eq(rd(&args[0])?, 1)?;
                    label(&args[1])?;
                    label(&args[2])?;
                }
                "qjumpr" => {
                    arity(1, 0)?;
                    let w = rd(&args[0])?;
                    if w < 128 && (1u128 << w) <= self.index_space_size(f)? {
                        return err("§3.3", "qjumpr width cannot represent index space");
                    }
                }
                "fault" => {
                    arity(1, 0)?;
                    code(&args[0])?;
                }
                "lift.oracle_call" | "lift.oracle_uncall" | "lift.perm_call" => {
                    self.validate_region_call(f, i, types)?
                }
                _ if op.starts_with("lift.load.i") || op.starts_with("lift.store.i") => {
                    let load = op.starts_with("lift.load");
                    arity(if load { 1 } else { 2 }, if load { 1 } else { 0 })?;
                    let n = op
                        .rsplit_once(".i")
                        .expect("suffix")
                        .1
                        .parse::<u64>()
                        .map_err(|_| Error("§4.8: malformed memory width".into()))?;
                    if n == 0 || n > 128 || n % 8 != 0 {
                        return err("§3.3", "load/store width must be a byte multiple up to 128");
                    }
                    addr(&args[0])?;
                    eq(
                        if load {
                            dest(&i.lhs[0])?
                        } else {
                            rd(&args[1])?
                        },
                        n,
                    )?;
                }
                _ => return err("§4.8", format!("unknown or invalid lifted opcode {op}")),
            }
        }
        Ok(())
    }
    fn validate_layout(&self) -> Result<()> {
        let Some(l) = self.section("layout") else {
            for f in self.functions.values() {
                if f.kind == "lproc"
                    || f.params
                        .iter()
                        .flatten()
                        .any(|p| matches!(p.ty, Type::Region(..)))
                    || f.instructions().any(|i| {
                        matches!(
                            i.op.as_str(),
                            "lift.addr"
                                | "lift.boundscheck"
                                | "lift.alloc"
                                | "lift.free"
                                | "lift.call"
                                | "lift.ret"
                                | "alloc"
                                | "free"
                        ) || i.op.starts_with("lift.load.")
                            || i.op.starts_with("lift.store.")
                            || i.op.starts_with("load.")
                            || i.op.starts_with("store.")
                    })
                {
                    return err("§7.2", "memory or lprocs require layout");
                }
            }
            return Ok(());
        };
        l.fields(
            &[
                "ptr_width",
                "H",
                "A_tab",
                "S_cells",
                "null_size",
                "heap_base",
                "heap_limit_max",
                "globals",
                "interface_regions",
            ],
            &["block_image"],
        )?;
        if l.get("ptr_width")?.uint()? != self.ptr_width as u128 {
            return err("§7.2", "layout pointer width mismatch");
        }
        let h = l.get("H")?.uint()?;
        let a = l.get("A_tab")?.uint()?;
        l.get("S_cells")?.uint()?;
        // §7.2: H <= ptr_width; A_tab and S_cells carry no bound.
        if h > self.ptr_width as u128 {
            return err("§7.2", "H exceeds ptr_width");
        }
        let capacity = h;
        // 2^A_tab table entries; None means at least 2^128 (never reached).
        let entries = crate::oracle::power_of_two(a);
        let globals = l.get("globals")?.object()?;
        let null = l.get("null_size")?.uint()?;
        if null == 0 {
            return err("§7.1", "null region must be nonempty");
        }
        let mut shared = vec![
            (0, null, "null".to_string()),
            (
                l.get("heap_base")?.uint()?,
                l.get("heap_limit_max")?.uint()?,
                "heap".into(),
            ),
        ];
        for (n, g) in globals {
            g.fields(&["base", "size", "image"], &[])?;
            let image = g.get("image")?.string()?;
            if image != n
                || self.data.get(n).is_none_or(|b| {
                    b.len() as u128 != g.get("size").and_then(Json::uint).unwrap_or(u128::MAX)
                })
            {
                return err(
                    "§7.2",
                    "global image must be same-name data of declared size",
                );
            }
            shared.push((g.get("base")?.uint()?, g.get("size")?.uint()?, n.clone()));
        }
        let interfaces = l.get("interface_regions")?.object()?;
        for (n, regions) in interfaces {
            let f = self
                .functions
                .get(n)
                .filter(|f| f.kind == "oracle")
                .ok_or_else(|| Error("§7.2: interface layout must name an oracle".into()))?;
            let regions = regions.object()?;
            if regions.len() != f.params[3].len() {
                return err("§7.2", "interface mapping count mismatch");
            }
            let mut ranges = shared.clone();
            for p in &f.params[3] {
                let (a, w) = match p.ty {
                    Type::Region(a, w) => (a, w),
                    _ => return err("§5.1", "mem slot requires region"),
                };
                if w != 8 {
                    return err("§7.2", "lifted interface regions require W=8");
                }
                let r = regions
                    .get(&p.name)
                    .ok_or_else(|| Error("§7.2: missing interface mapping".into()))?;
                r.fields(&["base"], &[])?;
                let size = crate::oracle::power_of_two(a as u128)
                    .ok_or_else(|| Error("§7.2: interface region exceeds 2^H".into()))?;
                ranges.push((r.get("base")?.uint()?, size, p.name.clone()));
            }
            if entries.is_some_and(|e| globals.len() as u128 + regions.len() as u128 > e) {
                return err("§7.2", "too many allocation pre-entries");
            }
            validate_ranges(&ranges, capacity)?;
        }
        for f in self
            .functions
            .values()
            .filter(|f| f.kind == "oracle" && !f.params[3].is_empty())
        {
            if !interfaces.contains_key(&f.name) {
                return err("§7.2", "missing oracle interface layout");
            }
        }
        if entries.is_some_and(|e| globals.len() as u128 > e) {
            return err("§7.2", "too many globals");
        }
        validate_ranges(&shared, capacity)
    }
}

fn is_head(s: &str) -> bool {
    [
        "fn cc @",
        "fn r @",
        "oracle @",
        "lproc @",
        "action @",
        "perm_action @",
    ]
    .iter()
    .any(|p| s.starts_with(p))
}
pub(crate) fn is_binop(op: &str) -> bool {
    [
        "add", "sub", "mul", "and", "or", "xor", "shl", "lshr", "ashr",
    ]
    .contains(&op)
}
pub(crate) fn is_div(op: &str) -> bool {
    ["udiv", "urem", "sdiv", "srem"].contains(&op)
}
/// The width suffix of a lifted row (`lift.const.iN`, `lift.zext.iM`, ...).
pub(crate) fn lifted_suffix(n: &str) -> Result<u64> {
    let w = host_count(n, "lifted width suffix")?;
    if w == 0 {
        return err("§3.3", "lifted width suffix must be positive");
    }
    Ok(w)
}
/// A const literal of an N-bit destination: canonical INT (§16.1) with
/// -2^(N-1) <= c < 2^N, i.e. representable as a signed or an unsigned N-bit
/// value; anything else would be a silent truncation (SPEC-QUESTIONS 12).
pub(crate) fn const_literal(s: &str, width: u64) -> Result<()> {
    if !integer(s) {
        return err("§16.1", format!("invalid or noncanonical INT literal {s}"));
    }
    let w = usize::try_from(width).map_err(|_| Error("host limit: const width".into()))?;
    let magnitude = s.trim_start_matches('-');
    let in_range = match crate::BitVector::parse_unsigned_exact(w + 1, magnitude) {
        Ok(m) if s.starts_with('-') => {
            // |c| <= 2^(N-1): bits >= N-1 must be zero, except exactly 2^(N-1).
            let mut limit = crate::BitVector::zero(w + 1)?;
            limit.set(w - 1, true)?;
            m.unsigned_cmp(&limit)? != std::cmp::Ordering::Greater
        }
        Ok(m) => !m.bit(w)?,
        Err(_) => false,
    };
    if !in_range {
        return err(
            "§3.3",
            format!("const literal {s} is outside [-2^(N-1), 2^N) for N = {width}"),
        );
    }
    Ok(())
}
pub(crate) fn parse_bits(s: &str, width: u16) -> Result<u128> {
    if !integer(s) {
        return err("§16.1", "invalid integer literal");
    }
    let negative = s.starts_with('-');
    let magnitude = s
        .trim_start_matches('-')
        .bytes()
        .fold(0u128, |value, digit| {
            value.wrapping_mul(10).wrapping_add((digit - b'0') as u128)
        });
    let n = if negative {
        0u128.wrapping_sub(magnitude)
    } else {
        magnitude
    };
    Ok(n & mask(width))
}
pub(crate) fn mask(width: u16) -> u128 {
    if width == 128 {
        u128::MAX
    } else {
        (1u128 << width) - 1
    }
}
fn validate_header(f: &Function) -> Result<()> {
    let empty = |indices: &[usize]| -> Result<()> {
        if indices.iter().any(|i| !f.params[*i].is_empty()) {
            err("§16.2", "nonempty reserved parameter list")
        } else {
            Ok(())
        }
    };
    let all = |ps: &[Param], kind: fn(&Type) -> bool| -> Result<()> {
        if ps.iter().any(|p| !kind(&p.ty)) {
            err("§3.3", "invalid interface operand type")
        } else {
            Ok(())
        }
    };
    let classical = |t: &Type| matches!(t, Type::Int(_) | Type::Ptr | Type::F64 | Type::Token);
    match f.kind.as_str() {
        "oracle" => {
            all(&f.params[4], classical)?;
            if f.params[1].len() != 1 || f.params[2].len() != 1 {
                return err("§5.1", "oracle needs one target and one fault slot");
            }
            for j in 0..3 {
                all(&f.params[j], |t| matches!(t, Type::Wire(_)))?;
            }
            if f.params[2][0].ty.width(1)? < 4 {
                return err("§5.4", "fault width must be at least 4");
            }
            all(&f.params[3], |t| matches!(t, Type::Region(..)))?;
            if f.params[0]
                .iter()
                .chain(&f.params[3])
                .any(|p| p.role.as_deref() != Some("in"))
            {
                return err("§5.1", "oracle inputs and regions require in role");
            }
        }
        "lproc" => {
            empty(&[2, 3, 4])?;
            if f.params[1].len() != 1 {
                return err("§3.7", "lproc needs one result location");
            }
            all(&f.params[0], |t| matches!(t, Type::Reg(_)))?;
            all(&f.params[1], |t| matches!(t, Type::Reg(_)))?;
        }
        "action" | "perm_action" => {
            all(&f.params[3], classical)?;
            empty(&[4])?;
            for j in 0..2 {
                all(&f.params[j], |t| matches!(t, Type::Wire(_)))?;
            }
            all(&f.params[2], |t| matches!(t, Type::Region(..)))?;
            if f.params[2]
                .iter()
                .any(|p| !matches!(p.role.as_deref(), Some("in" | "mut")))
            {
                return err("§5.1", "action regions need in or mut roles");
            }
        }
        _ => {
            empty(&[2, 3, 4])?;
            for group in 0..2 {
                all(&f.params[group], |t| {
                    matches!(
                        t,
                        Type::Int(_) | Type::Ptr | Type::F64 | Type::Token | Type::Qreg(_)
                    )
                })?;
            }
            if !f.regs.is_empty() || !f.bounds.is_empty() {
                return err(
                    "§3.3",
                    "classical functions use SSA definitions, not a mutable regs file",
                );
            }
        }
    }
    if matches!(f.kind.as_str(), "oracle" | "lproc") {
        all(&f.regs, |t| matches!(t, Type::Reg(_)))?;
        if !f.bounds.is_empty() {
            return err("§5.3", "bounds only on actions");
        }
    }
    Ok(())
}
/// §7.1/§7.2: every range lies in [0, 2^H) and nonempty ranges are disjoint.
/// Ends are exact: `None` is the end 2^128 (H = ptr_width = 128).
fn validate_ranges(ranges: &[(u128, u128, String)], h: u128) -> Result<()> {
    let mut sorted = Vec::new();
    for (base, size, n) in ranges {
        let end = match base.overflowing_add(*size) {
            (end, false) => Some(end),
            (0, true) => None,
            _ => return err("§7.2", format!("{n} exceeds 2^128")),
        };
        let inside = match (end, crate::oracle::power_of_two(h)) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(end), Some(cap)) => end <= cap,
        };
        if !inside {
            return err("§7.2", format!("{n} exceeds the 2^H address space"));
        }
        if *size != 0 {
            sorted.push((*base, end, n));
        }
    }
    sorted.sort();
    for p in sorted.windows(2) {
        if p[0].1.is_none_or(|end| end > p[1].0) {
            return err(
                "§7.2",
                format!("overlapping layout ranges {} and {}", p[0].2, p[1].2),
            );
        }
    }
    Ok(())
}
