#!/usr/bin/env python3
"""Atomically publish a completed CLI build to the shared consumer path.

Run only after cargo build succeeds, with a private CARGO_TARGET_DIR. The
published path is a regular snapshot, so cargo can replace its own output
without making concurrent consumers' executable disappear.
"""
import argparse
import os
from pathlib import Path
import shutil
import stat
import tempfile


def publish(source, destination):
    if source.resolve() == destination.absolute():
        raise ValueError("build output and published path must be separate")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with source.open("rb") as built:
            before = os.fstat(built.fileno())
            if not stat.S_ISREG(before.st_mode) or not before.st_mode & 0o111:
                raise ValueError("completed build must be a regular executable")
            with tempfile.NamedTemporaryFile(prefix=".bennett-isa-ref-",
                                             dir=destination.parent, delete=False) as snapshot:
                temporary = Path(snapshot.name)
                shutil.copyfileobj(built, snapshot)
                os.fchmod(snapshot.fileno(), stat.S_IMODE(before.st_mode))
                snapshot.flush()
                os.fsync(snapshot.fileno())
            after = os.fstat(built.fileno())
            if (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns):
                raise ValueError("build changed during publication; retry after cargo finishes")
        # Replace the directory entry itself, including an old symlink. Never
        # unlink the public executable first and never write through a symlink.
        os.replace(temporary, destination)
        temporary = None
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="completed private cargo build")
    parser.add_argument("--destination", type=Path,
                        default=Path(__file__).resolve().parents[1] /
                        "target/debug/bennett-isa-ref")
    args = parser.parse_args()
    publish(args.source, args.destination)
    print(f"Published completed CLI snapshot: {args.destination}")


if __name__ == "__main__":
    main()
