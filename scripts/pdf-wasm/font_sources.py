#!/usr/bin/env python3
"""Prepare original font-database inputs; maintainer tooling, never installed zel."""
import argparse
import copy
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import shutil
import tempfile

import sources as s
from format_sources import input_bytes

HERE = Path(__file__).resolve().parent


def select_resources(runtime, provenance, recipe):
    names = recipe["resources"]
    s.require(names and len(names) == len(set(names)), "Duplicate or empty font input selection")
    s.require(recipe["entrypoint"] == "luaotfload-tool.lua" and "51/luaotfload-tool.lua" in names,
              "Missing font generator entrypoint")
    s.require(type(recipe["sourceDateEpoch"]) is int and 0 <= recipe["sourceDateEpoch"] <= 2147483647,
              "Invalid font source date")
    fonts = {f["name"] for f in runtime["resources"] if f["name"].endswith(".otf")}
    s.require(fonts and fonts == {n for n in names if n.endswith(".otf")}, "Font selection differs from runtime")
    records = {}
    for record in provenance["files"]:
        s.require(record["key"] not in records, "Duplicate font provenance")
        records[record["key"]] = record
    result = []
    for name in names:
        s.safe_path(name)
        record = records.get("pdftex/" + name)
        s.require(record is not None and "source" in record, f"Missing original font source: {name}")
        result.append({"name": name, "bytes": record["bytes"], "sha256": record["sha256"],
                       "url": runtime["mirror"] + record["key"]})
    return result


def inventory(cache):
    policy = s.read_json(HERE / "source-policy.json")
    runtime = s.read_json(HERE / "lock.json")
    recipe = s.read_json(HERE / "font-inputs.json")
    provenance = json.loads(s.verified(cache / policy["provenance"]["name"], policy["provenance"]))
    tlpdb = s.parse_tlpdb(s.verified(cache / policy["tlpdb"]["name"], policy["tlpdb"]).decode())
    artifacts = json.loads(s.verified(cache / policy["generatedManifest"]["name"], policy["generatedManifest"]))
    runtime["resources"] = select_resources(runtime, provenance, recipe)
    policy = copy.deepcopy(policy)
    policy["generatedResources"] = []
    policy["supportingNotices"] = []
    result = s.build_inventory(runtime, provenance, tlpdb, policy, artifacts)
    result["purpose"] = "maintainer-font-source-verification"
    result["recipe"] = recipe
    return result


def prepare(cache, output):
    s.require(cache.is_dir() and not cache.is_symlink(), "Font cache must be a regular directory")
    s.require(not output.exists() and not output.is_symlink(), "Font source output already exists")
    inv = inventory(cache)
    s.require(inv == s.read_json(HERE / "font-sources.lock.json"), "Font source inventory is stale")
    receipts, notices = s.verify_packages(inv, cache)
    data = input_bytes(inv, cache)
    report = {"schemaVersion": 1, "fontSourceLockSha256": s.digest(s.encoded(inv)),
              "matchedSources": receipts, "packageCount": len(inv["packages"]),
              "notices": [{"path": n, "archive": v["archive"], "sha256": s.digest(v["bytes"])} for n, v in sorted(notices.items())],
              "redistributionReviewComplete": False}
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=".font-sources-", dir=output.parent))
    try:
        for f in inv["files"]:
            (stage / f["sha256"]).write_bytes(data[f["resource"]])
        (stage / "verification.json").write_bytes(s.encoded(report))
        with (stage / "COLLECTED-NOTICES.txt").open("wb") as out:
            out.write(b"Original font metadata source notices; redistribution review is separate.\n")
            for name, notice in sorted(notices.items()):
                out.write(("\n===== " + name + " =====\n").encode() + notice["bytes"] + b"\n")
        s.require(not output.exists() and not output.is_symlink(), "Font source output appeared")
        stage.rename(output)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
    print(f"Prepared {len(data)} original font inputs from {len(inv['packages'])} packages, with {len(notices)} notices")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("inventory", "fetch", "prepare"))
    parser.add_argument("cache", type=Path)
    parser.add_argument("output", type=Path, nargs="?")
    args = parser.parse_args()
    if args.command == "inventory":
        s.require(args.output is not None, "Specify a new font inventory file")
        data = s.encoded(inventory(args.cache))
        with args.output.open("xb") as out:
            out.write(data)
    elif args.command == "fetch":
        s.require(args.output is None, "fetch takes only a cache directory")
        args.cache.mkdir(parents=True, exist_ok=True)
        s.require(not args.cache.is_symlink(), "Font source cache cannot be a link")
        inv = s.read_json(HERE / "font-sources.lock.json")
        entries = inv["metadata"] + [a for p in inv["packages"] for a in p["archives"]]
        with ThreadPoolExecutor(max_workers=4) as pool:
            list(pool.map(lambda e: s.fetch_one(args.cache, e), entries))
        print(f"Verified {len(entries)} font source inputs")
    else:
        s.require(args.output is not None, "Specify a new font source output directory")
        prepare(args.cache, args.output)


if __name__ == "__main__":
    main()
