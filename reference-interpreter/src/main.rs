use bennett_isa_ref::{
    json::Json, Argument, Budget, Error, Object, OracleInput, OracleMachine, Result, Type,
};

const USAGE:&str="bennett-isa-ref check <object>\nbennett-isa-ref hash <object>\nbennett-isa-ref run-oracle <object> <oracle> --input <x[,y...]> [--budget B[,heap,stack,depth]] [--mem name=hex] [--arg value]\nbennett-isa-ref step-check <object> <oracle> --exhaustive [--budget B[,heap,stack,depth]] [--arg value] [--max-bits N]";
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
    if !["check", "hash", "run-oracle", "step-check"].contains(&command) {
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
        println!(
            "{}",
            object([
                ("object_hash".into(), string(identity.object_hash)),
                ("status".into(), string("ok")),
                ("symbols".into(), symbols)
            ])
            .canonical()
        );
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
    let mut limits = program.default_budget(if command == "step-check" {
        256
    } else {
        1_000_000
    })?;
    let mut exhaustive = false;
    let mut max_bits = 16u16;
    let mut fixed_regions = Vec::new();
    let mut pos = 3;
    while pos < args.len() {
        let option = &args[pos];
        pos += 1;
        if option == "--exhaustive" {
            exhaustive = true;
            continue;
        }
        let value = args
            .get(pos)
            .ok_or_else(|| fail(format!("missing value for {option}")))?;
        pos += 1;
        match option.as_str() {
            "--input" => {
                input.ins = if value.is_empty() {
                    Vec::new()
                } else {
                    value.split(',').map(uint).collect::<Result<_>>()?
                };
            }
            "--budget" => limits = budget(value, limits)?,
            "--max-bits" => max_bits = value.parse().map_err(fail)?,
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
                unreachable!("region type")
            };
            let n = usize::try_from(1u128 << a).map_err(fail)?;
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
        let r = program.run_oracle(name, &input, limits)?;
        println!(
            "{}",
            object([
                ("payload".into(), number(r.payload)),
                ("code".into(), number(r.code)),
                ("steps".into(), number(r.steps as u128))
            ])
            .canonical()
        );
        return Ok(());
    }
    if !exhaustive {
        return Err(fail("step-check requires --exhaustive"));
    }
    if max_bits > 24 {
        return Err(fail("--max-bits exceeds the CLI enumeration limit of 24"));
    }
    let input_widths = oracle.params[0]
        .iter()
        .map(|p| p.ty.width(program.ptr_width))
        .collect::<Result<Vec<_>>>()?;
    let mut bits = input_widths.iter().map(|w| *w as u128).sum::<u128>();
    for p in &oracle.params[3] {
        if !fixed_regions.contains(&p.name) {
            let Type::Region(a, w) = p.ty else {
                unreachable!("region type")
            };
            bits = bits
                .checked_add((1u128 << a) * w as u128)
                .ok_or_else(|| fail("domain bit count overflow"))?;
        }
    }
    if bits > max_bits as u128 {
        return Err(fail(format!("§17.13: exhaustive domain has {bits} input/memory bits, above --max-bits {max_bits}; no sampling performed")));
    }
    let cases = 1u128 << (bits as u32);
    for value in 0..cases {
        let mut cursor = 0;
        input.ins.clear();
        for w in &input_widths {
            let mask = if *w == 128 {
                u128::MAX
            } else {
                (1u128 << w) - 1
            };
            input.ins.push((value >> cursor) & mask);
            cursor += *w as u32;
        }
        for p in &oracle.params[3] {
            if !fixed_regions.contains(&p.name) {
                for byte in input.regions.get_mut(&p.name).expect("region initialised") {
                    *byte = ((value >> cursor) & 255) as u8;
                    cursor += 8;
                }
            }
        }
        let expected = program.run_oracle(name, &input, limits)?;
        let mut machine = OracleMachine::new(&program, name, &input, limits)?;
        let before = machine.snapshot();
        for _ in 0..limits.steps {
            machine.forward_step()?;
        }
        if machine.outcome() != expected {
            return Err(fail(format!("§14.3 T6: disagreement at input {value}")));
        }
        for _ in 0..limits.steps {
            machine.inverse_step()?;
        }
        if machine.snapshot() != before || !machine.frames_clean() {
            return Err(fail(format!("§14.3 T2: dirty prefix at input {value}")));
        }
    }
    let fixed = Json::Array(fixed_regions.into_iter().map(string).collect());
    println!(
        "{}",
        object([
            ("cases".into(), number(cases)),
            ("input_and_memory_bits".into(), number(bits)),
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
