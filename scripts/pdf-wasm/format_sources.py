#!/usr/bin/env python3
"""Prepare original INITEX sources; maintainer-only, no prebuilt format input."""
import argparse
import copy
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import shutil
import tempfile

import sources as s

HERE = Path(__file__).resolve().parent


def select_resources(runtime, provenance, recipe):
    names = recipe["resources"]
    s.require(names and len(names) == len(set(names)), "Duplicate or empty format input selection")
    s.require(recipe["entrypoint"] == "lualatex.ini" and "26/lualatex.ini" in names,
              "Missing format entrypoint")
    s.require(type(recipe["sourceDateEpoch"]) is int and 0 <= recipe["sourceDateEpoch"] <= 2147483647,
              "Invalid format source date")
    records = {}
    for record in provenance["files"]:
        s.require(record["key"] not in records, "Duplicate format provenance")
        records[record["key"]] = record
    result = []
    for name in names:
        s.safe_path(name)
        record = records.get("pdftex/" + name)
        s.require(record is not None and "source" in record, f"Missing original format source: {name}")
        result.append({"name": name, "bytes": record["bytes"], "sha256": record["sha256"],
                       "url": runtime["mirror"] + record["key"]})
    return result


def inventory(cache):
    policy = s.read_json(HERE / "source-policy.json")
    runtime = s.read_json(HERE / "lock.json")
    recipe = s.read_json(HERE / "format-inputs.json")
    provenance = json.loads(s.verified(cache / policy["provenance"]["name"], policy["provenance"]))
    tlpdb = s.parse_tlpdb(s.verified(cache / policy["tlpdb"]["name"], policy["tlpdb"]).decode())
    artifacts = json.loads(s.verified(cache / policy["generatedManifest"]["name"], policy["generatedManifest"]))
    runtime["resources"] = select_resources(runtime, provenance, recipe)
    policy = copy.deepcopy(policy)
    policy["generatedResources"] = []
    policy["supportingNotices"] = []
    # null.tex has an explanatory source header, not a bundled license text.
    # Preserve it and the catalogue's Knuth terms without claiming notice closure.
    policy["additionalNotices"]["knuth-lib"] = ["texmf-dist/tex/generic/knuth-lib/null.tex"]
    policy["openItems"].append("Retain the Knuth source header and catalogue terms; null.tex contains no license text (https://ctan.org/pkg/knuth-lib, https://ctan.org/license/knuth).")
    result = s.build_inventory(runtime, provenance, tlpdb, policy, artifacts)
    result["purpose"] = "maintainer-format-source-verification"
    result["recipe"] = recipe
    return result


def input_bytes(inv, cache):
    result = {}
    for package in inv["packages"]:
        files = [f for f in inv["files"] if f["package"] == package["name"]]
        names = {f["path"]: f["path"].removeprefix("texmf-dist/") if package["relocated"] else f["path"] for f in files}
        for entry in package["archives"]:
            s.verified(cache / entry["name"], entry)
            found = s.archive_files(cache / entry["name"], set(names.values()))
            for f in files:
                if names[f["path"]] not in found:
                    continue
                data = found[names[f["path"]]]
                s.require(f["resource"] not in result, "Duplicate prepared format source")
                s.require(len(data) == f["bytes"] and s.digest(data) == f["sha256"], "Prepared format source differs")
                result[f["resource"]] = data
    s.require(set(result) == {f["resource"] for f in inv["files"]}, "Prepared format sources are incomplete")
    return result


def prepare(cache, output):
    s.require(cache.is_dir() and not cache.is_symlink(), "Format cache must be a regular directory")
    s.require(not output.exists() and not output.is_symlink(), "Format source output already exists")
    inv = inventory(cache)
    s.require(inv == s.read_json(HERE / "format-sources.lock.json"), "Format source inventory is stale")
    receipts, notices = s.verify_packages(inv, cache)
    data = input_bytes(inv, cache)
    report = {"schemaVersion": 1, "formatSourceLockSha256": s.digest(s.encoded(inv)),
              "matchedSources": receipts, "packageCount": len(inv["packages"]),
              "notices": [{"path": n, "archive": v["archive"], "sha256": s.digest(v["bytes"])} for n, v in sorted(notices.items())],
              "redistributionReviewComplete": False}
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=".format-sources-", dir=output.parent))
    try:
        for f in inv["files"]:
            (stage / f["sha256"]).write_bytes(data[f["resource"]])
        (stage / "verification.json").write_bytes(s.encoded(report))
        with (stage / "COLLECTED-NOTICES.txt").open("wb") as out:
            out.write(b"Original format source notices; redistribution review is separate.\n")
            for name, notice in sorted(notices.items()):
                out.write(("\n===== " + name + " =====\n").encode() + notice["bytes"] + b"\n")
        s.require(not output.exists() and not output.is_symlink(), "Format source output appeared")
        stage.rename(output)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
    print(f"Prepared {len(data)} original format inputs from {len(inv['packages'])} packages, with {len(notices)} notices")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("inventory", "fetch", "prepare"))
    parser.add_argument("cache", type=Path)
    parser.add_argument("output", type=Path, nargs="?")
    args = parser.parse_args()
    if args.command == "inventory":
        s.require(args.output is not None, "Specify a new format inventory file")
        data = s.encoded(inventory(args.cache))
        with args.output.open("xb") as out:
            out.write(data)
    elif args.command == "fetch":
        s.require(args.output is None, "fetch takes only a cache directory")
        args.cache.mkdir(parents=True, exist_ok=True)
        s.require(not args.cache.is_symlink(), "Format source cache cannot be a link")
        inv = s.read_json(HERE / "format-sources.lock.json")
        entries = inv["metadata"] + [a for p in inv["packages"] for a in p["archives"]]
        with ThreadPoolExecutor(max_workers=4) as pool:
            list(pool.map(lambda e: s.fetch_one(args.cache, e), entries))
        print(f"Verified {len(entries)} format source inputs")
    else:
        s.require(args.output is not None, "Specify a new format source output directory")
        prepare(args.cache, args.output)


if __name__ == "__main__":
    main()
