use bennett_isa_ref::{
    json::Json, Argument, BitVector, Budget, ClassicalMachine, ClassicalValue, Complex, Error,
    Function, Object, OracleInput, OracleVectorInput, OracleVectorMachine, QuantumBindings,
    QuantumRegion, QuantumRegisters, QuantumState, Result, SnapshotDelta, StepPoint, Type,
    Xoshiro256,
};
use std::collections::VecDeque;

const USAGE:&str="bennett-isa-ref check <object>\nbennett-isa-ref hash <object>\nbennett-isa-ref run-oracle <object> <oracle> --input <x[,y...]> [--budget B[,heap,stack,depth]] [--mem name=hex] [--arg value]\nbennett-isa-ref step-check <object> <oracle> (--exhaustive [--max-bits N] | --suite | --samples N) [--seed a,b,c,d] [--budget B[,heap,stack,depth]] [--mem name=hex] [--arg value]\nbennett-isa-ref run-cc <object> <entry> [--args <a[,b...]> | --input <a[,b...]> | --arg value...] [--entropy hex] [--replies file] [--draws <d[,e...]>] [--budget B[,heap,stack,depth]] [--mem global=hex]\nbennett-isa-ref reverse <object> <entry> [run-cc options] [--steps N]\nbennett-isa-ref reverse <object> <oracle> --input <x[,y...]> [--budget B[,heap,stack,depth]] [--steps N] [--mem name=hex] [--arg value]\nbennett-isa-ref reverse <object> <action> (--mut <v[,...]> [--in <v[,...]>] [--mem name=hex] | --basis <bits> | --state <bits=re[,im];...>) [--args <a[,...]>] [--adjoint]\nbennett-isa-ref run-action <object> <action> (--mut <v[,...]> [--in <v[,...]>] [--mem name=hex] | --basis <bits> | --state <bits=re[,im];...>) [--args <a[,...]> | --arg value...] [--adjoint]\nbennett-isa-ref run-program <object> <cc-entry> [--args <a[,b...]> | --input <a[,b...]> | --arg value...] [--entropy hex] [--replies file] [--draws <d[,e...]>] [--budget B[,heap,stack,depth]] [--mem global=hex] [--reverse] [--target file]";
fn fail(message: impl std::fmt::Display) -> Error {
    Error::new(message.to_string())
}
fn uint(s: &str) -> Result<u128> {
    if let Some(hex) = s.strip_prefix("0x") {
        u128::from_str_radix(hex, 16).map_err(fail)
    } else {
        s.parse().map_err(fail)
    }
}
fn number(n: u128) -> Json {
    if n > 9_007_199_254_740_991 {
        Json::String(n.to_string())
    } else {
        Json::Number(n.to_string())
    }
}
fn vector_number(value: &BitVector) -> Json {
    match value.to_u128() {
        Ok(n) => number(n),
        Err(_) => Json::String(value.to_decimal()),
    }
}
fn vector_input(width: usize, raw: &str) -> Result<BitVector> {
    if let Some(hex) = raw.strip_prefix("0x") {
        if hex.is_empty() {
            return Err(fail("empty hexadecimal input"));
        }
        let mut value = BitVector::zero(width)?;
        for (i, c) in hex.chars().rev().enumerate() {
            let digit = c
                .to_digit(16)
                .ok_or_else(|| fail("invalid hexadecimal input"))?;
            for bit in 0..4 {
                if digit & (1 << bit) != 0 {
                    value.set(i * 4 + bit, true)?;
                }
            }
        }
        Ok(value)
    } else {
        BitVector::parse_unsigned_exact(width, raw)
    }
}
fn string(s: impl Into<String>) -> Json {
    Json::String(s.into())
}
fn object(pairs: impl IntoIterator<Item = (String, Json)>) -> Json {
    Json::Object(pairs.into_iter().collect())
}
fn budget(raw: &str, default: Budget) -> Result<Budget> {
    let v = raw
        .split(',')
        .map(|s| uint(s).map_err(|e| fail(format!("§5.4: invalid budget component {s:?}: {e}"))))
        .collect::<Result<Vec<_>>>()?;
    if v.len() != 1 && v.len() != 4 {
        return Err(fail("budget requires B or B,heap,stack,depth (§5.4)"));
    }
    let steps = u64::try_from(v[0]).map_err(|e| {
        Error::host_resource(format!(
            "§5.4: host limit: step budget exceeds the u64 counter: {e}"
        ))
    })?;
    Ok(if v.len() == 1 {
        Budget { steps, ..default }
    } else {
        Budget {
            steps,
            heap: v[1],
            stack: v[2],
            depth: v[3],
        }
    })
}
fn hex_buffer(len: usize) -> Result<String> {
    let count = len.checked_mul(2).ok_or_else(|| {
        Error::host_resource("§7.3/§10: host limit: CLI hex byte count exceeds usize".into())
    })?;
    let mut out = String::new();
    out.try_reserve_exact(count)
        .map_err(|e| Error::host_resource(format!("§7.3/§10: host limit: CLI hex bytes: {e}")))?;
    Ok(out)
}
fn hex(bytes: &[u8]) -> Result<String> {
    let mut out = hex_buffer(bytes.len())?;
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    for b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
    Ok(out)
}
#[test]
fn r21_cli_hex_storage_memory_limited() {
    if std::env::var_os("R21_HEX_CHILD").is_none() {
        let out = std::process::Command::new("python3").args(["-c",
            "import os,resource,sys; resource.setrlimit(resource.RLIMIT_AS,(256*1024*1024,)*2); resource.setrlimit(resource.RLIMIT_CORE,(0,0)); os.execv(sys.argv[1],sys.argv[1:])"])
            .arg(std::env::current_exe().unwrap())
            .args(["--exact", "r21_cli_hex_storage_memory_limited", "--nocapture"])
            .env("R21_HEX_CHILD", "1").output().unwrap();
        assert!(out.status.success(), "{out:?}");
        return;
    }
    std::panic::catch_unwind(|| {
        for len in [0, 1, 8, (1usize << 63) - 1, 1 << 63, usize::MAX] {
            let r = hex_buffer(len);
            if len <= 8 {
                assert!(r.unwrap().capacity() >= 2 * len);
            } else {
                let e = r.unwrap_err().to_string();
                assert!(e.contains('§') && e.contains("host limit"), "{e}");
            }
        }
        assert_eq!(hex(&[0, 127, 128, 255]).unwrap(), "007f80ff");
    })
    .expect("no CLI output length panic");
}
fn parse_hex(s: &str) -> Result<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(fail(
            "§16.1: invalid HEX (even-length hexadecimal expected)",
        ));
    }
    let mut out = Vec::new();
    out.try_reserve_exact(s.len() / 2)
        .map_err(|e| Error::host_resource(format!("§10: host limit: decoded reply bytes: {e}")))?;
    for i in (0..s.len()).step_by(2) {
        out.push(u8::from_str_radix(&s[i..i + 2], 16).map_err(fail)?);
    }
    Ok(out)
}
/// A binary64 as a canonical JSON number (RFC 8785 via `Json::parse`).
fn float_json(v: f64) -> Result<Json> {
    if !v.is_finite() {
        return Err(fail("§16.1: non-finite binary64 has no JSON number"));
    }
    Json::parse(&v.to_string())
}
/// One classical value as `{"type","value"}`: integers and pointers are
/// unsigned decimal strings of their bit-vector values (§9.1; a signed
/// reading at the declared width is the caller's, §3.3), f64 a JSON
/// number, token its four §5.4 components.
fn value_json(v: &ClassicalValue) -> Result<Json> {
    Ok(match v {
        ClassicalValue::Int(w, bits) => object([
            ("type".into(), string(format!("i{w}"))),
            ("value".into(), string(bits.to_string())),
        ]),
        ClassicalValue::Ptr(bits) => object([
            ("type".into(), string("ptr")),
            ("value".into(), string(bits.to_string())),
        ]),
        ClassicalValue::F64(bits) => object([
            ("type".into(), string("f64")),
            ("value".into(), float_json(f64::from_bits(*bits))?),
        ]),
        ClassicalValue::Token(b) => object([
            ("type".into(), string("token")),
            (
                "value".into(),
                object([
                    ("depth".into(), number(b.depth)),
                    ("heap".into(), number(b.heap)),
                    ("stack".into(), number(b.stack)),
                    ("steps".into(), number(b.steps as u128)),
                ]),
            ),
        ]),
        ClassicalValue::Qreg(id) => object([
            ("type".into(), string("qreg")),
            ("value".into(), number(*id as u128)),
        ]),
    })
}
fn values_json(values: &[ClassicalValue]) -> Result<Json> {
    Ok(Json::Array(collect_output(
        values.iter().map(value_json),
        "outcome values",
    )?))
}
fn fault_name(code: u128) -> Option<&'static str> {
    Some(match code {
        0 => "FAULT_NONE",
        1 => "FAULT_BUDGET_STEPS",
        2 => "FAULT_BUDGET_HEAP",
        3 => "FAULT_BUDGET_DEPTH",
        4 => "FAULT_UB_TRAP",
        5 => "FAULT_UB_DIV0",
        6 => "FAULT_UB_SDIV_OVERFLOW",
        7 => "FAULT_UB_MEMFAULT",
        8 => "FAULT_EFFECT_UNMODELLED",
        9 => "FAULT_CALLEE",
        10 => "FAULT_ENTROPY_EXHAUSTED",
        _ => return None,
    })
}
fn faults_json(fault: Option<u128>) -> Json {
    match fault {
        None => Json::Array(Vec::new()),
        Some(code) => Json::Array(vec![object([
            ("code".into(), number(code)),
            (
                "name".into(),
                fault_name(code).map(string).unwrap_or(Json::Null),
            ),
        ])]),
    }
}
fn tape_json(m: &ClassicalMachine) -> Result<Json> {
    Ok(Json::Array(collect_output(
        m.effect_tape().iter().map(|e| {
            Ok(object([
                (
                    "args".into(),
                    Json::Array(e.args.iter().map(|a| number(*a)).collect()),
                ),
                ("bytes".into(), string(hex(&e.bytes)?)),
                ("dev".into(), string(e.dev.clone())),
                ("op".into(), string(e.op.clone())),
            ]))
        }),
        "tape_json",
    )?))
}
fn measurements_json(m: &ClassicalMachine) -> Json {
    Json::Array(
        m.measurements()
            .into_iter()
            .map(|(reg, bits)| {
                // Wide measurements keep their exact bit-vector value.
                let mut value = BitVector::zero(bits.len()).expect("measured width");
                for (i, b) in bits.iter().enumerate() {
                    value.set(i, *b).expect("measured bit");
                }
                object([
                    (
                        "bits".into(),
                        string(
                            bits.iter()
                                .map(|b| if *b { '1' } else { '0' })
                                .collect::<String>(),
                        ),
                    ),
                    ("reg".into(), number(reg as u128)),
                    ("value".into(), vector_number(&value)),
                ])
            })
            .collect(),
    )
}
fn host_requests_json(m: &ClassicalMachine) -> Result<Json> {
    Ok(Json::Array(collect_output(
        m.host_exchanges().map(|e| {
            Ok(object([
                ("address".into(), number(e.request.address)),
                ("capacity".into(), number(e.request.capacity)),
                ("payload".into(), string(hex(&e.request.payload)?)),
                ("reply".into(), string(hex(&e.reply)?)),
                ("service".into(), string(e.request.service.clone())),
            ]))
        }),
        "host_requests_json",
    )?))
}
fn console_json(m: &ClassicalMachine) -> Result<Json> {
    match std::str::from_utf8(m.console()) {
        Ok(s) => Ok(string(copy_text(s, "console UTF-8 output")?)),
        Err(_) => Ok(Json::Null),
    }
}
fn earg(message: impl std::fmt::Display) -> Error {
    Error::new(format!("§10: E_ARG: {message}"))
}
/// Options shared by the CC/R entry commands (`run-cc`, `reverse`,
/// `run-program`); command-specific options are rejected for the others.
struct CcOptions {
    args: Vec<String>,
    entropy: Vec<u8>,
    replies: Option<String>,
    draws: Vec<f64>,
    fuel: u64,
    token_defaults: Budget,
    mem: Vec<(String, Vec<u8>)>,
    steps: Option<u64>,
    target: Option<String>,
    reverse_flag: bool,
}
fn parse_cc_options(
    program: &Object,
    command: &str,
    args: &[String],
    start: usize,
) -> Result<CcOptions> {
    let mut limits = program.default_budget(1_000_000)?;
    let mut options = CcOptions {
        args: Vec::new(),
        entropy: Vec::new(),
        replies: None,
        draws: Vec::new(),
        fuel: 1_000_000,
        token_defaults: limits,
        mem: Vec::new(),
        steps: None,
        target: None,
        reverse_flag: false,
    };
    let mut args_seen = false;
    let mut budget4_seen = false;
    let mut once = std::collections::BTreeSet::new();
    let mut pos = start;
    while pos < args.len() {
        let option = args[pos].as_str();
        pos += 1;
        if option == "--reverse" {
            if command != "run-program" {
                return Err(fail(
                    "--reverse is a run-program option (§10 bennett_reverse)",
                ));
            }
            if options.reverse_flag {
                return Err(fail("--reverse given twice"));
            }
            options.reverse_flag = true;
            continue;
        }
        if !option.starts_with("--") {
            return Err(fail(format!("unexpected argument {option}\n{USAGE}")));
        }
        let value = args
            .get(pos)
            .ok_or_else(|| fail(format!("missing value for {option}")))?;
        pos += 1;
        match option {
            "--args" | "--input" => {
                if args_seen {
                    return Err(fail(
                        "--args/--input/--arg: pass the positional argument list exactly once, in one form",
                    ));
                }
                args_seen = true;
                options.args = if value.is_empty() {
                    Vec::new()
                } else {
                    value.split(',').map(str::to_string).collect()
                };
            }
            "--arg" => {
                if args_seen && !options.args.is_empty() {
                    return Err(fail(
                        "--args/--input/--arg: pass the positional argument list exactly once, in one form",
                    ));
                }
                // Repeated --arg is one form; mixing it with --args is not.
                if args_seen {
                    return Err(fail(
                        "--args/--input/--arg: pass the positional argument list exactly once, in one form",
                    ));
                }
                args_seen = false;
                once.insert("--arg-seen");
                options.args.push(value.clone());
            }
            "--entropy" => {
                if !once.insert(option) {
                    return Err(fail("--entropy given twice"));
                }
                options.entropy = parse_hex(value)?;
            }
            "--replies" => {
                if !once.insert(option) {
                    return Err(fail("--replies given twice"));
                }
                options.replies = Some(value.clone());
            }
            "--draws" => {
                if !once.insert(option) {
                    return Err(fail("--draws given twice"));
                }
                if !value.is_empty() {
                    options.draws = value
                        .split(',')
                        .map(|d| d.parse::<f64>().map_err(fail))
                        .collect::<Result<_>>()?;
                }
            }
            "--budget" => {
                budget4_seen |= value.matches(',').count() == 3;
                limits = budget(value, limits)?;
                options.fuel = limits.steps;
                options.token_defaults = limits;
            }
            "--mem" => {
                let (name, hex) = value
                    .split_once('=')
                    .ok_or_else(|| fail("--mem requires name=hex"))?;
                if options.mem.iter().any(|(n, _)| n == name) {
                    return Err(fail(format!("duplicate --mem {name}")));
                }
                options.mem.push((name.into(), parse_hex(hex)?));
            }
            "--steps" => {
                if command != "reverse" {
                    return Err(fail("--steps is a reverse option"));
                }
                options.steps = Some(uint(value).and_then(|n| u64::try_from(n).map_err(fail))?);
            }
            "--target" => {
                if command != "run-program" {
                    return Err(fail("--target is a run-program option (§10 bennett_load)"));
                }
                options.target = Some(value.clone());
            }
            _ => return Err(fail(format!("unknown option {option}"))),
        }
        if (option == "--args" || option == "--input") && once.contains("--arg-seen") {
            return Err(fail(
                "--args/--input/--arg: pass the positional argument list exactly once, in one form",
            ));
        }
    }
    if budget4_seen {
        program.validate_budget(options.token_defaults)?;
    }
    Ok(options)
}
fn int_bits(raw: &str, width: u16) -> Result<u128> {
    let mask = if width == 128 {
        u128::MAX
    } else {
        (1u128 << width) - 1
    };
    if let Some(neg) = raw.strip_prefix('-') {
        let magnitude: u128 = neg
            .parse()
            .map_err(|_| earg(format!("invalid integer {raw}")))?;
        let limit = 1u128 << (width - 1);
        if magnitude > limit {
            return Err(earg(format!("integer {raw} does not fit i{width}")));
        }
        return Ok(magnitude.wrapping_neg() & mask);
    }
    let value = uint(raw).map_err(|_| earg(format!("invalid integer {raw}")))?;
    if value & !mask != 0 {
        return Err(earg(format!("integer {raw} does not fit i{width}")));
    }
    Ok(value)
}
/// §10 positional binding: raw argument i is parsed against parameter i's
/// declared type. `qreg:<bits>` allocates and prepares a fresh register
/// (first character is wire 0); `token:B[,heap,stack,depth]` is a §5.4
/// validated budget token; `f64:<float>` a binary64.
fn classical_args(
    program: &Object,
    f: &Function,
    raw: &[String],
    defaults: Budget,
) -> Result<(Vec<ClassicalValue>, QuantumRegisters)> {
    let params = &f.params[0];
    if raw.len() != params.len() {
        return Err(earg(format!(
            "entry @{} takes {} arguments, {} given",
            f.name,
            params.len(),
            raw.len()
        )));
    }
    let mut registers = QuantumRegisters::new();
    let mut values = Vec::new();
    for (p, raw) in params.iter().zip(raw) {
        values.push(match p.ty {
            Type::Int(w) => ClassicalValue::Int(w, int_bits(raw, w)?),
            Type::Ptr => ClassicalValue::Ptr(int_bits(raw, program.ptr_width)?),
            Type::F64 => {
                let text = raw.strip_prefix("f64:").unwrap_or(raw);
                let value: f64 = text
                    .parse()
                    .map_err(|_| earg(format!("invalid f64 argument {raw}")))?;
                if !value.is_finite() {
                    return Err(earg(format!("f64 argument {raw} is not finite")));
                }
                ClassicalValue::F64(value.to_bits())
            }
            Type::Token => {
                let text = raw.strip_prefix("token:").unwrap_or(raw);
                let token = budget(text, defaults).map_err(|e| earg(e.to_string()))?;
                program.validate_budget(token)?;
                ClassicalValue::Token(token)
            }
            Type::Qreg(n) => {
                let bits = raw
                    .strip_prefix("qreg:")
                    .ok_or_else(|| earg(format!("qreg argument must be qreg:<bits>, got {raw}")))?;
                let n = usize::try_from(n)
                    .map_err(|_| earg(format!("qreg width {n} exceeds host storage")))?;
                if bits.len() != n || !bits.bytes().all(|b| matches!(b, b'0' | b'1')) {
                    return Err(earg(format!(
                        "qreg argument needs exactly {n} bit characters, got {raw}"
                    )));
                }
                let id = registers.allocate(n)?;
                let prepared: Vec<bool> = bits.bytes().map(|b| b == b'1').collect();
                registers.prepare(id, &prepared)?;
                ClassicalValue::Qreg(id)
            }
            ref other => {
                return Err(earg(format!(
                    "parameter {} has non-classical type {}",
                    p.name,
                    other.text()
                )))
            }
        });
    }
    Ok((values, registers))
}
struct Reply {
    service: String,
    payload: Vec<u8>,
    reply: Vec<u8>,
}
/// A --replies file is a JSON array of `{"service","payload","reply"}`
/// objects, payload/reply hex, consumed in request order (§10 HOST_REQUEST).
fn load_replies(path: Option<&str>) -> Result<VecDeque<Reply>> {
    let Some(path) = path else {
        return Ok(VecDeque::new());
    };
    let text = std::fs::read_to_string(path).map_err(|e| fail(format!("{path}: {e}")))?;
    let entries = Json::parse(&text)?;
    let mut queue = VecDeque::new();
    for entry in entries.array()? {
        entry.fields(&["payload", "reply", "service"], &[])?;
        queue
            .try_reserve(1)
            .map_err(|e| Error::host_resource(format!("§10: host limit: reply queue: {e}")))?;
        queue.push_back(Reply {
            service: entry.get("service")?.string()?.into(),
            payload: parse_hex(entry.get("payload")?.string()?)?,
            reply: parse_hex(entry.get("reply")?.string()?)?,
        });
    }
    Ok(queue)
}
fn delta_registers_json(delta: &SnapshotDelta) -> Result<Vec<Json>> {
    collect_output(
        delta.registers.iter().map(|r| {
            Ok(object([
                ("activation".into(), number(r.activation as u128)),
                ("name".into(), string(r.name.clone())),
                (
                    "post".into(),
                    match &r.post {
                        Some(v) => value_json(v)?,
                        None => Json::Null,
                    },
                ),
                (
                    "pre".into(),
                    match &r.pre {
                        Some(v) => value_json(v)?,
                        None => Json::Null,
                    },
                ),
            ]))
        }),
        "delta_registers_json",
    )
}
fn delta_memory_json(delta: &SnapshotDelta) -> Result<Vec<Json>> {
    collect_output(
        delta.memory.iter().map(|(address, pre, post)| {
            Ok(object([
                ("address".into(), number(*address)),
                ("post".into(), number(*post as u128)),
                ("pre".into(), number(*pre as u128)),
            ]))
        }),
        "delta_memory_json",
    )
}
fn trace_record(step: u64, point: &StepPoint, delta: &SnapshotDelta) -> Result<Json> {
    Ok(object([
        ("block".into(), string(point.block.clone())),
        ("function".into(), string(point.function.clone())),
        ("index".into(), number(point.index as u128)),
        (
            "lhs".into(),
            Json::Array(point.instruction.lhs.iter().cloned().map(string).collect()),
        ),
        ("line".into(), number(point.instruction.line as u128)),
        ("memory".into(), Json::Array(delta_memory_json(delta)?)),
        ("op".into(), string(point.instruction.op.clone())),
        (
            "operands".into(),
            Json::Array(
                point
                    .instruction
                    .operands
                    .iter()
                    .cloned()
                    .map(string)
                    .collect(),
            ),
        ),
        (
            "registers".into(),
            Json::Array(delta_registers_json(delta)?),
        ),
        ("step".into(), number(step as u128)),
    ]))
}
/// Drives a machine to fault/finish (or `max_steps` executed instructions),
/// answering HOST_REQUEST suspensions from the reply queue in order, with
/// each reply checked against the actual service and payload (§10). When
/// `trace` is given, one record per instruction is appended; a hostcall's
/// record includes its reply's register and memory deltas.
fn drive(
    machine: &mut ClassicalMachine,
    replies: &mut VecDeque<Reply>,
    fuel: u64,
    max_steps: Option<u64>,
    mut trace: Option<&mut Vec<Json>>,
) -> Result<u64> {
    let mut steps = 0u64;
    loop {
        if let Some(request) = machine.outcome().request {
            let reply = replies.pop_front().ok_or_else(|| {
                fail(format!(
                    "§10: HOST_REQUEST for service {} has no reply; --replies is exhausted (run left suspended, no result)",
                    request.service
                ))
            })?;
            if reply.service != request.service || request.payload != reply.payload {
                let mut message = String::new();
                for part in [
                    "§10: reply mismatch: expected request service=",
                    &request.service,
                    " payload=",
                    &hex(&request.payload)?,
                    ", replies file next names service=",
                    &reply.service,
                    " payload=",
                    &hex(&reply.payload)?,
                ] {
                    message.try_reserve(part.len()).map_err(|e| {
                        bennett_isa_ref::Error::host_resource(format!(
                            "§10: host limit: reply mismatch diagnostic: {e}"
                        ))
                    })?;
                    message.push_str(part);
                }
                return Err(Error::new(message));
            }
            let before = trace.is_some().then(|| machine.snapshot());
            machine.reply(&reply.reply)?;
            if let (Some(trace), Some(before)) = (trace.as_deref_mut(), before) {
                let delta = before.delta(&machine.snapshot())?;
                if let Some(Json::Object(record)) = trace.last_mut() {
                    if let Some(Json::Array(registers)) = record.get_mut("registers") {
                        let more = delta_registers_json(&delta)?;
                        registers.try_reserve(more.len()).map_err(|e| {
                            Error::host_resource(format!(
                                "§7.4: host limit: trace register records: {e}"
                            ))
                        })?;
                        registers.extend(more);
                    }
                    if let Some(Json::Array(memory)) = record.get_mut("memory") {
                        let more = delta_memory_json(&delta)?;
                        memory.try_reserve(more.len()).map_err(|e| {
                            Error::host_resource(format!(
                                "§7.4: host limit: trace memory records: {e}"
                            ))
                        })?;
                        memory.extend(more);
                    }
                }
            }
            continue;
        }
        if !machine.is_running() {
            break;
        }
        if max_steps.is_some_and(|max| steps >= max) {
            break;
        }
        if steps >= fuel {
            return Err(fail(
                "§3.4: reference execution fuel exhausted; CC/R has no ISA step budget",
            ));
        }
        if let Some(trace) = trace.as_deref_mut() {
            trace.try_reserve(1).map_err(|e| {
                Error::host_resource(format!("§7.4: host limit: CLI trace records: {e}"))
            })?;
        }
        let before = trace.is_some().then(|| machine.snapshot());
        let point = machine
            .step_once()?
            .ok_or_else(|| fail("§3.4: running machine took no step"))?;
        steps += 1;
        if let (Some(trace), Some(before)) = (trace.as_deref_mut(), before) {
            let delta = before.delta(&machine.snapshot())?;
            trace.push(trace_record(steps - 1, &point, &delta)?);
        }
    }
    Ok(steps)
}
fn check_replies_consumed(replies: &VecDeque<Reply>) -> Result<()> {
    if replies.is_empty() {
        Ok(())
    } else {
        Err(fail(format!(
            "§10: {} repl{} in the --replies file never consumed by a HOST_REQUEST",
            replies.len(),
            if replies.len() == 1 { "y" } else { "ies" }
        )))
    }
}
fn classical_entry<'a>(program: &'a Object, name: &str) -> Result<&'a Function> {
    program
        .functions
        .get(name)
        .filter(|f| matches!(f.kind.as_str(), "cc" | "r"))
        .ok_or_else(|| fail(format!("§3.4: unknown CC/R entry @{name}")))
}
/// The §5.4 tokens among the bound arguments, in argument order.
fn budgets_json(args: &[ClassicalValue]) -> Json {
    Json::Array(
        args.iter()
            .filter_map(|a| match a {
                ClassicalValue::Token(b) => Some(object([
                    ("depth".into(), number(b.depth)),
                    ("heap".into(), number(b.heap)),
                    ("stack".into(), number(b.stack)),
                    ("steps".into(), number(b.steps as u128)),
                ])),
                _ => None,
            })
            .collect(),
    )
}
fn classical_machine<'a>(
    program: &'a Object,
    name: &str,
    options: &CcOptions,
) -> Result<(ClassicalMachine<'a>, Json)> {
    let entry = classical_entry(program, name)?;
    let (args, registers) = classical_args(program, entry, &options.args, options.token_defaults)?;
    let budgets = budgets_json(&args);
    let mut machine = ClassicalMachine::with_quantum_registers_and_globals(
        program,
        name,
        &args,
        &options.entropy,
        registers,
        &options.mem,
    )?;
    for draw in &options.draws {
        machine.push_measurement_draw(*draw)?;
    }
    Ok((machine, budgets))
}
fn run_cc(program: &Object, name: &str, options: CcOptions) -> Result<Json> {
    let mut replies = load_replies(options.replies.as_deref())?;
    let (mut machine, budgets) = classical_machine(program, name, &options)?;
    let mut trace = Vec::new();
    let steps = drive(
        &mut machine,
        &mut replies,
        options.fuel,
        None,
        Some(&mut trace),
    )?;
    check_replies_consumed(&replies)?;
    let outcome = machine.outcome();
    Ok(object([
        ("budgets".into(), budgets),
        ("clock".into(), number(machine.clock() as u128)),
        ("console".into(), console_json(&machine)?),
        ("console_hex".into(), string(hex(machine.console())?)),
        (
            "effect_tape_length".into(),
            number(machine.effect_tape_len() as u128),
        ),
        ("entry".into(), string(name)),
        (
            "fault".into(),
            outcome.fault.map(number).unwrap_or(Json::Null),
        ),
        ("faults".into(), faults_json(outcome.fault)),
        ("host_requests".into(), host_requests_json(&machine)?),
        ("measurements".into(), measurements_json(&machine)),
        ("memory_digest".into(), string(machine.memory_digest()?)),
        ("results".into(), values_json(&outcome.values)?),
        (
            "status".into(),
            string(if outcome.fault.is_some() {
                "FAILED"
            } else {
                "COMPLETED"
            }),
        ),
        ("steps".into(), number(steps as u128)),
        ("tape".into(), tape_json(&machine)?),
        ("trace".into(), Json::Array(trace)),
    ]))
}
/// Options for executing one Q action (`run-action`, and `reverse` on an
/// action). The interface bit order for `--basis`, `--state` and the output
/// state is: mut bundles then in bundles in declared order, then mem
/// regions in declared order with cells in order; within any bundle or
/// cell wire 0 comes first (bundles are little-endian, §14.2).
struct ActionOptions {
    muts: Option<Vec<String>>,
    ins: Option<Vec<String>>,
    basis: Option<String>,
    state: Option<String>,
    mem: Vec<(String, Vec<u8>)>,
    args: Vec<String>,
    adjoint: bool,
}
fn parse_action_options(args: &[String], start: usize) -> Result<ActionOptions> {
    let mut options = ActionOptions {
        muts: None,
        ins: None,
        basis: None,
        state: None,
        mem: Vec::new(),
        args: Vec::new(),
        adjoint: false,
    };
    let mut list_form = false;
    let mut single_form = false;
    let split = |value: &str| -> Vec<String> {
        if value.is_empty() {
            Vec::new()
        } else {
            value.split(',').map(str::to_string).collect()
        }
    };
    let mut pos = start;
    while pos < args.len() {
        let option = args[pos].as_str();
        pos += 1;
        if option == "--adjoint" {
            if options.adjoint {
                return Err(fail("--adjoint given twice"));
            }
            options.adjoint = true;
            continue;
        }
        let value = args
            .get(pos)
            .ok_or_else(|| fail(format!("missing value for {option}")))?;
        pos += 1;
        match option {
            "--mut" => {
                if options.muts.is_some() {
                    return Err(fail("--mut given twice"));
                }
                options.muts = Some(split(value));
            }
            "--in" => {
                if options.ins.is_some() {
                    return Err(fail("--in given twice"));
                }
                options.ins = Some(split(value));
            }
            "--basis" => {
                if options.basis.is_some() {
                    return Err(fail("--basis given twice"));
                }
                options.basis = Some(value.clone());
            }
            "--state" => {
                if options.state.is_some() {
                    return Err(fail("--state given twice"));
                }
                options.state = Some(value.clone());
            }
            "--mem" => {
                let (name, hex) = value
                    .split_once('=')
                    .ok_or_else(|| fail("--mem requires name=hex"))?;
                if options.mem.iter().any(|(n, _)| n == name) {
                    return Err(fail(format!("duplicate --mem {name}")));
                }
                options.mem.push((name.into(), parse_hex(hex)?));
            }
            "--args" => {
                if list_form || single_form {
                    return Err(fail(
                        "--args/--arg: pass the classical arguments exactly once, in one form",
                    ));
                }
                list_form = true;
                options.args = split(value);
            }
            "--arg" => {
                if list_form {
                    return Err(fail(
                        "--args/--arg: pass the classical arguments exactly once, in one form",
                    ));
                }
                single_form = true;
                options.args.push(value.clone());
            }
            _ => return Err(fail(format!("unknown option {option}"))),
        }
    }
    Ok(options)
}
struct ActionRun {
    state: QuantumState,
    bindings: QuantumBindings,
    /// Interface wires in the documented order (mut, in, mem cells).
    interface_wires: Vec<u64>,
    mut_bundles: Vec<Vec<u64>>,
    in_bundles: Vec<Vec<u64>>,
    /// (region name, its cells' wires) in declared order.
    regions: Vec<(String, Vec<Vec<u64>>)>,
    borrowed: Vec<u64>,
}
/// Builds the §5.3 interface bindings for one action execution on a basis
/// state (`--mut`/`--in`/`--mem`, `--basis`) or a sparse state vector
/// (`--state bits=re[,im];...`). Borrowed pool wires start in |0> and are
/// verified restored before any state is reported (§5.6).
fn build_action_state(
    program: &Object,
    f: &Function,
    options: &ActionOptions,
) -> Result<ActionRun> {
    let modes = [
        options.muts.is_some(),
        options.basis.is_some(),
        options.state.is_some(),
    ]
    .iter()
    .filter(|m| **m)
    .count();
    if modes != 1 {
        return Err(fail(
            "§5.3: an action run needs exactly one of --mut, --basis, --state",
        ));
    }
    if options.ins.is_some() && options.muts.is_none() {
        return Err(fail("§5.3: --in accompanies --mut only"));
    }
    if !options.mem.is_empty() && options.muts.is_none() {
        return Err(fail(
            "§5.3: --mem accompanies --mut only; --basis/--state bits already cover the mem cells",
        ));
    }
    let width = |p: &bennett_isa_ref::Param| p.ty.bits(program.ptr_width);
    let mut q = QuantumState::new();
    let mut bindings = QuantumBindings::default();
    let mut interface_wires: Vec<u64> = Vec::new();
    let mut mut_bundles = Vec::new();
    let mut in_bundles = Vec::new();
    for (group, bundles) in [(0usize, &mut mut_bundles), (1, &mut in_bundles)] {
        for p in &f.params[group] {
            let wires = q.allocate(width(p)?)?;
            bindings.bundles.insert(p.name.clone(), wires.clone());
            interface_wires.extend(&wires);
            bundles.push(wires);
        }
    }
    let mut regions = Vec::new();
    let mut region_cell_values: Vec<Vec<BitVector>> = Vec::new();
    for p in &f.params[2] {
        let Type::Region(a, w) = p.ty else {
            return Err(fail("§5.1: mem slot requires region type"));
        };
        let cells = (a < 63)
            .then(|| 1usize.checked_shl(a as u32))
            .flatten()
            .ok_or_else(|| {
                Error::host_resource(
                    "§3.3: host limit: interface region exceeds host memory".into(),
                )
            })?;
        let cell_width = usize::try_from(w).map_err(|_| {
            Error::host_resource("§3.3: host limit: cell width exceeds host storage".into())
        })?;
        let cell_bytes = cell_width.div_ceil(8);
        let byte_count = cells.checked_mul(cell_bytes).ok_or_else(|| {
            Error::host_resource(
                "§3.3: host limit: interface region byte count exceeds usize".into(),
            )
        })?;
        let mut values = Vec::new();
        values.try_reserve_exact(cells).map_err(|e| {
            Error::host_resource(format!("§3.3: host limit: interface region cells: {e}"))
        })?;
        for _ in 0..cells {
            values.push(BitVector::zero(cell_width)?);
        }
        if let Some((_, bytes)) = options.mem.iter().find(|(n, _)| *n == p.name) {
            if bytes.len() != byte_count {
                return Err(fail(format!(
                    "§5.1: region {} needs {} bytes ({} cells of {}), --mem gave {}",
                    p.name,
                    byte_count,
                    cells,
                    cell_bytes,
                    bytes.len()
                )));
            }
            for (i, chunk) in bytes.chunks_exact(cell_bytes).enumerate() {
                values[i] = BitVector::from_le_bytes(cell_width, chunk)?;
            }
        }
        let mut all = Vec::new();
        all.try_reserve_exact(cells).map_err(|e| {
            Error::host_resource(format!("§5.1: host limit: CLI region cells: {e}"))
        })?;
        for _ in 0..cells {
            let cell = q.allocate(cell_width)?;
            interface_wires.extend(&cell);
            all.push(cell);
        }
        bindings.regions.insert(
            p.name.clone(),
            QuantumRegion {
                address_width: a,
                cell_width: w,
                cells: all.clone(),
            },
        );
        regions.push((p.name.clone(), all));
        region_cell_values.push(values);
    }
    for (name, _) in &options.mem {
        if !f.params[2].iter().any(|p| &p.name == name) {
            return Err(fail(format!(
                "§5.1: action @{} has no mem region {name}",
                f.name
            )));
        }
    }
    if options.args.len() != f.params[3].len() {
        return Err(fail(format!(
            "§5.1: action @{} takes {} classical arguments, {} given",
            f.name,
            f.params[3].len(),
            options.args.len()
        )));
    }
    let defaults = program.default_budget(1_000_000)?;
    for (p, raw) in f.params[3].iter().zip(&options.args) {
        let arg = if let Some(s) = raw.strip_prefix("token:") {
            Argument::Token(budget(s, defaults)?)
        } else if let Some(s) = raw.strip_prefix("f64:") {
            Argument::Float(s.parse().map_err(fail)?)
        } else {
            Argument::Bits(uint(raw)?)
        };
        bindings.args.insert(p.name.clone(), arg);
    }
    let borrowed = q.allocate(program.borrow_capacity(&f.name)?)?;
    if !borrowed.is_empty() {
        bindings.borrowed.push(borrowed.clone());
    }
    let total = interface_wires.len();
    if let Some(muts) = &options.muts {
        if muts.len() != f.params[0].len() {
            return Err(fail(format!(
                "§5.1: --mut needs {} values for @{}, {} given",
                f.params[0].len(),
                f.name,
                muts.len()
            )));
        }
        let empty: Vec<String> = Vec::new();
        let ins = options.ins.as_ref().unwrap_or(&empty);
        if options.ins.is_some() && ins.len() != f.params[1].len() {
            return Err(fail(format!(
                "§5.1: --in needs {} values for @{}, {} given",
                f.params[1].len(),
                f.name,
                ins.len()
            )));
        }
        let prepare = |q: &mut QuantumState, wires: &[u64], value: &BitVector| -> Result<()> {
            for (i, w) in wires.iter().enumerate() {
                if value.bit(i)? {
                    q.gate("x", &[*w], None, &[])?;
                }
            }
            Ok(())
        };
        for (raw, p) in muts.iter().zip(&f.params[0]) {
            let value = vector_input(width(p)?, raw)?;
            prepare(&mut q, &bindings.bundles[&p.name].clone(), &value)?;
        }
        for (raw, p) in ins.iter().zip(&f.params[1]) {
            let value = vector_input(width(p)?, raw)?;
            prepare(&mut q, &bindings.bundles[&p.name].clone(), &value)?;
        }
        for ((_, cells), values) in regions.iter().zip(&region_cell_values) {
            for (cell, value) in cells.iter().zip(values) {
                prepare(&mut q, &cell.clone(), value)?;
            }
        }
    } else {
        let components: Vec<(Vec<bool>, Complex)> = if let Some(basis) = &options.basis {
            vec![(parse_bits(basis, total)?, Complex::ONE)]
        } else {
            parse_state_spec(options.state.as_deref().expect("mode"), total)?
        };
        let full: Vec<(Vec<bool>, Complex)> = components
            .into_iter()
            .map(|(mut bits, amp)| {
                bits.resize(total + borrowed.len(), false);
                (bits, amp)
            })
            .collect();
        q.set_state_vector(&full)?;
    }
    Ok(ActionRun {
        state: q,
        bindings,
        interface_wires,
        mut_bundles,
        in_bundles,
        regions,
        borrowed,
    })
}
fn parse_bits(text: &str, total: usize) -> Result<Vec<bool>> {
    if text.len() != total || !text.bytes().all(|b| matches!(b, b'0' | b'1')) {
        return Err(fail(format!(
            "§14.2: bit string needs exactly {total} characters of 0/1, got {text:?}"
        )));
    }
    Ok(text.bytes().map(|b| b == b'1').collect())
}
fn parse_state_spec(text: &str, total: usize) -> Result<Vec<(Vec<bool>, Complex)>> {
    let mut components = Vec::new();
    for entry in text.split(';') {
        let (bits, amp) = entry.split_once('=').ok_or_else(|| {
            fail(format!(
                "§14.2: state component {entry:?} needs bits=re[,im]"
            ))
        })?;
        let mut parts = amp.split(',');
        let re: f64 = parts
            .next()
            .expect("split")
            .parse()
            .map_err(|_| fail(format!("§14.2: invalid amplitude in {entry:?}")))?;
        let im: f64 = match parts.next() {
            Some(s) => s
                .parse()
                .map_err(|_| fail(format!("§14.2: invalid amplitude in {entry:?}")))?,
            None => 0.0,
        };
        if parts.next().is_some() || !re.is_finite() || !im.is_finite() {
            return Err(fail(format!("§14.2: invalid amplitude in {entry:?}")));
        }
        components.push((parse_bits(bits, total)?, Complex::new(re, im)));
    }
    if components.is_empty() {
        return Err(fail("§14.2: --state needs at least one component"));
    }
    Ok(components)
}
/// The final state over the interface wires as `{bits: [re, im]}`, one
/// entry per exactly nonzero component, amplitudes the binary64 values of
/// the exact shadow. Borrowed pool wires must be back in |0> on every
/// component (§5.6) or the run is an error, never a partial state.
fn action_state_json(run: &ActionRun) -> Result<Json> {
    let total = run.interface_wires.len();
    if run.state.wire_count() != total + run.borrowed.len() {
        return Err(fail(
            "§5.3: action left wires beyond its interface and borrow pool",
        ));
    }
    let mut entries = Vec::new();
    for (bits, (re, im)) in run.state.exact_support() {
        if bits[total..].iter().any(|b| *b) {
            return Err(fail(
                "§5.6: borrowed wires were not returned to their entry state",
            ));
        }
        let key: String = bits[..total]
            .iter()
            .map(|b| if *b { '1' } else { '0' })
            .collect();
        entries.push((key, Json::Array(vec![float_json(re)?, float_json(im)?])));
    }
    Ok(object(entries))
}
/// The definite basis value of one bundle, bit by bit, or None when some
/// wire is not in a basis state on every component.
fn bundle_value(state: &QuantumState, wires: &[u64]) -> Option<BitVector> {
    let mut value = BitVector::zero(wires.len()).ok()?;
    for (i, w) in wires.iter().enumerate() {
        value.set(i, state.read_basis(&[*w]).ok()? != 0).ok()?;
    }
    Some(value)
}
fn bundles_json(state: &QuantumState, bundles: &[Vec<u64>]) -> Json {
    Json::Array(
        bundles
            .iter()
            .map(|wires| {
                bundle_value(state, wires)
                    .as_ref()
                    .map(vector_number)
                    .unwrap_or(Json::Null)
            })
            .collect(),
    )
}
fn first_state_difference(a: &QuantumState, b: &QuantumState, total: usize) -> Option<String> {
    let support = |s: &QuantumState| {
        s.exact_support()
            .into_iter()
            .map(|(bits, amp)| {
                (
                    bits[..bits.len().min(total)]
                        .iter()
                        .map(|b| if *b { '1' } else { '0' })
                        .collect::<String>(),
                    amp,
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    let (sa, sb) = (support(a), support(b));
    for key in sa
        .keys()
        .chain(sb.keys())
        .collect::<std::collections::BTreeSet<_>>()
    {
        match (sa.get(key), sb.get(key)) {
            (Some(x), Some(y)) if x == y => {}
            _ => {
                return Some(format!(
                    "state component |{key}>: {:?} differs from {:?}",
                    sa.get(key),
                    sb.get(key)
                ))
            }
        }
    }
    None
}
fn reverse_cc(program: &Object, name: &str, options: CcOptions) -> Result<Json> {
    let mut replies = load_replies(options.replies.as_deref())?;
    let (mut machine, budgets) = classical_machine(program, name, &options)?;
    let entry = machine.snapshot();
    let entry_digest = machine.memory_digest()?;
    let steps = drive(
        &mut machine,
        &mut replies,
        options.fuel,
        options.steps,
        None,
    )?;
    check_replies_consumed(&replies)?;
    let forward = machine.outcome();
    let forward_tape = tape_json(&machine)?;
    let forward_requests = host_requests_json(&machine)?;
    // §10 reverse eligibility is enforced by the machine (measurement in
    // history is E_NOT_REVERSIBLE; a pending request never reaches here).
    machine.reverse()?;
    let after = machine.snapshot();
    let equals = after == entry;
    Ok(object([
        ("budgets".into(), budgets),
        ("clock".into(), number(machine.clock() as u128)),
        ("console".into(), console_json(&machine)?),
        ("console_hex".into(), string(hex(machine.console())?)),
        (
            "effect_tape_length".into(),
            number(machine.effect_tape_len() as u128),
        ),
        ("entry".into(), string(name)),
        ("entry_memory_digest".into(), string(entry_digest)),
        ("equals_entry".into(), Json::Bool(equals)),
        (
            "fault".into(),
            forward.fault.map(number).unwrap_or(Json::Null),
        ),
        ("faults".into(), faults_json(forward.fault)),
        (
            "first_difference".into(),
            entry
                .first_difference(&after)
                .map(string)
                .unwrap_or(Json::Null),
        ),
        ("host_requests".into(), forward_requests),
        ("memory_digest".into(), string(machine.memory_digest()?)),
        (
            "memory_equals_entry".into(),
            Json::Bool(entry.memory_equals(&after)),
        ),
        ("results".into(), values_json(&forward.values)?),
        (
            "status".into(),
            string(if equals { "REVERSED" } else { "FAILED" }),
        ),
        ("steps".into(), number(steps as u128)),
        ("tape".into(), forward_tape),
    ]))
}
/// The run-oracle-shaped options for `reverse` on an oracle.
struct OracleOptions {
    input: OracleVectorInput,
    limits: Budget,
    steps: Option<u64>,
}
fn parse_oracle_options(
    program: &Object,
    name: &str,
    args: &[String],
    start: usize,
) -> Result<OracleOptions> {
    let oracle = program
        .functions
        .get(name)
        .filter(|f| f.kind == "oracle")
        .ok_or_else(|| fail(format!("§5.2: unknown oracle @{name}")))?;
    let mut limits = program.default_budget(1_000_000)?;
    let mut vectors: Vec<BitVector> = Vec::new();
    let mut input_seen = false;
    let mut regions: std::collections::BTreeMap<String, Vec<u8>> = Default::default();
    let mut arguments: Vec<Argument> = Vec::new();
    let mut steps = None;
    let mut pos = start;
    while pos < args.len() {
        let option = args[pos].as_str();
        pos += 1;
        let value = args
            .get(pos)
            .ok_or_else(|| fail(format!("missing value for {option}")))?;
        pos += 1;
        match option {
            "--input" => {
                if input_seen {
                    return Err(fail("--input given twice"));
                }
                input_seen = true;
                let values: Vec<&str> = if value.is_empty() {
                    Vec::new()
                } else {
                    value.split(',').collect()
                };
                if values.len() != oracle.params[0].len() {
                    return Err(fail("§5.1: oracle input count differs from signature"));
                }
                vectors = values
                    .iter()
                    .zip(&oracle.params[0])
                    .map(|(raw, p)| vector_input(p.ty.width(program.ptr_width)? as usize, raw))
                    .collect::<Result<_>>()?;
            }
            "--budget" => limits = budget(value, limits)?,
            "--steps" => {
                steps = Some(uint(value).and_then(|n| u64::try_from(n).map_err(fail))?);
            }
            "--mem" => {
                let (region, hex) = value
                    .split_once('=')
                    .ok_or_else(|| fail("--mem requires name=hex"))?;
                if regions.insert(region.into(), parse_hex(hex)?).is_some() {
                    return Err(fail("duplicate --mem region"));
                }
            }
            "--arg" => {
                let arg = if let Some(s) = value.strip_prefix("token:") {
                    Argument::Token(budget(s, limits)?)
                } else if let Some(s) = value.strip_prefix("f64:") {
                    Argument::Float(s.parse().map_err(fail)?)
                } else {
                    Argument::Bits(uint(value)?)
                };
                arguments.push(arg);
            }
            _ => return Err(fail(format!("unknown option {option}"))),
        }
    }
    if !input_seen && !oracle.params[0].is_empty() {
        return Err(fail("§5.1: oracle input count differs from signature"));
    }
    for p in &oracle.params[3] {
        if !regions.contains_key(&p.name) {
            let Type::Region(a, _) = p.ty else {
                return Err(fail("§5.1: mem slot requires region type"));
            };
            let n = (a < 64)
                .then(|| 1usize.checked_shl(a as u32))
                .flatten()
                .ok_or_else(|| {
                    Error::host_resource(
                        "§3.3: host limit: interface region exceeds host memory".into(),
                    )
                })?;
            let mut zero = Vec::new();
            zero.try_reserve_exact(n).map_err(|e| {
                Error::host_resource(format!("§3.3: host limit: interface region storage: {e}"))
            })?;
            zero.resize(n, 0);
            regions.insert(p.name.clone(), zero);
        }
    }
    Ok(OracleOptions {
        input: OracleVectorInput {
            ins: vectors,
            args: arguments,
            regions,
        },
        limits,
        steps,
    })
}
fn reverse_oracle(program: &Object, name: &str, options: OracleOptions) -> Result<Json> {
    let limits = options.limits;
    let prefix = options.steps.unwrap_or(limits.steps);
    if prefix > limits.steps {
        return Err(fail(format!(
            "§5.4: reverse prefix of {prefix} steps exceeds budget B={} (the machine retains B frames, §3.6)",
            limits.steps
        )));
    }
    let mut machine = OracleVectorMachine::new(program, name, &options.input, limits)?;
    let entry = machine.snapshot();
    for _ in 0..prefix {
        machine.forward_step()?;
    }
    let outcome = machine.outcome()?;
    for _ in 0..prefix {
        machine.inverse_step()?;
    }
    let after = machine.snapshot();
    let equals = after == entry;
    let locations = object(
        after
            .locations()
            .into_iter()
            .map(|((f, n), v)| (format!("{f}.{n}"), vector_number(&v))),
    );
    let state = object([
        (
            "counters".into(),
            Json::Array(after.counter_values().iter().map(vector_number).collect()),
        ),
        ("faulted".into(), Json::Bool(after.faulted)),
        ("locations".into(), locations),
        ("pc".into(), number(after.pc as u128)),
        ("pc_prime".into(), number(after.pc_prime as u128)),
    ]);
    Ok(object([
        (
            "budget".into(),
            object([
                ("depth".into(), number(limits.depth)),
                ("heap".into(), number(limits.heap)),
                ("stack".into(), number(limits.stack)),
                ("steps".into(), number(limits.steps as u128)),
            ]),
        ),
        ("code".into(), vector_number(&outcome.code)),
        ("equals_entry".into(), Json::Bool(equals)),
        (
            "first_difference".into(),
            entry
                .first_difference(&after)
                .map(string)
                .unwrap_or(Json::Null),
        ),
        ("forward_steps".into(), number(prefix as u128)),
        ("frames_clean".into(), Json::Bool(machine.frames_clean())),
        ("oracle".into(), string(name)),
        ("payload".into(), vector_number(&outcome.payload)),
        ("reverse_steps".into(), number(prefix as u128)),
        ("state_after_reversal".into(), state),
        (
            "status".into(),
            string(if equals { "REVERSED" } else { "FAILED" }),
        ),
        ("steps".into(), number(outcome.steps as u128)),
    ]))
}
fn reverse_action(program: &Object, name: &str, options: ActionOptions) -> Result<Json> {
    let f = program
        .functions
        .get(name)
        .filter(|f| matches!(f.kind.as_str(), "action" | "perm_action"))
        .ok_or_else(|| fail(format!("§5.3: unknown action @{name}")))?;
    let mut run = build_action_state(program, f, &options)?;
    let entry_state = run.state.clone();
    program.execute_action(name, &mut run.state, &run.bindings, options.adjoint, &[])?;
    let forward_state = action_state_json(&run)?;
    program.execute_action(name, &mut run.state, &run.bindings, !options.adjoint, &[])?;
    let state = action_state_json(&run)?;
    let equals = run.state == entry_state;
    Ok(object([
        ("action".into(), string(name)),
        ("adjoint".into(), Json::Bool(options.adjoint)),
        ("equals_entry".into(), Json::Bool(equals)),
        (
            "first_difference".into(),
            if equals {
                Json::Null
            } else {
                first_state_difference(&entry_state, &run.state, run.interface_wires.len())
                    .map(string)
                    .unwrap_or(Json::Null)
            },
        ),
        ("forward_state".into(), forward_state),
        ("in".into(), bundles_json(&run.state, &run.in_bundles)),
        ("mut".into(), bundles_json(&run.state, &run.mut_bundles)),
        ("state".into(), state),
        (
            "status".into(),
            string(if equals { "REVERSED" } else { "FAILED" }),
        ),
        ("wires".into(), number(run.interface_wires.len() as u128)),
    ]))
}
fn reverse(program: &Object, name: &str, args: &[String]) -> Result<Json> {
    let f = program
        .functions
        .get(name)
        .ok_or_else(|| fail(format!("§5.1: unknown symbol @{name}")))?;
    match f.kind.as_str() {
        "cc" | "r" => {
            let options = parse_cc_options(program, "reverse", args, 3)?;
            reverse_cc(program, name, options)
        }
        "oracle" => {
            let options = parse_oracle_options(program, name, args, 3)?;
            reverse_oracle(program, name, options)
        }
        "action" | "perm_action" => {
            let options = parse_action_options(args, 3)?;
            reverse_action(program, name, options)
        }
        other => Err(fail(format!(
            "§5.1: reverse does not apply to {other} symbol @{name}"
        ))),
    }
}
fn run_action(program: &Object, name: &str, options: ActionOptions) -> Result<Json> {
    let f = program
        .functions
        .get(name)
        .filter(|f| matches!(f.kind.as_str(), "action" | "perm_action"))
        .ok_or_else(|| fail(format!("§5.3: unknown action @{name}")))?;
    let mut run = build_action_state(program, f, &options)?;
    program.execute_action(name, &mut run.state, &run.bindings, options.adjoint, &[])?;
    let state = action_state_json(&run)?;
    let mem = object(run.regions.iter().map(|(region, cells)| {
        (
            region.clone(),
            Json::Array(
                cells
                    .iter()
                    .map(|cell| {
                        bundle_value(&run.state, cell)
                            .as_ref()
                            .map(vector_number)
                            .unwrap_or(Json::Null)
                    })
                    .collect(),
            ),
        )
    }));
    Ok(object([
        ("action".into(), string(name)),
        ("adjoint".into(), Json::Bool(options.adjoint)),
        ("in".into(), bundles_json(&run.state, &run.in_bundles)),
        ("mem".into(), mem),
        ("mut".into(), bundles_json(&run.state, &run.mut_bundles)),
        ("state".into(), state),
        ("wires".into(), number(run.interface_wires.len() as u128)),
    ]))
}
/// One-shot §10 launch: bennett_load (object parsed and hash-checked by the
/// caller), positional bennett_run of a device CC entry with copied
/// arguments and entropy, HOST_REQUEST/reply through the checked reply
/// queue, bennett_result_* reporting, optional bennett_reverse, and
/// bennett_run_close. Paths that need staged deliverables 8-11 (PROGRESS)
/// fail by name before any run is reported.
fn run_program(program: &Object, name: &str, options: CcOptions) -> Result<Json> {
    if options.target.is_some() {
        return Err(fail(
            "§10 bennett_load: a target description cannot be honoured yet (staged deliverable 9: §8 target descriptions, suite E; PROGRESS.md deliverables 8-11); no run is reported",
        ));
    }
    if program.section("schedule").is_some() {
        return Err(fail(
            "§10 bennett_load: the object's schedule cannot be executed verbatim yet (staged deliverable 10: §11 schedule consumer, suite H; PROGRESS.md deliverables 8-11); no run is reported",
        ));
    }
    let f = program
        .functions
        .get(name)
        .ok_or_else(|| earg(format!("unknown CC entry @{name}")))?;
    if f.kind != "cc" {
        return Err(earg(format!(
            "entry @{name} has kind {}; a launch executes a device CC entry",
            f.kind
        )));
    }
    let (args, registers) = classical_args(program, f, &options.args, options.token_defaults)?;
    let budgets = budgets_json(&args);
    let mut machine = ClassicalMachine::with_quantum_registers_and_globals(
        program,
        name,
        &args,
        &options.entropy,
        registers,
        &options.mem,
    )?;
    for draw in &options.draws {
        machine.push_measurement_draw(*draw)?;
    }
    let entry = machine.snapshot();
    let entry_digest = machine.memory_digest()?;
    let mut replies = load_replies(options.replies.as_deref())?;
    let steps = drive(&mut machine, &mut replies, options.fuel, None, None)?;
    check_replies_consumed(&replies)?;
    let outcome = machine.outcome();
    let failed = outcome.fault.is_some();
    // §10: a FAILED run exposes no values; its fault records are the result.
    let results = if failed {
        Json::Array(Vec::new())
    } else {
        values_json(&outcome.values)?
    };
    let forward_tape = tape_json(&machine)?;
    let forward_requests = host_requests_json(&machine)?;
    let forward_measurements = measurements_json(&machine);
    let mut status = if failed { "FAILED" } else { "COMPLETED" };
    let mut reversed = false;
    if options.reverse_flag {
        if failed {
            return Err(fail(
                "§10: E_INVALID_STATE: bennett_reverse applies to a COMPLETED run; this run FAILED",
            ));
        }
        // Eligibility (no measure, untouched registers) is the machine's:
        // a measured run fails here with §10 E_NOT_REVERSIBLE.
        machine.reverse()?;
        reversed = true;
        status = "REVERSED";
    }
    let after = machine.snapshot();
    Ok(object([
        ("budgets".into(), budgets),
        ("clock".into(), number(machine.clock() as u128)),
        ("console".into(), console_json(&machine)?),
        ("console_hex".into(), string(hex(machine.console())?)),
        (
            "effect_tape_length".into(),
            number(machine.effect_tape_len() as u128),
        ),
        ("entry".into(), string(name)),
        ("entry_memory_digest".into(), string(entry_digest)),
        ("equals_entry".into(), Json::Bool(after == entry)),
        (
            "fault".into(),
            outcome.fault.map(number).unwrap_or(Json::Null),
        ),
        ("faults".into(), faults_json(outcome.fault)),
        (
            "first_difference".into(),
            entry
                .first_difference(&after)
                .map(string)
                .unwrap_or(Json::Null),
        ),
        ("host_requests".into(), forward_requests),
        ("measurements".into(), forward_measurements),
        ("memory_digest".into(), string(machine.memory_digest()?)),
        (
            "memory_equals_entry".into(),
            Json::Bool(entry.memory_equals(&after)),
        ),
        ("results".into(), results),
        ("reversed".into(), Json::Bool(reversed)),
        ("run_state".into(), string("CLOSED")),
        ("status".into(), string(status)),
        ("steps".into(), number(steps as u128)),
        ("tape".into(), forward_tape),
    ]))
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return Err(fail(USAGE));
    }
    if matches!(args[0].as_str(), "--help" | "help") {
        println!("{USAGE}");
        return Ok(());
    }
    let command = args[0].as_str();
    if ![
        "check",
        "hash",
        "run-oracle",
        "step-check",
        "run-cc",
        "reverse",
        "run-action",
        "run-program",
    ]
    .contains(&command)
    {
        return Err(fail(format!("unknown subcommand {}\n{USAGE}", args[0])));
    }
    let path = args.get(1).ok_or_else(|| fail(USAGE))?;
    let text = std::fs::read_to_string(path).map_err(|e| fail(format!("{path}: {e}")))?;
    let program = Object::parse(&text)?;
    if matches!(command, "check" | "hash") {
        if args.len() != 2 {
            return Err(fail(USAGE));
        }
        if command == "check" {
            program.check_hashes()?;
        }
        let identity = program.identity()?;
        let symbols = object(identity.interface_hashes.into_iter().map(|(name, hash)| {
            let closure = identity.closure_hashes[&name]
                .clone()
                .map(string)
                .unwrap_or(Json::Null);
            (
                name,
                object([
                    ("closure_hash".into(), closure),
                    ("interface_hash".into(), string(hash)),
                ]),
            )
        }));
        // An object with unresolved imports is valid text but not executable
        // until linked (§9.3): say so instead of a bare "ok".
        let unresolved: Vec<Json> = program
            .section("imports")
            .map(|i| i.object().map(|o| o.keys().cloned().map(string).collect()))
            .transpose()?
            .unwrap_or_default();
        let status = if unresolved.is_empty() {
            "ok"
        } else {
            "unlinked"
        };
        println!(
            "{}",
            object([
                ("object_hash".into(), string(identity.object_hash)),
                ("status".into(), string(status)),
                ("symbols".into(), symbols),
                ("unresolved_imports".into(), Json::Array(unresolved)),
            ])
            .canonical()
        );
        return Ok(());
    }
    if matches!(command, "run-cc" | "reverse" | "run-action" | "run-program") {
        // Give CLI/IO/JSON input errors the relevant ISA context; preserve
        // more specific citations from the library. Print only a complete
        // result, so every error exits without partial JSON.
        let rule = match command {
            "run-action" => "§5.3",
            "reverse" => "§14.1 / §7.4 / §10",
            _ => "§10",
        };
        let output = (|| -> Result<Json> {
            let name = args
                .get(2)
                .ok_or_else(|| fail(USAGE))?
                .trim_start_matches('@');
            program.check_hashes()?;
            match command {
                "run-program" => {
                    let options = parse_cc_options(&program, command, &args, 3)?;
                    run_program(&program, name, options)
                }
                "run-action" => {
                    let options = parse_action_options(&args, 3)?;
                    run_action(&program, name, options)
                }
                "reverse" => reverse(&program, name, &args),
                "run-cc" => {
                    let options = parse_cc_options(&program, command, &args, 3)?;
                    run_cc(&program, name, options)
                }
                _ => unreachable!(),
            }
        })()
        .map_err(|e| {
            if e.0.contains('§') {
                e
            } else {
                e.context(rule)
            }
        })?;
        println!("{}", output.try_canonical()?);
        return Ok(());
    }
    let name = args
        .get(2)
        .ok_or_else(|| fail(USAGE))?
        .trim_start_matches('@');
    let oracle = program
        .functions
        .get(name)
        .filter(|f| f.kind == "oracle")
        .ok_or_else(|| fail(format!("§5.2: unknown oracle @{name}")))?;
    program.check_hashes()?;
    let mut input = OracleInput::default();
    let mut vectors = Vec::new();
    let mut limits = program.default_budget(if command == "step-check" {
        256
    } else {
        1_000_000
    })?;
    let mut exhaustive = false;
    let mut suite = false;
    let mut samples: Option<u128> = None;
    let mut seed = [1u64, 2, 3, 4];
    let mut max_bits = 16u128;
    let mut fixed_regions = Vec::new();
    let mut pos = 3;
    while pos < args.len() {
        let option = &args[pos];
        pos += 1;
        if option == "--exhaustive" {
            exhaustive = true;
            continue;
        }
        if option == "--suite" {
            suite = true;
            continue;
        }
        let value = args
            .get(pos)
            .ok_or_else(|| fail(format!("missing value for {option}")))?;
        pos += 1;
        match option.as_str() {
            "--input" => {
                let values = if value.is_empty() {
                    Vec::new()
                } else {
                    value.split(',').collect::<Vec<_>>()
                };
                if values.len() != oracle.params[0].len() {
                    return Err(fail("§5.1: oracle input count differs from signature"));
                }
                if command == "run-oracle" {
                    vectors = values
                        .iter()
                        .zip(&oracle.params[0])
                        .map(|(raw, p)| vector_input(p.ty.width(program.ptr_width)? as usize, raw))
                        .collect::<Result<_>>()?;
                } else {
                    input.ins = values.iter().map(|raw| uint(raw)).collect::<Result<_>>()?;
                }
            }
            "--budget" => limits = budget(value, limits)?,
            "--max-bits" => max_bits = uint(value)?,
            "--samples" => {
                let count =
                    uint(value).map_err(|e| fail(format!("§17.13: invalid sample count: {e}")))?;
                if count == 0 {
                    return Err(fail("§17.13: --samples requires at least one case"));
                }
                samples = Some(count);
            }
            "--seed" => {
                let words = value
                    .split(',')
                    .map(|w| w.parse::<u64>().map_err(fail))
                    .collect::<Result<Vec<_>>>()?;
                if words.len() != 4 || words.iter().all(|w| *w == 0) {
                    return Err(fail("§12.1: --seed needs four u64 words, not all zero"));
                }
                seed = [words[0], words[1], words[2], words[3]];
            }
            "--mem" => {
                let (name, hex) = value
                    .split_once('=')
                    .ok_or_else(|| fail("--mem requires name=hex"))?;
                if hex.len() % 2 != 0 || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
                    return Err(fail(
                        "§16.1: invalid memory HEX (even-length hexadecimal expected)",
                    ));
                }
                let bytes = (0..hex.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(fail))
                    .collect::<Result<Vec<_>>>()?;
                if input.regions.insert(name.into(), bytes).is_some() {
                    return Err(fail("duplicate --mem region"));
                }
                fixed_regions.push(name.to_string());
            }
            "--arg" => {
                let arg = if let Some(s) = value.strip_prefix("token:") {
                    Argument::Token(budget(s, limits)?)
                } else if let Some(s) = value.strip_prefix("f64:") {
                    Argument::Float(s.parse().map_err(fail)?)
                } else {
                    Argument::Bits(uint(value)?)
                };
                input.args.push(arg);
            }
            _ => return Err(fail(format!("unknown option {option}"))),
        }
    }
    for p in &oracle.params[3] {
        if !input.regions.contains_key(&p.name) {
            let Type::Region(a, _) = p.ty else {
                return Err(fail("§5.1: mem slot requires region type"));
            };
            let n = (a < 64)
                .then(|| 1usize.checked_shl(a as u32))
                .flatten()
                .ok_or_else(|| {
                    Error::host_resource(
                        "§3.3: host limit: interface region exceeds host memory".into(),
                    )
                })?;
            let mut zero = Vec::new();
            zero.try_reserve_exact(n).map_err(|e| {
                Error::host_resource(format!("§3.3: host limit: interface region storage: {e}"))
            })?;
            zero.resize(n, 0);
            input.regions.insert(p.name.clone(), zero);
        }
    }
    if command == "run-oracle" {
        if exhaustive {
            return Err(fail("--exhaustive is a step-check option"));
        }
        let wide_input = OracleVectorInput {
            ins: vectors,
            args: input.args,
            regions: input.regions,
        };
        let r = program.run_oracle_vectors(name, &wide_input, limits)?;
        println!(
            "{}",
            object([
                ("payload".into(), vector_number(&r.payload)),
                ("code".into(), vector_number(&r.code)),
                ("steps".into(), number(r.steps as u128))
            ])
            .canonical()
        );
        return Ok(());
    }
    let modes = [exhaustive, suite, samples.is_some()]
        .iter()
        .filter(|m| **m)
        .count();
    if modes != 1 {
        return Err(fail(
            "step-check needs exactly one of --exhaustive, --suite, --samples N",
        ));
    }
    // The domain: every input bundle and every byte of every unfixed interface
    // region (target/fault entry values do not affect the computation).
    let mut components: Vec<usize> = oracle.params[0]
        .iter()
        .map(|p| p.ty.bits(program.ptr_width))
        .collect::<Result<_>>()?;
    let input_count = components.len();
    for p in &oracle.params[3] {
        if !fixed_regions.contains(&p.name) {
            components.extend(std::iter::repeat_n(8, input.regions[&p.name].len()));
        }
    }
    let bits: u128 = components.iter().map(|w| *w as u128).sum();
    // §17 item 13: exhaustive for <= 16 interface bits, 2^16 samples above.
    let sampled = samples.is_some() || (suite && bits > 16);
    let count = samples.unwrap_or(1 << 16);
    if exhaustive && bits > max_bits {
        return Err(fail(format!("§17.13: the exhaustive domain has {bits} bits, above --max-bits {max_bits}; raise --max-bits, or use --suite or --samples N")));
    }
    let mut rng = Xoshiro256(seed);
    let mut current: Vec<BitVector> = components
        .iter()
        .map(|w| BitVector::zero(*w))
        .collect::<Result<_>>()?;
    let mut cases = 0u128;
    loop {
        if sampled {
            if cases == count {
                break;
            }
            for (v, w) in current.iter_mut().zip(&components) {
                *v = rng.bits(*w)?;
            }
        }
        let mut vector_input = OracleVectorInput {
            ins: current[..input_count].to_vec(),
            args: input.args.clone(),
            regions: input.regions.clone(),
        };
        let mut cursor = input_count;
        for p in &oracle.params[3] {
            if !fixed_regions.contains(&p.name) {
                let bytes = vector_input
                    .regions
                    .get_mut(&p.name)
                    .ok_or_else(|| fail("§5.1: missing interface region"))?;
                for byte in bytes.iter_mut() {
                    *byte = current[cursor].to_u128()? as u8;
                    cursor += 1;
                }
            }
        }
        let label = current
            .iter()
            .map(BitVector::to_decimal)
            .collect::<Vec<_>>()
            .join(",");
        let expected = program.run_oracle_vectors(name, &vector_input, limits)?;
        let mut machine = OracleVectorMachine::new(&program, name, &vector_input, limits)?;
        let before = machine.snapshot();
        for _ in 0..limits.steps {
            machine.forward_step()?;
        }
        if machine.outcome()? != expected {
            return Err(fail(format!(
                "§14.3 T6: disagreement at domain element [{label}]"
            )));
        }
        for _ in 0..limits.steps {
            machine.inverse_step()?;
        }
        if machine.snapshot() != before || !machine.frames_clean() {
            return Err(fail(format!(
                "§14.3 T2: dirty prefix at domain element [{label}]"
            )));
        }
        cases += 1;
        if !sampled {
            // Odometer, last component fastest (§12.2 order).
            let mut i = current.len();
            let mut wrapped = true;
            while i > 0 {
                i -= 1;
                let next = current[i].binary("add", &BitVector::from_u128(components[i], 1)?)?;
                let carry = next.is_zero();
                current[i] = next;
                if !carry {
                    wrapped = false;
                    break;
                }
            }
            if wrapped {
                break;
            }
        }
    }
    let fixed = Json::Array(fixed_regions.into_iter().map(string).collect());
    println!(
        "{}",
        object([
            ("cases".into(), number(cases)),
            ("input_and_memory_bits".into(), number(bits)),
            (
                "mode".into(),
                string(if sampled { "sampled" } else { "exhaustive" })
            ),
            ("fixed_regions".into(), fixed),
            ("status".into(), string("ok")),
            (
                "target_fault".into(),
                string("XOR delivery is independent of their entry values")
            )
        ])
        .canonical()
    );
    Ok(())
}

fn copy_text(text: &str, what: &str) -> Result<String> {
    let mut out = String::new();
    out.try_reserve_exact(text.len())
        .map_err(|e| Error::host_resource(format!("§7.3/§10: host limit: {what}: {e}")))?;
    out.push_str(text);
    Ok(out)
}
fn collect_output<T>(iter: impl Iterator<Item = Result<T>>, what: &str) -> Result<Vec<T>> {
    let mut out = Vec::new();
    for item in iter {
        out.try_reserve(1)
            .map_err(|e| Error::host_resource(format!("§7.4/§10: host limit: {what}: {e}")))?;
        out.push(item?);
    }
    Ok(out)
}
