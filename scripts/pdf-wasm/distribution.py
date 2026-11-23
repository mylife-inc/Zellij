#!/usr/bin/env python3
"""Prepare private qualification archives of original PDF sources and notices.

This does not publish artifacts, change any license, or authorize redistribution.
No Python dependency is introduced into the installed CLI.
"""
import argparse
import copy
import gzip
import hashlib
import json
from pathlib import Path
import re
import shutil
import tarfile
import tempfile

import sources as s
import format_sources
import font_sources

HERE = Path(__file__).resolve().parent
LOCKS = ("sources.lock.json", "format-sources.lock.json", "font-sources.lock.json")


def merge_inventories(inventories):
    """Union matching originals, retaining every notice and rejecting conflicts."""
    packages, files, archives = {}, {}, {}
    metadata, supporting = {}, {}
    engine = None

    def insert(table, key, value, label):
        s.safe_path(key)
        s.require(key not in table or table[key] == value, f"Conflicting {label}: {key}")
        s.require(not any(k != key and k.casefold() == key.casefold() for k in table),
                  f"Case-colliding {label}: {key}")
        table[key] = copy.deepcopy(value)

    for inv in inventories:
        current = inv["engineSource"]
        s.require(engine is None or engine == current, "Conflicting engine source")
        engine = current
        for item in inv["metadata"]:
            insert(metadata, item["name"], item, "metadata")
        for item in inv.get("supportingNotices", []):
            insert(supporting, item["package"], item, "supporting notice")
        for package in inv["packages"]:
            name = package["name"]
            notices = package["noticePaths"]
            clean = {k: v for k, v in package.items() if k != "noticePaths"}
            if name in packages:
                old = packages[name]
                s.require({k: v for k, v in old.items() if k != "noticePaths"} == clean,
                          f"Conflicting package: {name}")
                notices = old["noticePaths"] + notices
            packages[name] = {**copy.deepcopy(clean), "noticePaths": sorted(set(notices))}
            for entry in package["archives"]:
                insert(archives, entry["name"], entry, "archive")
        for item in inv["files"]:
            insert(files, item["resource"], item, "source resource")
    s.require(engine is not None and packages and files, "Empty source selection")
    for entry in [engine] + [item["archive"] for item in supporting.values()] + list(metadata.values()):
        insert(archives, entry["name"], entry, "archive/metadata")
    for name in archives:
        s.require("/" not in name, "Upstream source names must be flat")
    return {"packages": [packages[k] for k in sorted(packages)],
            "files": [files[k] for k in sorted(files)], "engineSource": engine,
            "supportingNotices": [supporting[k] for k in sorted(supporting)]}, archives


def ec_listing(data):
    lines = data.decode("utf-8").splitlines()
    start = next((i for i, line in enumerate(lines) if line.split() == ["Length", "Date", "Name"]), None)
    s.require(start is not None, "EC listing header missing")
    result = {}
    count = None
    for line in lines[start + 1:]:
        text = line.strip()
        if not text or set(text) <= {"-", " "}:
            continue
        s.require(count is None, "Unexpected data after EC listing count")
        if text.isdecimal():
            count = int(text)
            continue
        match = re.fullmatch(r"\s*(\d+)\s+\d{2}-\d{2}-\d{2}\s+([A-Za-z0-9_.-]+)\s*", line)
        s.require(match is not None, "Malformed EC listing row")
        size, name = int(match[1]), match[2]
        s.require(name not in (".", "..") and name.casefold() not in {n.casefold() for n in result},
                  "Duplicate or unsafe EC filename")
        s.require(0 < size <= 64 * 1024 * 1024, "Invalid EC file size")
        result[name] = size
    s.require(result and count == len(result) and {"00files.txt", "copyrite.txt"} <= result.keys(),
              "Incomplete EC listing")
    return result


