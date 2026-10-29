use bennett_isa_ref::{
    json::Json, Argument, BitVector, Budget, Error, Object, OracleInput, OracleVectorInput,
    OracleVectorMachine, Result, Type, Xoshiro256,
};

const USAGE:&str="bennett-isa-ref check <object>\nbennett-isa-ref hash <object>\nbennett-isa-ref run-oracle <object> <oracle> --input <x[,y...]> [--budget B[,heap,stack,depth]] [--mem name=hex] [--arg value]\nbennett-isa-ref step-check <object> <oracle> (--exhaustive [--max-bits N] | --suite | --samples N) [--seed a,b,c,d] [--budget B[,heap,stack,depth]] [--mem name=hex] [--arg value]";
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
