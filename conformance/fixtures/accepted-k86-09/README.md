# Native model vectors

`vectors.json` records 17 per-entry model outcomes and hashes of complete
canonical targets, two bound schedules and one unchanged source object.
`checked` means the resolved native model obligations pass. It is not a
certificate verdict or a claim of complete scheduled-process checking.

The controlled X matrix is diag(I2,X). Replacing X by −X keeps disabled
branches equal to identity and changes the enabled block's sign. The
difference has singular values 0,0,2,2: its operator norm is 2, while its
Frobenius norm is sqrt(8). Accordingly the native advertised bound 2 passes;
the separate numerical claim with bound 2 fails and bound 3 passes.
Self-comparison passes with numerical bound zero. The reference tests also
exercise the strict interval channel dual independently.

Model bits are ordered most significant first, with controls preceding the
base basis. For CX with one inherited control, entry order is own control,
inherited control, target; allocation order is inherited control, own
control, target, encoded by operand_map [1,0,2].

The source and exact-X schedule are independently admitted by the reference
object reader. Adding model bytes changes target_hash and schedule_hash;
the source object identity stays unchanged. A finite binary64 angle 0.1
binds exactly to 3602879701896397/36028797018963968. The corresponding vector
is explicitly unchecked until nonrational operation identities are checked.

Kraus/instrument/explicit-control encodings and optional enclosure or
unitarity certificate productions are not specified completely in §8.3.
These fixtures introduce no encoding for them. Exact nonrational identity
checking, full schedule reconstruction and executable numerical/channel
certificate dispatch remain incomplete.

Reproduce:

```sh
python3 conformance/tools/k86_vectors.py --check
cargo test --offline --manifest-path reference-interpreter/Cargo.toml --test isa09_native_vectors --test isa09_native_models
cargo test --offline --manifest-path conformance/Cargo.toml --test k86_vectors
```

The generator uses the published X fragment and independent integer algebra;
it never reads model-checker output to establish expected results.
