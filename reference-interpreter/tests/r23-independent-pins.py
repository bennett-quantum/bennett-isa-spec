#!/usr/bin/env python3
"""Byte-exact reviewer artifacts, fixed independently of Rust identity code."""
from pathlib import Path
import hashlib
root = Path(__file__).parent / "fixtures" / "r23"
pins = {'codex-store-retry-results.json': '1e040faedc7046076d96b10d515463d5e4c21d01b27a26e9a0c24976673a2899', 'codex-store-retry.bisa': '24ee4403ae9037b1f6d7b8bfcbf01cf0fb05d4732cefd9f10fe8021ccf4a23fd', 'codex-store-retry.envelope': 'bc33128d4826fa8851677edb2f9f998280dcbb241d33848a2b326bd425051c52', 'codex-store-retry.rs': 'ef98eb7f48844a7c6088160bef5139c9f8c9b8b1365079c16a4ce3dfbf07ee7f', 'sol-probe.rs': '5755268116ac887f1c8300f19d874806c3cc8c4b8674d7c05cdf63df5180c46f', 'sol-store-cow-results.json': '65e0b48cbd63060e2479b39716d62dd6fc5ee943e4903e9d54a5be1bc551f8c8', 'sol-store-cow.bisa': 'e94c5cbbbdfdfeb32d4101cf6a6033172afcea1c6f6b9cc1bd1df80ad541ece5', 'sol-store-cow.envelope': 'abcde78cd19169240834bca4e8c30f1a036782615a97495b329c6f7b4dd3d97f'}
for name, digest in pins.items():
    actual = hashlib.sha256((root / name).read_bytes()).hexdigest()
    assert actual == digest, (name, actual, digest)
print(f"R23: {len(pins)} byte-exact reviewer artifact pins pass")
