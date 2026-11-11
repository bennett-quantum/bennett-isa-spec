# Streamed action reversal, version 1

```
bennett-isa-ref reverse-action-batch OBJECT ACTION [--range LO..HI]
  [--format binary|jsonl] [--flush-each] [--arg VALUE ... | --args VALUES]
  [--token-aliases JSON] [--adjoint]
```

The object is parsed and its stored hashes admitted once. Fixed interface
bindings, classical arguments, borrowed-pool layout, source instructions and
lifetime-width setup are reused. Every tuple starts with fresh quantum state
and the ordinary invocation token resolution, executes the selected direction,
then its inverse. Execution, faults, cleanup and exact amplitude arithmetic use
the same interpreter as `reverse`; no native result or sampled shortcut is used.
Classical argument spellings and alias rules are those of `run-action`.

Prepared execution may reuse verified pure `F_B` values for unscheduled ISA 0.8
oracles without classical/region arguments. The cache belongs to one prepared
object and keys the callee, complete input vectors, all four budget components
and control count. Only payload/fault values are retained, after the literal
forward/inverse machine has restored every private field and frame. Target and
initial fault bits are excluded by §5.2's XOR-delivery contract. Every tuple
still executes its own forward/inverse map on fresh state and is fully checked.
The data cache is bounded; ineligible calls and misses beyond its capacity use
the literal path. No tuple, fault, phase or reversal check is sampled or omitted.

Without `--range`, stdin is JSONL. Each line has **exactly** these four keys:

```json
{"mut":[0,0],"in":[1,2],"mem":{},"borrowed":[]}
```

Arrays contain one unsigned value per declared bundle; `mem` maps every
declared interface-region name to all its cell values, in address order.
`borrowed` is empty when no idle borrowed pool is needed, otherwise contains
one integer for the complete pool. Values use ISA canonical uint encoding:
JSON numbers below 2^53, otherwise canonical unsigned decimal strings. Counts,
widths and values must match exactly; unknown/missing fields, extra values,
blank lines and malformed lines fail. Classical arguments are fixed for the
invocation. Each accepted line is executed once, in order; duplicates remain
separate cases. Input is consumed incrementally, not collected until EOF.

`LO..HI` is a half-open tuple range, with canonical decimal bounds of arbitrary
precision: `0 <= LO <= HI <= 2^N`. N is the total interface-plus-pool bit count.
Components enumerate in this order: input bundles, mutable bundles, interface
region cells, borrowed pool; declaration/address order within each group.
The **last component varies fastest**; each component is an unsigned integer.
For rung-1 `(x,y)` inputs and `(target,fault)` mutables this is exactly
`x, y, target, fault`, fault fastest. All tuples in the range are executed.
Shards must cover the requested domain without gaps or overlaps. Empty shards
are valid; they do not establish coverage of a nonempty domain.

## Output framing

Default encoding is `binary`. Both encodings start with one UTF-8 JCS header
and LF. The header has exactly these keys:

```
type:"header", format:"bennett.reverse-action-batch/1", encoding:"binary"|"jsonl",
action:string, adjoint:boolean, object_hash:hash, interface_hash:hash,
closure_hash:hash|null, args:[CLI-string,...], token_aliases:[uint,...]|null,
domain_bits:uint, record_bytes:uint, mode:"range"|"jsonl",
lo:decimal-string|null, hi:decimal-string|null,
rank_order:[component-index,...], components:[Component,...]
Component = {group:"mut"|"in"|"mem"|"borrowed", name:string,
             cell:uint|null, offset:uint, width:uint}
```

Components in the header are in **physical output order**: mutable bundles,
input bundles, region cells, borrowed pool. Offset/width address that packed
bit string. `rank_order` lists their indices in enumeration order. Within a
bundle/cell/pool bit zero comes first. Packed bytes are little-endian bit
strings; unused high bits in the final byte are zero. The pool's display name
is `$borrowed`; its cell is null. `lo` and `hi` are null for stdin mode.

Each binary case is `0x01, FLAGS, FORWARD, REVERSED`, with exactly
`record_bytes = 2 + 2*ceil(N/8)` bytes. Each payload is ceil(N/8) bytes and
contains the **complete** output tuple, including inputs, regions and pool.
An unavailable non-basis tuple has zero payload bytes and its basis bit clear.
JSONL cases contain the equivalent five-element array and LF:

```
["case", INDEX, FLAGS, "forward-lowercase-hex"|null, "reversed-lowercase-hex"|null]
```

INDEX is always a canonical decimal **string**: the global rank in range mode,
or the zero-based line/case number in stdin mode. Binary indices are implicit
in record order. FLAGS is the following bit mask (bit 7 is reserved and zero):

| Bit | Exact observation when set |
| --- | --- |
| 0 | Forward output has one basis component. |
| 1 | Forward basis amplitude is exactly +1, using the exact shadow. |
| 2 | Forward execution released all workspace and restored the entry wire layout/counter. |
| 3 | Input bundles, in-role region cells and borrowed pool equal entry values. |
| 4 | Full state after inverse equals entry exactly, including amplitudes and wire identities/counter. |
| 5 | Reversed output has one basis component. |
| 6 | Reversed basis amplitude is exactly +1. |

All seven bits must be set (127) for a passing local execution check. A phase
failure or superposition still runs the inverse and reports a failed case.
An interpreter error, including an unclean release, aborts the stream.

Binary output ends with byte `0x00`, followed by the JCS footer and LF.
JSONL output ends with the same footer as a normal line:

```
{type:"footer", format:"bennett.reverse-action-batch/1", complete:true,
 cases:decimal-string, failed:decimal-string, first_failed:decimal-string|null}
```

`failed` counts cases whose FLAGS differs from 127. `first_failed` is the first
such INDEX. Exit 0 requires zero failed local checks and successful output;
otherwise exit 1. On input, execution or I/O error there is **no complete
footer**, exit is nonzero and stderr names the failing tuple when available.
A prefix, timeout, missing footer, extra/truncated record or failed process is
never a complete shard. Output is buffered in bounded chunks; `--flush-each`
flushes every case for interactive request/response use. The header always
flushes immediately. No state from an earlier tuple is reused as entry state.

## Independent consumer checks

Check the pinned executable, hashes, signature/layout, fixed arguments and
shard endpoints. Decode every ordered tuple, compare the entire forward tuple
against the native result with its initial mutable values, and compare the
reversed tuple against that exact input. Require FLAGS=127, complete framing,
the expected count and a successful exit. Track coverage across all shards.
The footer reports local execution checks, not native equality or full-domain
acceptance. Consumers can hash the ordered raw stream while independently
checking every case, retaining the count, digest and first mismatches without
storing all records. No tuples may be omitted or sampled.

For example, select bounded shards and retain the unchanged command cap:

```
timeout 300 bennett-isa-ref reverse-action-batch object.bennett rung1_full \
  --arg token:256,0,32,32 --range 0..4096 > shard.bin
```

Use measured cases/s under the intended load to choose shard sizes and core
counts. A completed small shard is throughput/correctness evidence for those
tuples, not acceptance of the complete 268,435,456-tuple binary-input domain.
