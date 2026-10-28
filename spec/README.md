# spec/

`Bennett-ISA-Spec.md` is the normative text: draft 0.8, **accepted by the
three-model council on 2026-10-28 (round 8, all members accept, no blocking
issues, mean 4.17/5)**. Council records and the runner live in the `bennett`
meta repo (`docs/council/isa-2026-10-28/`, `tools/council.py`).

Rules for changing it:

- Every semantic change bumps the full version string (section 1); drafts
  are not envelope-compatible, so the four example hashes in section 16.3
  change with it. Run `python3 reference-interpreter/isa_check.py --repatch`
  then `python3 reference-interpreter/isa_check.py`.
- Surgical edits only; a rewrite has introduced a mechanical error every
  time it was tried.
- Add every new example or stated result to `isa_check.py` before anyone
  grades it.
- Section 17 lists decisions deferred to implementation with their
  defaults; an implementer who must choose something not covered there
  files an issue and the default goes into section 17 first.
- Revisions go back through the council.
