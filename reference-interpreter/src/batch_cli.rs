//! Stream framing for BATCH-REVERSE.md; execution remains in PreparedAction.
use super::*;
use std::io::{BufRead, Write};

const FORMAT: &str = "bennett.reverse-action-batch/1";

struct Component {
    group: &'static str,
    name: String,
    cell: Option<usize>,
    wires: Vec<u64>,
}
fn packed(bits: &[bool]) -> Vec<u8> {
    let mut bytes = vec![0; bits.len().div_ceil(8)];
    for (i, value) in bits.iter().enumerate() {
        bytes[i / 8] |= u8::from(*value) << (i % 8);
    }
    bytes
}
fn flags(out: &bennett_isa_ref::BasisReverse) -> u8 {
    u8::from(out.forward.is_some())
        | u8::from(out.forward_positive) << 1
        | u8::from(out.workspace_clean) << 2
        | u8::from(out.inputs_preserved) << 3
        | u8::from(out.equals_entry) << 4
        | u8::from(out.reversed.is_some()) << 5
        | u8::from(out.reversed_positive) << 6
}
fn decimal(raw: &str, width: usize) -> Result<BitVector> {
    if raw.is_empty()
        || !raw.bytes().all(|b| b.is_ascii_digit())
        || (raw.len() > 1 && raw.starts_with('0'))
    {
        return Err(fail(
            "batch range requires canonical unsigned decimal bounds",
        ));
    }
    BitVector::parse_unsigned_exact(width, raw)
}
struct Range {
    current: BitVector,
    end: BitVector,
    one: BitVector,
}
impl Range {
    fn parse(raw: &str, bits: usize) -> Result<Self> {
        let (lo, hi) = raw
            .split_once("..")
            .ok_or_else(|| fail("batch range requires LO..HI (exclusive HI)"))?;
        let width = bits
            .checked_add(1)
            .ok_or_else(|| Error::host_resource("batch range width exceeds usize".into()))?;
        let current = decimal(lo, width)?;
        let end = decimal(hi, width)?;
        let mut limit = BitVector::zero(width)?;
        limit.set(bits, true)?;
        if current.unsigned_cmp(&end)?.is_gt() || end.unsigned_cmp(&limit)?.is_gt() {
            return Err(fail("batch range requires 0 <= LO <= HI <= 2^domain_bits"));
        }
        Ok(Self {
            current,
            end,
            one: BitVector::from_u128_exact(width, 1)?,
        })
    }
    fn next(&mut self, positions: &[usize]) -> Result<Option<(String, Vec<bool>)>> {
        if self.current == self.end {
            return Ok(None);
        }
        let mut bits = vec![false; positions.len()];
        let index = match self.current.to_u128() {
            Ok(value) => {
                for (bit, position) in positions.iter().enumerate() {
                    bits[*position] = bit < 128 && value & (1u128 << bit) != 0;
                }
                value.to_string()
            }
            Err(_) => {
                for (bit, position) in positions.iter().enumerate() {
                    bits[*position] = self.current.bit(bit)?;
                }
                self.current.to_decimal()
            }
        };
        self.current = self.current.binary("add", &self.one)?;
        Ok(Some((index, bits)))
    }
}
fn put(value: &Json, wires: &[u64], bits: &mut [bool]) -> Result<()> {
    let value = BitVector::parse_unsigned_exact(wires.len(), value.uint_text()?)?;
    for (bit, wire) in wires.iter().enumerate() {
        bits[*wire as usize] = value.bit(bit)?;
    }
    Ok(())
}
fn tuple(value: &Json, run: &ActionRun) -> Result<Vec<bool>> {
    let fields = value.object()?;
    if fields.len() != 4
        || !["mut", "in", "mem", "borrowed"]
            .iter()
            .all(|key| fields.contains_key(*key))
    {
        return Err(fail("batch tuple requires exactly mut, in, mem, borrowed"));
    }
    let mut bits = vec![false; run.state.wire_count()];
    for (key, bundles) in [("mut", &run.mut_bundles), ("in", &run.in_bundles)] {
        let values = value.get(key)?.array()?;
        if values.len() != bundles.len() {
            return Err(fail(format!("batch {key} bundle count mismatch")));
        }
        for (value, wires) in values.iter().zip(bundles) {
            put(value, wires, &mut bits)?;
        }
    }
    let mem = value.get("mem")?.object()?;
    if mem.len() != run.regions.len() {
        return Err(fail("batch mem region count mismatch"));
    }
    for (name, cells) in &run.regions {
        let values = mem
            .get(name)
            .ok_or_else(|| fail("batch missing mem region"))?
            .array()?;
        if values.len() != cells.len() {
            return Err(fail("batch mem cell count mismatch"));
        }
        for (value, wires) in values.iter().zip(cells) {
            put(value, wires, &mut bits)?;
        }
    }
    let borrowed = value.get("borrowed")?.array()?;
    if borrowed.len() != usize::from(!run.borrowed.is_empty()) {
        return Err(fail("batch borrowed pool count mismatch"));
    }
    if let Some(value) = borrowed.first() {
        put(value, &run.borrowed, &mut bits)?;
    }
    Ok(bits)
}

