#!/usr/bin/env python3
"""Prepare independently pinned originals for the maintainer downloader.

No original source/notice is edited. Python is not an installed CLI dependency.
"""
import argparse
from pathlib import Path
import shutil
import tempfile

import sources as s
import format_sources


def prepare(cache, output):
    s.require(not output.exists() and not output.is_symlink(), "Original output already exists")
    inventory = s.original_selection(s.read_json(s.HERE / "lock.json"), s.read_json(s.HERE / "source-policy.json"))
    s.require(inventory["files"], "No original resources selected")
    cache.mkdir(parents=True, exist_ok=True)
    s.require(cache.is_dir() and not cache.is_symlink(), "Original cache must be a regular directory")
    for package in inventory["packages"]:
        for entry in package["archives"]:
            s.fetch_one(cache, entry)
    receipts, notices = s.verify_packages(inventory, cache)
    files = format_sources.input_bytes(inventory, cache)
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=".pdf-originals-", dir=output.parent))
    try:
        for record in inventory["files"]:
            (stage / record["sha256"]).write_bytes(files[record["resource"]])
        (stage / "verification.json").write_bytes(s.encoded({"schemaVersion": 1, "matchedOriginals": receipts,
            "archives": [a for p in inventory["packages"] for a in p["archives"]],
            "notices": [{"path": n, "sha256": s.digest(v["bytes"])} for n, v in sorted(notices.items())]}))
        s.require(not output.exists() and not output.is_symlink(), "Original output appeared")
        stage.rename(output)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
    print(f"Prepared {len(files)} unchanged original resources")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cache", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    prepare(args.cache, args.output)
