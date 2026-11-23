#!/usr/bin/env python3
"""Maintainer-only provenance checks. No compiler execution or tar extraction.

Python is used only by maintainers/CI, never by installed zel PDF rendering.
This verifies source correspondence; it does not grant redistribution approval.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import tarfile
import tempfile
import time
import urllib.request

HERE = Path(__file__).resolve().parent


def require(ok, message):
    if not ok:
        raise ValueError(message)


def digest(data, algorithm="sha256"):
    return hashlib.new(algorithm, data).hexdigest()


def read_json(path):
    return json.loads(path.read_bytes())


def encoded(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + "\n").encode()


def safe_path(name):
    require(isinstance(name, str) and name and not name.startswith("/")
            and "\\" not in name and all(p not in ("", ".", "..") for p in name.split("/"))
            and re.fullmatch(r"[A-Za-z0-9_.+/@ -]+", name), f"Unsafe source path: {name!r}")
    return name


def verified(path, entry):
    require(path.is_file() and not path.is_symlink(), f"Missing regular source file: {path.name}")
    size = path.stat().st_size
    require(size <= 256 * 1024 * 1024 and ("bytes" not in entry or size == entry["bytes"]),
            f"Source size mismatch: {path.name} ({size} bytes, expected {entry.get('bytes', 'at most 256 MiB')})")
    data = path.read_bytes()
    for algorithm in ("sha256", "sha512"):
        if algorithm in entry:
            require(digest(data, algorithm) == entry[algorithm], f"Source checksum mismatch: {path.name}")
    require("sha256" in entry or "sha512" in entry, "Source checksum missing")
    return data


def parse_tlpdb(text):
    result = {}
    for block in text.split("\n\n"):
        match = re.match(r"name ([a-zA-Z0-9_.-]+)\n", block)
        if not match:
            continue
        name = match[1]
        require(name not in result, f"Duplicate TeX package: {name}")
        fields = {}
        for line in block.splitlines():
            if line and not line.startswith(" "):
                key, _, value = line.partition(" ")
                if key in ("name", "revision", "relocated", "catalogue-license") or "container" in key:
                    require(key not in fields, f"Duplicate TeX field: {name}/{key}")
                    fields[key] = value
        result[name] = fields
    return result


def build_inventory(runtime, provenance, tlpdb, policy, artifacts):
    require(artifacts["provenanceSha256"] == policy["provenance"]["sha256"], "Generated resource provenance mismatch")
    require(artifacts["mirrorRevision"] == provenance["mirrorRevision"], "Generated resource mirror mismatch")
    require(provenance["source"]["repository"]["url"] == policy["repository"], "TeX repository mismatch")
    require(provenance["source"]["tlpdb"]["sha256"] == policy["tlpdb"]["sha256"], "TeX database mismatch")
    records = {}
    for record in provenance["files"] + artifacts["artifacts"]:
        require(record["key"] not in records, "Duplicate provenance key")
        records[record["key"]] = record
    packages, files, generated = {}, [], []
    seen = set()
    for resource in runtime["resources"]:
        require(resource["name"] not in seen, "Duplicate runtime resource")
        seen.add(resource["name"])
        require(resource["url"].startswith(runtime["mirror"]), "Resource outside the pinned mirror")
        key = resource["url"][len(runtime["mirror"]):]
        record = records.get(key)
        require(record is not None, f"Resource provenance missing: {key}")
        require(record["sha256"] == resource["sha256"] and record.get("bytes", record.get("size")) == resource["bytes"],
                f"Resource provenance mismatch: {key}")
        if "source" not in record:
            require(resource["name"] in policy["generatedResources"], f"Unreviewed generated resource: {key}")
            generated.append({"resource": resource["name"], "sha256": resource["sha256"], "kind": record.get("kind")})
            continue
        source = record["source"]
        name = source["package"]
        require(re.fullmatch(r"[a-zA-Z0-9_.-]+", name), "Unsafe package name")
        package = tlpdb.get(name)
        require(package is not None and package["revision"] == str(source["packageRevision"]),
                f"TeX package revision mismatch: {name}")
        require(source["license"]["ids"], f"Missing package license metadata: {name}")
        notices = sorted(set(source["noticePaths"] + policy["additionalNotices"].get(name, [])))
        require(notices, f"Missing notice locations: {name}")
        for notice in notices:
            safe_path(notice)
        item = {"name": name, "revision": package["revision"], "relocated": package.get("relocated") == "1",
                "license": source["license"], "noticePaths": notices, "archives": []}
        for prefix, suffix in (("", ""), ("doc", ".doc"), ("src", ".source")):
            if prefix + "containersize" not in package:
                continue
            checksum = package.get(prefix + "containerchecksum", "")
            require(re.fullmatch(r"[a-f0-9]{128}", checksum), f"Missing archive checksum: {name}{suffix}")
            size = int(package[prefix + "containersize"])
            require(0 < size <= 256 * 1024 * 1024, "Unsupported source archive size")
            filename = name + suffix + ".tar.xz"
            item["archives"].append({"name": filename, "kind": prefix or "run", "bytes": size,
                                     "sha512": checksum, "url": policy["repository"] + "archive/" + filename})
        require(any(a["kind"] == "run" for a in item["archives"]), f"Missing runtime source archive: {name}")
        require(name not in packages or packages[name] == item, f"Inconsistent package provenance: {name}")
        packages[name] = item
        files.append({"resource": resource["name"], "path": safe_path(source["path"]), "package": name,
                      "bytes": resource["bytes"], "sha256": resource["sha256"]})
    require(set(policy["generatedResources"]) == {f["resource"] for f in generated}, "Generated resource policy is stale")
    supporting = []
    for supplement in policy.get("supportingNotices", []):
        package = tlpdb.get(supplement["package"])
        require(package is not None and package["revision"] == supplement["revision"], "Supporting notice revision mismatch")
        checksum = package.get("doccontainerchecksum", "")
        require(re.fullmatch(r"[a-f0-9]{128}", checksum), "Supporting notice archive checksum missing")
        filename = supplement["package"] + ".doc.tar.xz"
        supporting.append({"package": supplement["package"], "revision": supplement["revision"],
                           "paths": [safe_path(p) for p in supplement["paths"]],
                           "relocated": package.get("relocated") == "1",
                           "archive": {"name": filename, "bytes": int(package["doccontainersize"]), "sha512": checksum,
                                       "url": policy["repository"] + "archive/" + filename}})
    return {"schemaVersion": 1, "purpose": "maintainer-source-verification", "redistributionReviewComplete": False,
            "runtimeLockSha256": digest(encoded(runtime)), "policySha256": digest(encoded(policy)),
            "metadata": [policy["provenance"], policy["tlpdb"], policy["generatedManifest"]],
            "engineSource": {"name": "corresponding-source.tar.xz", **runtime["correspondingSource"]},
            "packages": [packages[k] for k in sorted(packages)],
            "files": sorted(files, key=lambda f: f["resource"]),
            "generatedResources": sorted(generated, key=lambda f: f["resource"]),
            "supportingNotices": supporting,
            "openItems": policy["openItems"]}


def inventory_from_cache(cache):
    policy = read_json(HERE / "source-policy.json")
    runtime = read_json(HERE / "lock.json")
    provenance = json.loads(verified(cache / policy["provenance"]["name"], policy["provenance"]))
    tlpdb = verified(cache / policy["tlpdb"]["name"], policy["tlpdb"]).decode()
    artifacts = json.loads(verified(cache / policy["generatedManifest"]["name"], policy["generatedManifest"]))
    return build_inventory(runtime, provenance, parse_tlpdb(tlpdb), policy, artifacts)


def archive_files(path, wanted):
    """Read only explicit regular members; never extract archive paths/links."""
    result = {}
    with tarfile.open(path, "r:xz") as archive:
        seen = set()
        for item in archive:
            if item.name not in wanted:
                continue
            require(item.name not in seen, f"Duplicate source member: {item.name}")
            seen.add(item.name)
            require(item.isfile() and 0 <= item.size <= 64 * 1024 * 1024,
                    f"Unsafe source member: {item.name}")
            data = archive.extractfile(item).read(64 * 1024 * 1024 + 1)
            require(len(data) == item.size, f"Source member size mismatch: {item.name}")
            result[item.name] = data
    return result


def verify_packages(inventory, cache):
    receipts, notice_data = [], {}
    for package in inventory["packages"]:
        needed = {f["path"] for f in inventory["files"] if f["package"] == package["name"]}
        needed.update(package["noticePaths"])
        def member(name):
            if package["relocated"]:
                require(name.startswith("texmf-dist/"), "Invalid relocated TeX path")
                return name[len("texmf-dist/"):]
            return name
        found = {}
        for entry in package["archives"]:
            archive = cache / entry["name"]
            verified(archive, entry)
            current = archive_files(archive, {member(name) for name in needed})
            require(not set(current).intersection(found), f"Duplicate package source across archives: {package['name']}")
            found.update({name: (data, entry["name"]) for name, data in current.items()})
        for source in (f for f in inventory["files"] if f["package"] == package["name"]):
            require(member(source["path"]) in found, f"Runtime source missing: {source['path']}")
            data, archive = found[member(source["path"])]
            require(len(data) == source["bytes"] and digest(data) == source["sha256"],
                    f"Runtime source differs: {source['resource']}")
            receipts.append({"resource": source["resource"], "archive": archive, "path": source["path"], "sha256": digest(data)})
        for notice in package["noticePaths"]:
            require(member(notice) in found, f"Notice missing: {notice}")
            data, archive = found[member(notice)]
            require(data.strip(), f"Empty notice: {notice}")
            notice_data[notice] = {"bytes": data, "archive": archive}
    for supplement in inventory.get("supportingNotices", []):
        entry = supplement["archive"]
        verified(cache / entry["name"], entry)
        names = {p: p.removeprefix("texmf-dist/") if supplement["relocated"] else p for p in supplement["paths"]}
        found = archive_files(cache / entry["name"], set(names.values()))
        for name, member in names.items():
            require(member in found and found[member].strip(), f"Supporting notice missing: {name}")
            require(name not in notice_data, f"Duplicate supporting notice: {name}")
            notice_data[name] = {"bytes": found[member], "archive": entry["name"]}
    return receipts, notice_data


def verify_engine(inventory, runtime, cache):
    entry = inventory["engineSource"]
    archive = cache / entry["name"]
    verified(archive, entry)
    prefix = "wasmtex-" + runtime["engineRelease"] + "-source/"
    paths = ["SOURCE-MANIFEST.json", "REBUILD.md", "RELINK.md", "release/manifest.json", "release/BUILD-RECEIPT.luahbtex.json"]
    data = archive_files(archive, {prefix + p for p in paths})
    require(len(data) == len(paths), "Engine source records are incomplete")
    manifest = json.loads(data[prefix + "release/manifest.json"])
    source = json.loads(data[prefix + "SOURCE-MANIFEST.json"])
    require(manifest["releaseId"] == runtime["engineRelease"] == source["releaseId"], "Engine source release mismatch")
    require(source["engineAssetManifest"]["sha256"] == digest(data[prefix + "release/manifest.json"]), "Engine source manifest mismatch")
    files = {f["name"]: f for f in manifest["files"]}
    core = [e for e in runtime["engine"] if e["name"].endswith((".js", ".wasm", ".fmt.gz"))]
    for item in core:
        require(files.get(item["name"]) == {k: item[k] for k in ("name", "bytes", "sha256")},
                f"Engine source does not match: {item['name']}")
    receipt_bytes = data[prefix + "release/BUILD-RECEIPT.luahbtex.json"]
    expected_receipt = next(e for e in runtime["engine"] if e["name"] == "BUILD-RECEIPT.luahbtex.json")
    require(digest(receipt_bytes) == expected_receipt["sha256"] and len(receipt_bytes) == expected_receipt["bytes"],
            "Engine build receipt mismatch")
    receipt = json.loads(receipt_bytes)
    generation = receipt["generationReceipts"]
    snapshots = {s["commit"]: s for s in source["sources"]["wasmtex"]}
    for part in ("engine", "formats"):
        require(generation[part]["sourceRevision"] in snapshots, f"Missing {part} source generation")
    # Preserve all bundled component notices from the exact engine source snapshot.
    snapshot = snapshots[generation["engine"]["sourceRevision"]]["path"]
    with tarfile.open(archive, "r:xz") as tar:
        names = {m.name for m in tar if m.name.startswith(prefix + snapshot + "/LICENSES/") and m.isfile()}
    names.update({prefix + snapshot + "/LICENSE", prefix + snapshot + "/THIRD_PARTY_NOTICES.md"})
    notices = archive_files(archive, names)
    require(len(notices) == len(names) and len(notices) > 10 and all(data.strip() for data in notices.values()),
            "Missing engine component notices")
    return {"release": runtime["engineRelease"], "matchedArtifacts": [e["name"] for e in core],
            "engineSourceRevision": generation["engine"]["sourceRevision"],
            "formatSourceRevision": generation["formats"]["sourceRevision"], "rebuildVerified": False}, notices


def download_one(cache, entry):
    name = safe_path(entry["name"])
    require("/" not in name, "Source cache names must be flat")
    destination = cache / name
    if destination.exists() or destination.is_symlink():
        verified(destination, entry)
        return
    require(entry["url"].startswith("https://"), "Source downloads require HTTPS")
    fd, temporary = tempfile.mkstemp(prefix=".source-download-", dir=cache)
    try:
        with os.fdopen(fd, "wb") as output, urllib.request.urlopen(entry["url"], timeout=120) as response:
            size = 0
            while chunk := response.read(1024 * 1024):
                size += len(chunk)
                require(size <= entry.get("bytes", 256 * 1024 * 1024), "Source download exceeds limit")
                output.write(chunk)
        verified(Path(temporary), entry)
        require(not destination.exists(), "Source download destination appeared")
        os.rename(temporary, destination)
    finally:
        Path(temporary).unlink(missing_ok=True)


def fetch_one(cache, entry):
    # Retry incomplete transport responses only while no cache file is published.
    # An already present corrupt file is left intact and rejected for inspection.
    destination = cache / safe_path(entry["name"])
    if destination.exists() or destination.is_symlink():
        verified(destination, entry)
        return
    for attempt in range(3):
        try:
            return download_one(cache, entry)
        except (OSError, ValueError):
            if attempt == 2:
                raise
            time.sleep(attempt + 1)


def check(cache, output):
    require(cache.is_dir() and not cache.is_symlink(), "Source cache must be a regular directory")
    require(not output.exists() and not output.is_symlink(), "Source verification output already exists")
    inventory = inventory_from_cache(cache)
    require(inventory == read_json(HERE / "sources.lock.json"), "Source inventory is stale; regenerate and review it")
    receipts, notices = verify_packages(inventory, cache)
    engine, engine_notices = verify_engine(inventory, read_json(HERE / "lock.json"), cache)
    report = {"schemaVersion": 1, "redistributionReviewComplete": False, "inventorySha256": digest(encoded(inventory)),
              "packageCount": len(inventory["packages"]), "archiveCount": sum(len(p["archives"]) for p in inventory["packages"]),
              "matchedResources": receipts, "generatedResources": inventory["generatedResources"], "engine": engine,
              "supportingArchiveCount": len(inventory["supportingNotices"]),
              "notices": [{"path": n, "archive": v["archive"], "sha256": digest(v["bytes"])} for n, v in sorted(notices.items())],
              "engineNotices": [{"path": n, "sha256": digest(v)} for n, v in sorted(engine_notices.items())],
              "openItems": inventory["openItems"]}
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=".pdf-source-check-", dir=output.parent))
    try:
        (stage / "verification.json").write_bytes(encoded(report))
        with (stage / "COLLECTED-NOTICES.txt").open("wb") as out:
            out.write(b"Maintainer source evidence, not a redistribution approval. See verification.json for open items.\n")
            for name, value in sorted(notices.items()):
                out.write(("\n===== " + name + " =====\n").encode() + value["bytes"] + b"\n")
            for name, data in sorted(engine_notices.items()):
                out.write(("\n===== " + name + " =====\n").encode() + data + b"\n")
        require(not output.exists(), "Source verification output appeared")
        stage.rename(output)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
    print(f"Verified {len(receipts)} resource files, {len(inventory['packages'])} packages, {len(notices)} TeX notices and {len(engine_notices)} engine notices; redistribution review remains open.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("inventory", "fetch", "verify"))
    parser.add_argument("cache", type=Path)
    parser.add_argument("output", type=Path, nargs="?")
    args = parser.parse_args()
    if args.command == "inventory":
        require(args.output is not None, "Specify a new inventory output file")
        data = encoded(inventory_from_cache(args.cache))
        with args.output.open("xb") as out:
            out.write(data)
    elif args.command == "fetch":
        require(args.output is None, "fetch takes only a cache directory")
        args.cache.mkdir(parents=True, exist_ok=True)
        require(not args.cache.is_symlink(), "Source cache cannot be a link")
        inventory = read_json(HERE / "sources.lock.json")
        entries = inventory["metadata"] + [inventory["engineSource"]] + [a for p in inventory["packages"] for a in p["archives"]] + [s["archive"] for s in inventory["supportingNotices"]]
        with ThreadPoolExecutor(max_workers=4) as pool:
            list(pool.map(lambda entry: fetch_one(args.cache, entry), entries))
        print(f"Verified {len(entries)} cached source inputs")
    else:
        require(args.output is not None, "Specify a new verification output folder")
        check(args.cache, args.output)


if __name__ == "__main__":
    main()