def complete_ec(cache, package):
    s.require(package["name"] == "ec" and package["relocated"], "Unexpected EC package")
    wanted_manifest = "doc/fonts/ec/00files.txt"
    manifest = None
    for entry in package["archives"]:
        s.verified(cache / entry["name"], entry)
        found = s.archive_files(cache / entry["name"], {wanted_manifest})
        if wanted_manifest in found:
            s.require(manifest is None, "Duplicate EC manifest")
            manifest = found[wanted_manifest]
    s.require(manifest is not None, "EC manifest missing")
    names = ec_listing(manifest)
    contents, receipts = {}, []
    for entry in package["archives"]:
        with tarfile.open(cache / entry["name"], "r:xz") as archive:
            for member in archive:
                name = member.name.rsplit("/", 1)[-1]
                if name not in names:
                    continue
                s.safe_path(member.name)
                s.require(name not in contents, f"Duplicate EC source: {name}")
                s.require(member.isfile() and member.size == names[name], f"Invalid EC source: {name}")
                data = archive.extractfile(member).read(names[name] + 1)
                s.require(len(data) == names[name], f"Truncated EC source: {name}")
                contents[name] = data
                receipts.append({"name": name, "archive": entry["name"], "member": member.name,
                                 "bytes": len(data), "sha256": s.digest(data)})
    s.require(contents.keys() == names.keys(), "EC complete distribution is missing files")
    s.require(contents["00files.txt"] == manifest, "EC manifest changed")
    return contents, sorted(receipts, key=lambda r: r["name"])


def verified_inputs(cache):
    s.require(cache.is_dir() and not cache.is_symlink(), "Source cache must be a regular directory")
    inventories = [s.inventory_from_cache(cache), format_sources.inventory(cache), font_sources.inventory(cache)]
    for name, inv in zip(LOCKS, inventories):
        s.require(inv == s.read_json(HERE / name), f"Source inventory is stale: {name}")
    merged, upstream = merge_inventories(inventories)
    matched, notices = s.verify_packages(merged, cache)
    engine, engine_notices = s.verify_engine(merged, s.read_json(HERE / "lock.json"), cache)
    ec, ec_receipts = complete_ec(cache, next(p for p in merged["packages"] if p["name"] == "ec"))
    return merged, upstream, matched, notices, engine, engine_notices, ec, ec_receipts


def file_record(path, name):
    info = path.lstat()
    s.require(path.is_file() and not path.is_symlink(), f"Nonregular distribution file: {name}")
    with path.open("rb") as source:
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    return {"name": s.safe_path(name), "bytes": info.st_size, "sha256": digest}


def write_bytes(root, name, data):
    file = root / s.safe_path(name)
    file.parent.mkdir(parents=True, exist_ok=True)
    with file.open("xb") as out:
        out.write(data)


def verify_archive(path, expected):
    records = {r["name"]: r for r in expected}
    s.require(len(records) == len(expected), "Duplicate archive inventory")
    seen = []
    with tarfile.open(path, "r:gz") as tar:
        for member in tar:
            s.require(member.name in records and member.name not in seen, "Unexpected archive member")
            record = records[member.name]
            s.require(member.isfile() and member.size == record["bytes"] and not member.pax_headers,
                      "Invalid archive member")
            s.require((member.mode, member.uid, member.gid, member.mtime, member.uname, member.gname)
                      == (0o644, 0, 0, 0, "", ""), "Noncanonical archive metadata")
            with tar.extractfile(member) as source:
                digest = hashlib.file_digest(source, "sha256").hexdigest()
            s.require(digest == record["sha256"], "Archive content checksum mismatch")
            seen.append(member.name)
    s.require(seen == sorted(records), "Incomplete or unordered archive")


def archive(root, output):
    """Fixed-order/timestamp gzip archive, with only regular pre-staged files."""
    files = []
    for item in sorted(root.rglob("*")):
        s.require(not item.is_symlink(), "Distribution cannot contain a symlink")
        if item.is_dir():
            continue
        files.append(file_record(item, item.relative_to(root).as_posix()))
    s.require(files, "Empty distribution archive")
    with output.open("xb") as raw, gzip.GzipFile(filename="", fileobj=raw, mode="wb", compresslevel=9, mtime=0) as gz, \
            tarfile.open(fileobj=gz, mode="w|", format=tarfile.USTAR_FORMAT) as tar:
        for record in files:
            info = tarfile.TarInfo(record["name"])
            info.size, info.mode, info.mtime = record["bytes"], 0o644, 0
            with (root / record["name"]).open("rb") as source:
                tar.addfile(info, source)
    verify_archive(output, files)
    return file_record(output, output.name), files


