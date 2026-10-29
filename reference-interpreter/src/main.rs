use bennett_isa_ref::{
    json::Json, Argument, BitVector, Budget, ClassicalMachine, ClassicalValue, Error, Function,
    Object, OracleInput, OracleVectorInput, OracleVectorMachine, QuantumRegisters, Result,
    SnapshotDelta, StepPoint, Type, Xoshiro256,
};
use std::collections::VecDeque;

const USAGE:&str="bennett-isa-ref check <object>\nbennett-isa-ref hash <object>\nbennett-isa-ref run-oracle <object> <oracle> --input <x[,y...]> [--budget B[,heap,stack,depth]] [--mem name=hex] [--arg value]\nbennett-isa-ref step-check <object> <oracle> (--exhaustive [--max-bits N] | --suite | --samples N) [--seed a,b,c,d] [--budget B[,heap,stack,depth]] [--mem name=hex] [--arg value]\nbennett-isa-ref run-cc <object> <entry> [--args <a[,b...]> | --input <a[,b...]> | --arg value...] [--entropy hex] [--replies file] [--draws <d[,e...]>] [--budget B[,heap,stack,depth]] [--mem global=hex]";
fn fail(message: impl std::fmt::Display) -> Error {
    Error(message.to_string())
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
    let v = raw.split(',').map(uint).collect::<Result<Vec<_>>>()?;
    if v.len() != 1 && v.len() != 4 {
        return Err(fail("budget requires B or B,heap,stack,depth (§5.4)"));
    }
    let steps = u64::try_from(v[0]).map_err(fail)?;
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
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn parse_hex(s: &str) -> Result<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(fail("invalid HEX (even-length hexadecimal expected)"));
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(fail))
        .collect()
}
/// A binary64 as a canonical JSON number (RFC 8785 via `Json::parse`).
fn float_json(v: f64) -> Result<Json> {
    if !v.is_finite() {
        return Err(fail("§16.1: non-finite binary64 has no JSON number"));
    }
    Json::parse(&v.to_string())
}
fn signed_decimal(width: u16, v: u128) -> String {
    if width == 128 {
        (v as i128).to_string()
    } else if width > 0 && (v >> (width - 1)) & 1 == 1 {
        format!("-{}", (1u128 << width) - v)
    } else {
        v.to_string()
    }
}
/// One classical value as `{"type","value"}`: integers are signed decimal
/// strings (the §10 INT spelling at any width, §9.1), pointers unsigned
/// decimal strings, f64 a JSON number, token its four §5.4 components.
fn value_json(v: &ClassicalValue) -> Result<Json> {
    Ok(match v {
        ClassicalValue::Int(w, bits) => object([
            ("type".into(), string(format!("i{w}"))),
            ("value".into(), string(signed_decimal(*w, *bits))),
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
    Ok(Json::Array(
        values.iter().map(value_json).collect::<Result<_>>()?,
    ))
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
fn tape_json(m: &ClassicalMachine) -> Json {
    Json::Array(
        m.effect_tape()
            .iter()
            .map(|e| {
                object([
                    (
                        "args".into(),
                        Json::Array(e.args.iter().map(|a| number(*a)).collect()),
                    ),
                    ("bytes".into(), string(hex(&e.bytes))),
                    ("dev".into(), string(e.dev.clone())),
                    ("op".into(), string(e.op.clone())),
                ])
            })
            .collect(),
    )
}
fn measurements_json(m: &ClassicalMachine) -> Json {
    Json::Array(
        m.measurements()
            .into_iter()
            .map(|(reg, bits)| {
                let value = bits
                    .iter()
                    .enumerate()
                    .fold(0u128, |v, (i, b)| v | ((*b as u128) << i));
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
                    ("value".into(), number(value)),
                ])
            })
            .collect(),
    )
}
fn host_requests_json(m: &ClassicalMachine) -> Json {
    Json::Array(
        m.host_transcript()
            .into_iter()
            .map(|e| {
                object([
                    ("address".into(), number(e.request.address)),
                    ("capacity".into(), number(e.request.capacity)),
                    ("payload".into(), string(hex(&e.request.payload))),
                    ("reply".into(), string(hex(&e.reply))),
                    ("service".into(), string(e.request.service.clone())),
                ])
            })
            .collect(),
    )
}
fn console_json(m: &ClassicalMachine) -> Json {
    match std::str::from_utf8(m.console()) {
        Ok(s) => string(s),
        Err(_) => Json::Null,
    }
}
fn earg(message: impl std::fmt::Display) -> Error {
    Error(format!("§10: E_ARG: {message}"))
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
        queue.push_back(Reply {
            service: entry.get("service")?.string()?.into(),
            payload: parse_hex(entry.get("payload")?.string()?)?,
            reply: parse_hex(entry.get("reply")?.string()?)?,
        });
    }
    Ok(queue)
}
fn delta_registers_json(delta: &SnapshotDelta) -> Result<Vec<Json>> {
    delta
        .registers
        .iter()
        .map(|r| {
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
        })
        .collect()
}
fn delta_memory_json(delta: &SnapshotDelta) -> Vec<Json> {
    delta
        .memory
        .iter()
        .map(|(address, pre, post)| {
            object([
                ("address".into(), number(*address)),
                ("post".into(), number(*post as u128)),
                ("pre".into(), number(*pre as u128)),
            ])
        })
        .collect()
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
        ("memory".into(), Json::Array(delta_memory_json(delta))),
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
            if reply.service != request.service || reply.payload != request.payload {
                return Err(fail(format!(
                    "§10: reply mismatch: expected request service={} payload={}, replies file next names service={} payload={}",
                    request.service,
                    hex(&request.payload),
                    reply.service,
                    hex(&reply.payload)
                )));
            }
            let before = trace.is_some().then(|| machine.snapshot());
            machine.reply(&reply.reply)?;
            if let (Some(trace), Some(before)) = (trace.as_deref_mut(), before) {
                let delta = before.delta(&machine.snapshot());
                if let Some(Json::Object(record)) = trace.last_mut() {
                    if let Some(Json::Array(registers)) = record.get_mut("registers") {
                        registers.extend(delta_registers_json(&delta)?);
                    }
                    if let Some(Json::Array(memory)) = record.get_mut("memory") {
                        memory.extend(delta_memory_json(&delta));
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
        let before = trace.is_some().then(|| machine.snapshot());
        let point = machine
            .step_once()?
            .ok_or_else(|| fail("§3.4: running machine took no step"))?;
        steps += 1;
        if let (Some(trace), Some(before)) = (trace.as_deref_mut(), before) {
            let delta = before.delta(&machine.snapshot());
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
fn classical_machine<'a>(
    program: &'a Object,
    name: &str,
    options: &CcOptions,
) -> Result<ClassicalMachine<'a>> {
    let entry = classical_entry(program, name)?;
    let (args, registers) = classical_args(program, entry, &options.args, options.token_defaults)?;
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
    Ok(machine)
}
fn run_cc(program: &Object, name: &str, options: CcOptions) -> Result<Json> {
    let mut replies = load_replies(options.replies.as_deref())?;
    let mut machine = classical_machine(program, name, &options)?;
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
        ("clock".into(), number(machine.clock() as u128)),
        ("console".into(), console_json(&machine)),
        ("console_hex".into(), string(hex(machine.console()))),
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
        ("host_requests".into(), host_requests_json(&machine)),
        ("measurements".into(), measurements_json(&machine)),
        ("memory_digest".into(), string(machine.memory_digest())),
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
        ("tape".into(), tape_json(&machine)),
        ("trace".into(), Json::Array(trace)),
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
    if !["check", "hash", "run-oracle", "step-check", "run-cc"].contains(&command) {
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
    if command == "run-cc" {
        let name = args
            .get(2)
            .ok_or_else(|| fail(USAGE))?
            .trim_start_matches('@');
        program.check_hashes()?;
        let options = parse_cc_options(&program, command, &args, 3)?;
        println!("{}", run_cc(&program, name, options)?.canonical());
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
            "--samples" => samples = Some(uint(value)?),
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
                    return Err(fail("invalid memory HEX"));
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
                .ok_or_else(|| fail("host limit: interface region exceeds host memory"))?;
            let mut zero = Vec::new();
            zero.try_reserve_exact(n).map_err(fail)?;
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
