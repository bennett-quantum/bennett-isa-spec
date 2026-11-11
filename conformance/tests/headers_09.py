"""Compare published declarations to ISA text; compile family-selection probes."""

import argparse
import hashlib
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
INCLUDE = ROOT / "include"
COMPILERS = ("clang-18", "clang-23")
HEADERS_08 = ("bennett_launch.h", "bennett_sched.h")
HEADERS_09 = ("bennett_launch_0_9.h", "bennett_sched_0_9.h")
# Published 6443493 bytes; independent of the candidate's working tree.
HASHES_08 = (
    "d5bb3eee994a850239a69f4241de341bdcb4e41cac8a7c053a5bbc0a4b6b9739",
    "9c011841e1bd9339586dd15ae67c4e288dd6fe0bc3f29a2f08a3af6cd37ef736",
)


def without_comments(text):
    return re.sub(r"/\*.*?\*/", "", text, flags=re.S)


def declarations(text):
    parts = without_comments(text).split(";")
    assert not parts.pop().strip(), "unterminated C declaration"
    return [part.strip() for part in parts if part.strip()]


def c_blocks(spec, section):
    text = spec.split(f"\n## {section}. ", 1)[1].split("\n## ", 1)[0]
    blocks = re.findall(r"```c\n(.*?)\n```", text, re.S)
    assert blocks, f"missing §{section} C blocks"
    return [decl for block in blocks for decl in declarations(block)]


def expected_declarations(family, role):
    filename = "Bennett-ISA-Spec.md" if family == 8 else "Bennett-ISA-Spec-0.9.md"
    spec = (ROOT / "spec" / filename).read_text()
    launch = c_blocks(spec, 10)
    sched = c_blocks(spec, 11)
    if family == 9:
        # §10.1 publishes both role queries together; each belongs to its role.
        sched += [decl for decl in launch if "bennett_sched_features(" in decl]
        launch = [decl for decl in launch if "bennett_sched_features(" not in decl]
    return {"launch": launch, "sched": sched}[role]


def tokens(decl):
    return re.findall(r"[A-Za-z_][A-Za-z_0-9]*|[0-9]+|[^\s]", decl)


def inventory(decls):
    result = {}
    for decl in decls:
        if decl.startswith("typedef "):
            name = re.search(r"(bennett_\w+)\s*$", decl).group(1)
        else:
            name = re.search(r"(bennett_\w+)\s*\(", decl).group(1)
        assert name not in result, f"duplicate declaration: {name}"
        result[name] = tokens(decl)
    return result


def header_declarations(header):
    text = without_comments((INCLUDE / header).read_text())
    # Only preprocessor directives and the C++ linkage wrapper are scaffolding.
    text = "\n".join(
        line for line in text.splitlines()
        if not line.lstrip().startswith("#")
        and line.strip() not in ('extern "C" {', "}")
    )
    return declarations(text)


def contract_probe(family, role):
    """Function-pointer types test every arity, type and argument position."""
    header = {8: HEADERS_08, 9: HEADERS_09}[family][role == "sched"]
    lines = [f'#include "{header}"', f'#include "{header}"',
             '#ifdef __cplusplus', '#include <type_traits>',
             '#define CHECK(c) static_assert(c, #c)',
             '#define SAME(e, t) (std::is_same<decltype(e), t>::value)',
             '#else', '#define CHECK(c) _Static_assert(c, #c)',
             '#define SAME(e, t) _Generic((e), t: 1, default: 0)', '#endif']
    prototypes = []
    for decl in expected_declarations(family, role):
        if decl.startswith("typedef enum {"):
            entries, name = decl.removeprefix("typedef enum {").split("}")
            lines.append(f'CHECK(sizeof({name.strip()}) > 0);')
            value = 0
            for entry in entries.split(","):
                constant, sep, explicit = entry.strip().partition("=")
                if sep:
                    value = int(explicit)
                lines.append(f'CHECK({constant.strip()} == {value});')
                value += 1
        elif decl.startswith("typedef "):
            underlying, name = decl.removeprefix("typedef ").rsplit(" ", 1)
            if underlying.startswith("struct "):
                lines.append(f'CHECK(SAME(({name}*)0, {underlying}*));')
            else:
                assert underlying == "uint64_t", f"unexpected handle: {decl}"
                lines.append(f'CHECK(SAME(({name})0, {underlying}));')
        else:
            head, args = decl.split("(", 1)
            result, name = head.rstrip().rsplit(" ", 1)
            lines += [f'typedef {result} (*expected_{name})({args};',
                      f'CHECK(SAME(&{name}, expected_{name}));']
            prototypes.append(decl + ";")
    # Redeclaring with explicit C linkage also checks the C++ wrapper.
    lines += ['#ifdef __cplusplus', 'extern "C" {', '#endif', *prototypes,
              '#ifdef __cplusplus', '}', '#endif', 'int main(void) { return 0; }']
    return "\n".join(lines) + "\n"