def assemble(cache, output):
    s.require(not output.exists() and not output.is_symlink(), "Distribution output already exists")
    merged, upstream, matched, notices, engine, engine_notices, ec, ec_receipts = verified_inputs(cache)
    recipe = s.read_json(HERE / "distribution-inputs.json")
    selected = recipe["buildFiles"]
    s.require(selected == sorted(set(selected)) and all("/" not in s.safe_path(n) for n in selected),
              "Invalid distribution build-file inventory")
    build_files = {}
    for name in selected:
        file = HERE / name
        s.require(file.is_file() and not file.is_symlink() and file.stat().st_size <= 8 * 1024 * 1024,
                  f"Missing regular build source: {name}")
        build_files[name] = file.read_bytes()
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=".pdf-distribution-", dir=output.parent))
    try:
        source_root, notice_root = stage / "sources", stage / "notices"
        source_root.mkdir()
        notice_root.mkdir()
        notice_records = []
        for kind, entries in [("tex", notices), ("engine", {n: {"bytes": b} for n, b in engine_notices.items()})]:
            for name, item in sorted(entries.items()):
                data = item["bytes"]
                target = "licenses/" + s.digest(data) + ".txt"
                if not (notice_root / target).exists():
                    write_bytes(notice_root, target, data)
                notice_records.append({"kind": kind, "originalPath": name, "file": target,
                                       "archive": item.get("archive", merged["engineSource"]["name"]),
                                       "bytes": len(data), "sha256": s.digest(data)})
        write_bytes(notice_root, "ZELLIJ-ADAPTER-NOTICE.txt", build_files["NOTICE"])
        for name, data in ec.items():
            write_bytes(notice_root, "originals/ec/" + name, data)
        index = {"schemaVersion": 1, "experimental": True, "redistributionReviewComplete": False,
                 "sourceArchive": "experimental-pdf-sources.tar.gz", "engine": engine,
                 "packages": merged["packages"], "notices": notice_records, "completeEcDistribution": ec_receipts,
                 "reviewItems": recipe["reviewItems"]}
        write_bytes(notice_root, "NOTICES.json", s.encoded(index))
        write_bytes(notice_root, "README.md", build_files["distribution-notices.md"])
        for name, entry in sorted(upstream.items()):
            # Validate the exact bytes being copied, including inputs changed
            # after correspondence checking. A failed build leaves no output.
            write_bytes(source_root, "upstream/" + name, s.verified(cache / name, entry))
        for name, data in build_files.items():
            write_bytes(source_root, "scripts/pdf-wasm/" + name, data)
        write_bytes(source_root, "README.md", build_files["distribution-sources.md"])
        for file in notice_root.rglob("*"):
            if file.is_file():
                write_bytes(source_root, "notices/" + file.relative_to(notice_root).as_posix(), file.read_bytes())
        source_files = [file_record(p, p.relative_to(source_root).as_posix())
                        for p in sorted(source_root.rglob("*")) if p.is_file()]
        manifest = {"schemaVersion": 1, "experimental": True, "redistributionReviewComplete": False,
                    "sourceLocks": {n: s.digest(build_files[n]) for n in LOCKS},
                    "matchedOriginals": matched, "files": source_files, "reviewItems": recipe["reviewItems"]}
        write_bytes(source_root, "SOURCE-MANIFEST.json", s.encoded(manifest))
        notice_archive, notice_files = archive(notice_root, stage / "experimental-pdf-notices.tar.gz")
        source_archive, all_source_files = archive(source_root, stage / "experimental-pdf-sources.tar.gz")
        report = {"schemaVersion": 1, "experimental": True, "redistributionReviewComplete": False,
                  "sourceLocks": manifest["sourceLocks"], "packageCount": len(merged["packages"]),
                  "matchedOriginals": len(matched), "upstreamInputs": len(upstream),
                  "noticeCount": len(notice_records), "completeEcFiles": len(ec_receipts),
                  "notices": {"archive": notice_archive, "files": len(notice_files)},
                  "sources": {"archive": source_archive, "files": len(all_source_files)},
                  "sourceManifestSha256": s.digest(s.encoded(manifest)), "reviewItems": recipe["reviewItems"]}
        write_bytes(stage, "verification.json", s.encoded(report))
        s.require(not output.exists() and not output.is_symlink(), "Distribution output appeared")
        stage.rename(output)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
    print(f"Prepared {report['packageCount']} original packages and {report['completeEcFiles']} EC files; not published")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cache", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    assemble(args.cache, args.output)
