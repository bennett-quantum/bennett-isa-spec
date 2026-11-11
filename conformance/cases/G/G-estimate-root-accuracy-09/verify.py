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
assert vectors["legacy_missing_accuracy"] == "pending"
assert "Pending legacy accuracy policy" in draft

supplement = (root / "spec/Launch-Estimate-Accuracy-1.md").read_text()
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
print("PASS: exact explicit accuracy, absent/mismatched schedules, rational shares and C/C++ extension signature")