class HeaderFamilies(unittest.TestCase):
    def compile(self, source, success=True, languages=("c",)):
        temporary = Path(os.environ["TMPDIR"])
        temporary.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="headers-", dir=temporary) as directory:
            for compiler in COMPILERS:
                for language in languages:
                    includes = [line for line in source.splitlines() if line.startswith('#include "')]
                    with self.subTest(compiler=compiler, language=language, includes=includes):
                        src = Path(directory) / ("probe.c" if language == "c" else "probe.cpp")
                        src.write_text(source)
                        env = {k: v for k, v in os.environ.items() if k not in
                               ("DISPLAY", "WAYLAND_DISPLAY", "DBUS_SESSION_BUS_ADDRESS", "BROWSER")}
                        standard = "c11" if language == "c" else "c++17"
                        run = subprocess.run(
                            [compiler, f"-std={standard}", "-Wall", "-Wextra", "-Werror",
                             "-pedantic-errors", "-I", str(INCLUDE), "-c", str(src),
                             "-o", str(Path(directory) / "probe.o")],
                            env=env, capture_output=True, text=True, check=False,
                        )
                        if success:
                            self.assertEqual(run.returncode, 0, run.stderr)
                        else:
                            self.assertNotEqual(run.returncode, 0, "mixed/invalid family accepted")

    def test_08_bytes_unchanged(self):
        for header, digest in zip(HEADERS_08, HASHES_08):
            with self.subTest(header=header):
                self.assertEqual(hashlib.sha256((INCLUDE / header).read_bytes()).hexdigest(), digest)

    def test_09_declarations_match_spec(self):
        for role, header, count in zip(("launch", "sched"), HEADERS_09, (59, 20)):
            with self.subTest(header=header):
                expected = inventory(expected_declarations(9, role))
                self.assertEqual(len(expected), count, "review changed normative inventory")
                self.assertEqual(inventory(header_declarations(header)), expected)

    def test_each_header_alone_matches_spec(self):
        for family in (8, 9):
            for role in ("launch", "sched"):
                self.compile(contract_probe(family, role), languages=("c", "c++"))

    def test_same_family_include_orders(self):
        for headers in (HEADERS_08, HEADERS_09):
            for first, second in (headers, headers[::-1]):
                self.compile(f'#include "{first}"\n#include "{second}"\n#include "{first}"\n')

    def test_selector_default_and_explicit_families(self):
        for selection, family in (("", 8), ("#define BENNETT_ABI_FAMILY 8\n", 8),
                                  ("#define BENNETT_ABI_FAMILY 9\n", 9)):
            source = selection + '#include "bennett_abi.h"\n#include "bennett_abi.h"\n'
            source += f'_Static_assert(BENNETT_ABI_FAMILY == {family}, "selected family");\n'
            # Each selected family must retain its own estimate's actual signature.
            estimate = next(d for d in expected_declarations(family, "launch")
                            if re.search(r"\bbennett_estimate_run\(", d))
            head, args = estimate.split("(", 1)
            result, name = head.rstrip().rsplit(" ", 1)
            source += f'typedef {result} (*estimate_signature)({args};\n'
            source += f'_Static_assert(_Generic(&{name}, estimate_signature: 1, default: 0), "estimate ABI");\n'
            source += 'bennett_sched_outcome outcome = BENNETT_SCHED_OK;\n'
            self.compile(source)
            for header in {8: HEADERS_08, 9: HEADERS_09}[family]:
                self.compile(selection + f'#include "{header}"\n' + source[len(selection):])
                self.compile(source + f'#include "{header}"\n')

    def test_mixed_families_rejected_in_both_orders(self):
        for old in HEADERS_08:
            for new in HEADERS_09:
                for first, second in ((old, new), (new, old)):
                    self.compile(f'#include "{first}"\n#include "{second}"\n', success=False)
        for family, other_headers in ((8, HEADERS_09), (9, HEADERS_08)):
            for header in other_headers:
                select = f'#define BENNETT_ABI_FAMILY {family}\n#include "bennett_abi.h"\n'
                self.compile(select + f'#include "{header}"\n', success=False)
                self.compile(f'#include "{header}"\n' + select, success=False)
        for old, new in ((8, 9), (9, 8)):
            self.compile(f'#define BENNETT_ABI_FAMILY {old}\n#include "bennett_abi.h"\n'
                         f'#undef BENNETT_ABI_FAMILY\n#define BENNETT_ABI_FAMILY {new}\n'
                         '#include "bennett_abi.h"\n', success=False)

    def test_invalid_selection_rejected(self):
        for family in (0, 7, 10, "unknown"):
            self.compile(f'#define BENNETT_ABI_FAMILY {family}\n#include "bennett_abi.h"\n', success=False)
        for header in HEADERS_09:
            self.compile(f'#define BENNETT_ABI_FAMILY 8\n#include "{header}"\n', success=False)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--include-dir", type=Path, default=INCLUDE)
    options, remaining = parser.parse_known_args()
    INCLUDE = options.include_dir.resolve()
    unittest.main(argv=[__file__, *remaining], verbosity=2)
