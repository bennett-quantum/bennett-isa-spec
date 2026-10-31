"""Independent §9.2 literal envelopes; no interpreter or rehash helper."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
for n, expected, file_pin in [
    (18446744073709551614, "9146b47a91b5cade7648cdb4386bab0fc399048bd754674cb4745e79eeccb318", "254bf985f88a26f94c13ff4e503cb1fff95ea5d4452f17422e194e73bb6eb367"),
    (18446744073709551615, "97eb756124446d26628e2abbe5fb74223cbad5eb7902aacd31b7a309069f48f1", "94416b11560b138ee17b68f5aa930d73c5c9521e343a533842a7d0d6aa0a8344"),
]:
    data = (HERE / f"r20-const-width-{n}.bisa").read_bytes()
    assert hashlib.sha256(data).hexdigest() == file_pin
    prefix, body = data.decode().split("section functions\n")
    symbol = json.loads(prefix.split("section symbols\n")[1].strip())["0"]
    signature = json.dumps(symbol["signature"], sort_keys=True, separators=(",", ":"))
    envelope = ("isa=0.8\nptr_width=32\n" + signature + "\n" + body).encode()
    assert len(envelope) == 269
    assert hashlib.sha256(envelope).hexdigest() == expected == symbol["interface_hash"]
    print(f"N={n}: envelope={len(envelope)} bytes, interface=closure={expected}, file={file_pin}")