pub(super) fn run(program: &Object, name: &str, args: &[String]) -> Result<bool> {
    let f = program
        .functions
        .get(name)
        .filter(|f| matches!(f.kind.as_str(), "action" | "perm_action"))
        .ok_or_else(|| fail(format!("§5.3: unknown action @{name}")))?;
    let mut raw_range = None;
    let mut jsonl = false;
    let mut flush_each = false;
    let mut format_seen = false;
    let mut common = vec!["--mut".into(), vec!["0"; f.params[0].len()].join(",")];
    let mut pos = 3;
    while pos < args.len() {
        let option = args[pos].as_str();
        pos += 1;
        if option == "--flush-each" {
            if flush_each {
                return Err(fail("--flush-each given twice"));
            }
            flush_each = true;
            continue;
        }
        if option == "--adjoint" {
            common.push(option.into());
            continue;
        }
        let value = args
            .get(pos)
            .ok_or_else(|| fail(format!("missing value for {option}")))?;
        pos += 1;
        match option {
            "--range" => {
                if raw_range.replace(value.as_str()).is_some() {
                    return Err(fail("--range given twice"));
                }
            }
            "--format" => {
                if format_seen {
                    return Err(fail("--format given twice"));
                }
                format_seen = true;
                jsonl = match value.as_str() {
                    "jsonl" => true,
                    "binary" => false,
                    _ => return Err(fail("batch format is binary or jsonl")),
                };
            }
            "--arg" | "--args" | "--token-aliases" => {
                common.push(option.into());
                common.push(value.clone());
            }
            _ => return Err(fail(format!("unknown batch option {option}"))),
        }
    }
    let options = parse_action_options(&common, 0)?;
    let run = build_action_state(program, f, &options)?;
    let prepared = program.prepare_action(name, &run.bindings, options.token_aliases.as_deref())?;
    let mut components = Vec::new();
    for (group, params, bundles) in [
        ("mut", &f.params[0], &run.mut_bundles),
        ("in", &f.params[1], &run.in_bundles),
    ] {
        for (param, wires) in params.iter().zip(bundles) {
            components.push(Component {
                group,
                name: param.name.clone(),
                cell: None,
                wires: wires.clone(),
            });
        }
    }
    for (name, cells) in &run.regions {
        for (cell, wires) in cells.iter().enumerate() {
            components.push(Component {
                group: "mem",
                name: name.clone(),
                cell: Some(cell),
                wires: wires.clone(),
            });
        }
    }
    if !run.borrowed.is_empty() {
        components.push(Component {
            group: "borrowed",
            name: "$borrowed".into(),
            cell: None,
            wires: run.borrowed.clone(),
        });
    }
    let order = ["in", "mut", "mem", "borrowed"]
        .into_iter()
        .flat_map(|group| {
            components
                .iter()
                .enumerate()
                .filter(move |(_, c)| c.group == group)
                .map(|(index, _)| index)
        })
        .collect::<Vec<_>>();
    let positions = order
        .iter()
        .rev()
        .flat_map(|index| components[*index].wires.iter().map(|wire| *wire as usize))
        .collect::<Vec<_>>();
    let total = run.state.wire_count();
    let mut range = raw_range.map(|r| Range::parse(r, total)).transpose()?;
    let identity = program.identity()?;
    let header = object([
        ("type".into(), string("header")),
        ("format".into(), string(FORMAT)),
        (
            "encoding".into(),
            string(if jsonl { "jsonl" } else { "binary" }),
        ),
        ("action".into(), string(name)),
        ("adjoint".into(), Json::Bool(options.adjoint)),
        ("object_hash".into(), string(identity.object_hash)),
        (
            "interface_hash".into(),
            string(&identity.interface_hashes[name]),
        ),
        (
            "closure_hash".into(),
            identity.closure_hashes[name]
                .as_ref()
                .map(string)
                .unwrap_or(Json::Null),
        ),
        (
            "args".into(),
            Json::Array(options.args.iter().map(string).collect()),
        ),
        (
            "token_aliases".into(),
            options
                .token_aliases
                .as_ref()
                .map(|a| Json::Array(a.iter().map(|v| number(*v as u128)).collect()))
                .unwrap_or(Json::Null),
        ),
        ("domain_bits".into(), number(total as u128)),
        (
            "record_bytes".into(),
            number((2 + 2 * total.div_ceil(8)) as u128),
        ),
        (
            "mode".into(),
            string(if range.is_some() { "range" } else { "jsonl" }),
        ),
        (
            "lo".into(),
            range
                .as_ref()
                .map(|r| string(r.current.to_decimal()))
                .unwrap_or(Json::Null),
        ),
        (
            "hi".into(),
            range
                .as_ref()
                .map(|r| string(r.end.to_decimal()))
                .unwrap_or(Json::Null),
        ),
        (
            "rank_order".into(),
            Json::Array(order.iter().map(|i| number(*i as u128)).collect()),
        ),
        (
            "components".into(),
            Json::Array(
                components
                    .iter()
                    .map(|c| {
                        object([
                            ("group".into(), string(c.group)),
                            ("name".into(), string(&c.name)),
                            (
                                "cell".into(),
                                c.cell.map(|i| number(i as u128)).unwrap_or(Json::Null),
                            ),
                            (
                                "offset".into(),
                                number(c.wires.first().copied().unwrap_or(0) as u128),
                            ),
                            ("width".into(), number(c.wires.len() as u128)),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]);
    let stdout = std::io::stdout();
    let mut output = std::io::BufWriter::with_capacity(65536, stdout.lock());
    writeln!(output, "{}", header.try_canonical()?).map_err(fail)?;
    output.flush().map_err(fail)?;
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let mut line = String::new();
    let mut count = 0u128;
    let mut failed = 0u128;
    let mut first_failed = None;
    let zero_payload = vec![0; total.div_ceil(8)];
    loop {
        let (index, bits) = if let Some(range) = &mut range {
            let Some(row) = range.next(&positions)? else {
                break;
            };
            row
        } else {
            line.clear();
            if input.read_line(&mut line).map_err(fail)? == 0 {
                break;
            }
            let value =
                Json::parse(line.trim()).map_err(|e| e.context(&format!("batch tuple {count}")))?;
            let bits =
                tuple(&value, &run).map_err(|e| e.context(&format!("batch tuple {count}")))?;
            (count.to_string(), bits)
        };
        let outcome = prepared
            .reverse_basis(&run.state, &bits, options.adjoint)
            .map_err(|e| e.context(&format!("batch tuple {index}")))?;
        let bits = flags(&outcome);
        if bits != 127 {
            failed = failed
                .checked_add(1)
                .ok_or_else(|| Error::host_resource("batch failed count exceeds u128".into()))?;
            if first_failed.is_none() {
                first_failed = Some(index.clone());
            }
        }
        let forward = outcome.forward.as_deref().map(packed);
        let reversed = outcome.reversed.as_deref().map(packed);
        if jsonl {
            let row = Json::Array(vec![
                string("case"),
                string(index),
                number(bits as u128),
                forward
                    .as_deref()
                    .map(hex)
                    .transpose()?
                    .map(string)
                    .unwrap_or(Json::Null),
                reversed
                    .as_deref()
                    .map(hex)
                    .transpose()?
                    .map(string)
                    .unwrap_or(Json::Null),
            ]);
            writeln!(output, "{}", row.try_canonical()?).map_err(fail)?;
        } else {
            output.write_all(&[1, bits]).map_err(fail)?;
            output
                .write_all(forward.as_deref().unwrap_or(&zero_payload))
                .map_err(fail)?;
            output
                .write_all(reversed.as_deref().unwrap_or(&zero_payload))
                .map_err(fail)?;
        }
        count = count
            .checked_add(1)
            .ok_or_else(|| Error::host_resource("batch count exceeds u128".into()))?;
        if flush_each {
            output.flush().map_err(fail)?;
        }
    }
    let footer = object([
        ("type".into(), string("footer")),
        ("format".into(), string(FORMAT)),
        ("complete".into(), Json::Bool(true)),
        ("cases".into(), string(count.to_string())),
        ("failed".into(), string(failed.to_string())),
        (
            "first_failed".into(),
            first_failed.map(string).unwrap_or(Json::Null),
        ),
    ]);
    if !jsonl {
        output.write_all(&[0]).map_err(fail)?;
    }
    writeln!(output, "{}", footer.try_canonical()?).map_err(fail)?;
    output.flush().map_err(fail)?;
    Ok(failed == 0)
}
