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
