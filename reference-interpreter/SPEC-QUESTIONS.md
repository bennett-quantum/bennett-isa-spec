# Implementation readings of ISA 0.8

These questions do not narrow the supported ISA. The accepted specification is
unchanged. Each entry records the applicable rule and the chosen reading.

1. **Indices: blocks or rows? (§3.6, §4.2, §16.3).** The index-space paragraph
   calls its elements blocks, but every instruction is a budget step with k+1
   fallthrough, calls record k+1, and the published inc8 takes three steps in
   one block. Use one index per instruction, concatenating blocks in text
   order; a label denotes its block's first instruction. A function's range
   contains all its instruction indices. Preserve these indices in lift.label.
2. **Fixture comments versus canonical text (§16.1–§16.2).** The job requests
   expected-result comments in fixtures; the interchange grammar has no comment
   syntax. Store hand-derived comments in adjacent .comments files. The .bisa
   files remain canonical objects, with no undocumented comment extension.
3. **Idle steps (§3.6, budget design note I5).** Implement the printed HALT
   successor XOR and two exchanges, including old_pc = HALT in an idle frame.
   Prefix reversal clears it. Do not silently implement the proposed 0.9 change.
4. **Counter widths (§3.7, §9.1).** The layout schema supplies size exponents
   but no Hc/Ac/Sc/Dc fields. Derive the minimum legal widths from §3.7 and the
   invocation's depth limit. The sequential interpreter uses unbounded logical
   counters; concrete machine counters use these derived widths.
5. **Reported step count on budget exhaustion (§14.1, Python seed).** Count
   executed rows, at most B. The seed increments to B+1 before refusing the
   next row; expose the normative executed-row count, and test the seed's
   outcomes independently of that off-by-one diagnostic.
6. **Unknown device identity (§9.2, §17 item 3).** An unmodelled effect has
   the specified fault outcome 8 and its operation token is in the canonical
   body. It contributes no device-envelope entry because no such model exists.
   A known device model or host service used without a declaration is an error.
7. **Ordering multi-member lproc SCCs (§3.6, §9.2).** Member closure hashes
   are unrelated SHA-256 values and cannot order members by SCC index. Sort
   multi-member SCC groups by their scc_hash, then their members by name/SCC
   index; a singleton uses its symbol closure hash. The mutual.bisa fixture
   deliberately has hash(f) > hash(g) and checks that g's label is still 4.
8. **Host reply length wider than ptr_width (§10).** rlen must report the full
   reply length, in an integer of ptr_width. Reject an unrepresentable reply
   with E_ARG before any memory write, allowing the host to provide a valid
   reply. Truncating rlen would defeat the specified truncation detection.
9. **Empty data images (§16.1 versus §16.2).** HEX explicitly admits the
   empty string and a zero-sized data signature is legal, but NAME SP HEX
   then ends with a space. Honor the explicit data grammar: accept exactly
   `NAME SP LF` for a zero-sized data image, as the sole trailing-space
   exception. All nonempty data and all other lines keep §16.1's space rule.
10. **Fault width in shared lprocs (§3.3, §3.7).** An lproc signature has no
    fault width. Check its fault literals against every enclosing oracle's F;
    equivalently use the minimum F among its enclosing oracle closures. An
    unattached lproc is checked again when linked to an oracle. This permits
    fault31 under F=5 and rejects it if the same procedure is used under F=4.
11. **Metadata encodings omitted by the schema sketches (§9.2, §9.4, §12–13).**
    Encode a program-point tuple as `[interface_hash, block_name, ordinal]`;
    schedule paths retain §11's explicit `{symbol,point,iteration}` form.
    Certificate `versions` uses the seven version names from the object
    header. A frame uses `interface` (role/width/class slots), class maps
    `scratch`, `borrow`, `scheme_ancillae`, and `retained_history` (an expression
    in c). Scratch counts may separate `clean` and `dirty`. Optional
    `live_ranges` holds `{frame,start,end}` half-open generated-event intervals;
    reuse of one frame requires disjoint intervals. These concrete JSON
    choices implement the prose fields without adding semantic permissions.
    Coverage classification `trapped` requires its fault_code; `shimmed`
    requires its shim_hash. The specification does not enumerate variant
    names, so nonempty variant identifiers are retained without inventing an
    enum. Certificate proof verification remains a separate obligation from
    validating these encodings.
