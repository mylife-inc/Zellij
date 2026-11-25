#!/usr/bin/env python3
"""Assemble a versioned, self-contained PDF qualification package; never publish.

The optional compiler has no Node binary or source tarball inside it. Its
manifest binds the separate corresponding-source archive by size and SHA-256.
Adapter licensing is scoped and approved; final compiler distribution review remains open.
"""
import argparse
import json
from pathlib import Path
import re
import shutil
import tarfile
import tempfile

import compact
import distribution as d
import sources as s

HERE = Path(__file__).resolve().parent
RUNNER_FILES = ("bundle.mjs", "font-inputs.json", "format-inputs.json", "lock.json",
                "pass.mjs", "probe.mjs", "process.mjs")


def qualified_sources(root):
    """Verify the source archive itself, not only a neighboring receipt/tree."""
    report = s.read_json(root / "verification.json")
    s.require(report["redistributionReviewComplete"] is False, "Unexpected distribution review state")
    record = report["sources"]["archive"]
    s.require(record["name"] == "experimental-pdf-sources.tar.gz", "Unexpected source archive")
    archive = root / record["name"]
    s.require(d.file_record(archive, archive.name) == record, "Source archive changed")
    with tarfile.open(archive, "r:gz") as tar:
        member = tar.getmember("SOURCE-MANIFEST.json")
        s.require(member.isfile() and member.size <= 1024 * 1024, "Invalid source manifest")
        manifest_bytes = tar.extractfile(member).read()
    s.require(s.digest(manifest_bytes) == report["sourceManifestSha256"], "Source manifest changed")
    manifest = json.loads(manifest_bytes)
    records = manifest["files"] + [{"name": "SOURCE-MANIFEST.json", "bytes": len(manifest_bytes),
                                   "sha256": s.digest(manifest_bytes)}]
    d.verify_archive(archive, records)
    selected = {}
    with tarfile.open(archive, "r:gz") as tar:
        for member in tar:
            if member.name.startswith(("notices/", "scripts/pdf-wasm/")):
                s.require(member.size <= 8 * 1024 * 1024, "Oversized toolchain source")
                selected[member.name] = tar.extractfile(member).read()
    # Refuse a stale source archive, including changes to the packaging recipe.
    for name in s.read_json(HERE / "distribution-inputs.json")["buildFiles"]:
        file = HERE / name
        s.require(file.is_file() and not file.is_symlink(), "Nonregular build source")
        s.require(selected.get("scripts/pdf-wasm/" + name) == file.read_bytes(),
                  f"Source archive is stale: {name}")
    return report, selected


def collect(resources, distribution):
    report, original = qualified_sources(distribution)
    expected = compact.expected_manifest(s.read_json(HERE / "lock.json"),
                                         s.read_json(HERE / "format-inputs.json"),
                                         s.read_json(HERE / "font-inputs.json"))
    s.require(resources.is_dir() and not resources.is_symlink(), "Invalid resource directory")
    manifest_file = resources / "manifest.json"
    s.require(manifest_file.is_file() and not manifest_file.is_symlink()
              and manifest_file.stat().st_size <= 256 * 1024, "Invalid resource manifest")
    manifest_bytes = manifest_file.read_bytes()
    s.require(json.loads(manifest_bytes) == expected, "Resource manifest differs from reviewed inventory")
    files = {"resources/manifest.json": manifest_bytes}
    for entry in expected["engine"] + expected["resources"] + [expected["format"]]:
        files["resources/" + entry["sha256"]] = s.verified(resources / entry["sha256"], entry)
    s.require({p.name for p in resources.iterdir()} == {n.removeprefix("resources/") for n in files},
              "Unlisted resource files")
    for name in RUNNER_FILES:
        files["runner/" + name] = original["scripts/pdf-wasm/" + name]
    files.update({n: b for n, b in original.items() if n.startswith("notices/")})
    return report, files


def package(resources, distribution, output, version):
    s.require(re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?", version), "Invalid toolchain version")
    s.require(not output.exists() and not output.is_symlink(), "Toolchain output already exists")
    report, files = collect(resources, distribution)
    files = dict(files)
    manifest = {"format": 1, "kind": "experimental-pdf-toolchain", "version": version,
                "redistribution_review_complete": False, "node_major": 24,
                "source_archive": report["sources"]["archive"],
                "source_manifest_sha256": report["sourceManifestSha256"],
                "files": {n: {"bytes": len(b), "sha256": s.digest(b)} for n, b in sorted(files.items())}}
    files["manifest.json"] = s.encoded(manifest)
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=".pdf-toolchain-", dir=output.parent))
    try:
        root = stage / "package"
        root.mkdir()
        for name, data in files.items():
            d.write_bytes(root, name, data)
        archive, entries = d.archive(root, stage / f"zel-pdf-qualification-v{version}.tar.gz")
        d.write_bytes(stage, archive["name"] + ".sha256",
                      f"{archive['sha256']}  {archive['name']}\n".encode())
        result = {"schemaVersion": 1, "experimental": True, "redistributionReviewComplete": False,
                  "version": version, "archive": archive, "files": len(entries),
                  "uncompressedBytes": sum(e["bytes"] for e in entries),
                  "manifestSha256": s.digest(files["manifest.json"]),
                  "sourceArchive": manifest["source_archive"],
                  "sourceManifestSha256": manifest["source_manifest_sha256"],
                  "includesPrivateNode": False, "includesSourceArchive": False}
        d.write_bytes(stage, "verification.json", s.encoded(result))
        s.require(not output.exists() and not output.is_symlink(), "Toolchain output appeared")
        stage.rename(output)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
    print(f"PDF qualification package: {archive['bytes']} bytes; not published")
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("resources", type=Path)
    parser.add_argument("distribution", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--version", required=True)
    args = parser.parse_args()
    package(args.resources, args.distribution, args.output, args.version)
