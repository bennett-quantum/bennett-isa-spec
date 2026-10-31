#!/usr/bin/env python3
"""Byte-exact round-22 reviewer evidence, independent of production code."""
from pathlib import Path
import hashlib
ROOT = Path(__file__).resolve().parents[1] / "fixtures" / "r22"
PINS = {'domain-role-spoof.bisa': '19456db6726bdce2b10f7ae983a243c5cf2c2682b4d707aced28f81d6206e249', 'false-cnot-full-control.bisa': 'b2064197ffe69d1ba01154846e6fffbbc8c0e0408304e15de3b0eaefb758ccd4', 'false-cnot-proof.bisa': 'c9cf4ac51d800f9a918deec3a14790691d0b9c2d55b395949add620425707cca', 'false-cnot-renamed-control.bisa': 'e63096f6b7f3bb0e40ec9d6e41c40ac7f1f50fbe972688f675836c23541c69fb', 'host-probe.rs': '8a6b03ddb9ff7233b3ae565df6c1f3155e5ec16f419adedf3d1e68b962299403', 'hostcall-64MiB.bisa': 'df7f95dd72496eb446c39319d5ef1e6287ebed8fdd6b9a7f92eba31ba1431071', 'hostcall-64MiB.envelope': '655e72211b66a9e4e3f8dcaff74c5203da6ec1d834fc43570833f0ab1387a577', 'hostcall-backtrace.log': '2dcd7f5e3b05e8c2f3d2a3b9fe7054b2a8966431782262b8cf0dc33000837d09', 'probe.rs': '8db85d0f9304f9a34e6fc60273ce6ba8d809ef73d4c46fa2ada4510a275d7010', 'reproduce-false-proof.py': '0a86e5947a3c427a6b5d0b111afb8f734f373dc62360db0572fc62de9766a62c', 'reproduce.py': 'f8dda8465a95401a80d037cf9780eef4df9568e16e9bf91de2404f3435a2aa8d', 'results.json': 'fb165ec4593001ca8622939b7a1701b8e3b552733a91e8c4c8c07572adbccd05'}
for name, expected in PINS.items():
    assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == expected, name
print(f"R22 independent pins: {len(PINS)} reviewer artifacts (5 objects / envelope / logs / probes)")
