"""Exact request-accuracy/default-schedule vectors and additive C/C++ ABI."""
from fractions import Fraction
import json
from pathlib import Path
import re
import subprocess
import tempfile

here = Path(__file__).resolve().parent
root = here.parents[3]
draft = (root / "spec/Bennett-ISA-Spec-0.9.md").read_text()
marker = "**ROOT-ACCURACY witness**\n\n```json\n"
assert marker in draft, "missing explicit root-accuracy publication"
vectors = json.loads((here / "accuracy.json").read_text())
assert vectors == json.loads(draft.split(marker, 1)[1].split("\n```", 1)[0])
requested = Fraction(vectors["request_epsilon"])
for row in vectors["cases"]:
    schedule = row["schedule"]
    matches = schedule is not None and schedule["other_bindings_match"] and Fraction(schedule["epsilon"]) == requested
    assert row["effective"] == ("attached" if matches else "default")
    assert Fraction(row["epsilon"]) == requested
share = vectors["default_share"]
assert Fraction(int(share["numerator"]), int(share["denominator"])) == requested / vectors["inexact_operations"]
assert vectors["exact_native_default_share"] == "0"
assert vectors["explicit_zero"] == {"epsilon": "0", "inexact_operations": 0, "default_shares": ["0"]}
diagnostic = "E_ESTIMATE_ACCURACY_REQUIRED: use bennett_estimate_run_at_accuracy with epsilon_decimal"
assert vectors["legacy_missing_accuracy"] == {
    "status": "BENNETT_E_ARG", "diagnostic": diagnostic, "out": None, "effects": False
}
assert vectors["legacy_entries"] == ["bennett_estimate_run", "bennett_estimate_run_with_token_aliases"]
for entry in vectors["legacy_entries"]:
    for row in vectors["legacy_cases"]:
        schedule = row["schedule"]
        matches_other_bindings = schedule is not None and schedule["other_bindings_match"]
        assert row["resolution"] == ("bound" if matches_other_bindings else "missing")
        if matches_other_bindings:
            # Legacy requests have no independent accuracy to compare; the
            # matching schedule supplies it, preserving zero and its bytes.
            assert row["epsilon"] == schedule["epsilon"]
            assert Fraction(row["epsilon"]) >= 0
            assert row["retains_schedule_bytes"]
        else:
            assert "epsilon" not in row  # neither absent nor stale implies zero
            assert vectors["legacy_missing_accuracy"]["status"] == "BENNETT_E_ARG"
            assert "bennett_estimate_run_at_accuracy" in diagnostic
assert "Pending legacy accuracy policy" not in draft
legacy_rule = draft.split("**Legacy estimate accuracy", 1)[1].split("**ROOT-ACCURACY witness**", 1)[0]
assert "`BENNETT_E_ARG`" in legacy_rule and diagnostic in legacy_rule

supplement = (root / "spec/Launch-Estimate-Accuracy-1.md").read_text()
assert diagnostic in supplement
assert "explicitly pending" not in supplement and "pending legacy" not in supplement
case = json.loads((here / "case.json").read_text())
assert case["expected_outcome"]["old_entry_policy"] == "matching_schedule_or_accuracy_required"
assert case["expected_outcome"]["legacy_missing_accuracy"] == vectors["legacy_missing_accuracy"]
header = (root / "spec/bennett_estimate_accuracy_1.h").read_text()
pattern = r"bennett_status\s+bennett_estimate_run_at_accuracy\s*\([^;]*\);"
declaration = re.findall(pattern, supplement)
export = re.findall(pattern, header)
assert len(declaration) == len(export) == 1
normalize = lambda s: re.findall(r"[A-Za-z_][A-Za-z_0-9]*|[^\s]", s)
assert normalize(declaration[0]) == normalize(export[0])
assert "const char* epsilon_decimal" in declaration[0]
assert '"estimate_accuracy/1"' in supplement
scratch = root / ".tmp/accuracy-vectors"
scratch.mkdir(parents=True, exist_ok=True)
probe = '''#include "bennett_estimate_accuracy_1.h"
typedef bennett_status (*estimate_fn)(bennett_program_t*, const char*, bennett_token,
    const char*, uint32_t, const uint8_t*, size_t, const uint8_t*, size_t,
    const char*, bennett_estimate_t**);
static estimate_fn exact_signature = &bennett_estimate_run_at_accuracy;
'''
with tempfile.TemporaryDirectory(dir=scratch) as directory:
    for compiler, extension, standard in [("cc", "c", "c11"), ("c++", "cc", "c++17")]:
        source = Path(directory) / ("probe." + extension)
        source.write_text(probe)
        subprocess.run([compiler, "-std=" + standard, "-Werror", "-fsyntax-only", "-I" + str(root / "include"), "-I" + str(root / "spec"), str(source)], check=True)
print("PASS: explicit/legacy accuracy resolution, missing-accuracy diagnostic, no implicit value, rational shares and C/C++ signature")
