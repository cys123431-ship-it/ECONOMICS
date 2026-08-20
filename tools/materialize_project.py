from __future__ import annotations
import base64, gzip, io, tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PARTS = ROOT / "transport"

def main() -> None:
    encoded = "".join(p.read_text(encoding="ascii").strip() for p in sorted(PARTS.glob("project_*.b64")))
    raw = gzip.decompress(base64.b64decode(encoded))
    with tarfile.open(fileobj=io.BytesIO(raw), mode="r:") as tf:
        for member in tf.getmembers():
            dest = (ROOT / member.name).resolve()
            if ROOT.resolve() not in dest.parents and dest != ROOT.resolve():
                raise SystemExit(f"unsafe archive path: {member.name}")
        tf.extractall(ROOT)
    print("materialized Rust project source")

if __name__ == "__main__":
    main()
