from __future__ import annotations

import base64
import bz2
import gzip
import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STAGING = ROOT / "assets" / "staging"
TARGET = ROOT / "assets" / "Market_Economy_Radar_Rulebook_v4_ULTRA.txt.gz"
RAW_SHA256 = "2f2a3a189c594fdb2a581e6f052123a0dc778e8065677e88d5764f9c813b0b56"
GZ_SHA256 = "bd3630be31f3e166a8711db0c52ae360f0301b3f71c00a2bdfe54507b2b44572"
EXPECTED_RULES = 27_494


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    if TARGET.exists() and sha256(TARGET.read_bytes()) == GZ_SHA256:
        print(f"rulebook already materialized: {TARGET}")
        return

    parts = sorted(STAGING.glob("part_*.b64"))
    if not parts:
        raise SystemExit("rulebook asset is missing and no staging transport is available")

    encoded = "".join(p.read_text(encoding="ascii").strip() for p in parts)
    raw = bz2.decompress(base64.b64decode(encoded))
    if sha256(raw) != RAW_SHA256:
        raise SystemExit("raw rulebook SHA-256 mismatch")
    rules = sum(1 for line in raw.splitlines() if line.startswith(b"RULE\t"))
    if rules != EXPECTED_RULES:
        raise SystemExit(f"rule count mismatch: {rules}")

    gz = gzip.compress(raw, compresslevel=9, mtime=0)
    if sha256(gz) != GZ_SHA256:
        raise SystemExit("deterministic gzip SHA-256 mismatch")
    TARGET.parent.mkdir(parents=True, exist_ok=True)
    TARGET.write_bytes(gz)
    print(f"materialized {TARGET} ({len(gz)} bytes, {rules} rules)")


if __name__ == "__main__":
    main()
