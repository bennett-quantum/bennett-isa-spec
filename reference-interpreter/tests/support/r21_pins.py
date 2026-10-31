#!/usr/bin/env python3
"""Round-21 reviewer-file pins, independent of interpreter parsing/identity code."""
from pathlib import Path
import hashlib

ROOT = Path(__file__).resolve().parents[1] / "fixtures" / "r21"
PINS = {'cc-length.bisa': '80d62127c4a29f3563e85c518fb86afc12854eaae048dcddeedd4afb3194abad', 'domain-all-A0.bisa': 'dd24d74d29c1db496b4063f9066e83a547945bfb90b6377d7bf41d916e9bb375', 'domain-all-A1.bisa': '717934a41afd1e58a48bb675cd0fa89938d49b718671a5cb98f7c0ddcfbef425', 'domain-all-A64.bisa': '13e471aa26b3b2a2d7f60e7e458b57730d970381df17e9cd67647b414a93b365', 'domain-all.bisa': '5162fbe154f7d5978a21dd48eb3b880879ace59f04cd97b97af9d0e9c67f0ca9', 'r-length.bisa': 'a6c5dd7f71d0d76d7359396d4c4a7e8935977081e0370a597b8e92fde288ed6a', 'sol-claim-region-all-sampled-small.bisa': '0d6659a6fe44e8b2c680d49af705557d8b4db433a26418615b5ec5c915731186', 'sol-claim-region-all-sampled.bisa': 'dfca89aa2073705841586a1c4a76e27be92618417476e9cb3b9016c0e09ab3c4', 'sol-claim-region-all-small.bisa': 'a9b7c6d75df9d58ed95844ef1a9eb9d01f46b85b1653dc587c9cccf9c848dd15'}
for name, expected in PINS.items():
    assert hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == expected, name
for name, expected in [
    ("cc-length", "23802c541101025a57a13ffb5f00ca72a7ad5ff95f78e9a4fd565c59f566752d"),
    ("r-length", "2751a25e6e906ea512bc075da07cda7463161576c56cebf06455ba2d9dfab704"),
]:
    assert hashlib.sha256((ROOT / (name + ".envelope")).read_bytes()).hexdigest() == expected, name
print(f"R21 independent pins: {len(PINS)} complete objects / 2 literal envelopes")
