#!/usr/bin/env python3
"""Assemble a reproducible experimental PDF resource archive; never publish it.

This is maintainer tooling. Final notices/source distribution and production
cache integration are separate from this resource-only package experiment.
"""
import argparse
import gzip
import io
from pathlib import Path
import shutil
import tarfile
import tempfile

import sources as s

HERE = Path(__file__).resolve().parent
FONT_MAP = b"% Zellij uses OpenType fonts.\n"


def expected_manifest(runtime, format_recipe, font_recipe):
    replacements = {
        "51/luaotfload-names.lua": font_recipe["output"],
        "11/pdftex.map": {"bytes": len(FONT_MAP), "sha256": s.digest(FONT_MAP)},
    }
    clean = lambda entry: {key: entry[key] for key in ("name", "bytes", "sha256")}
    return {"schemaVersion": 1, "kind": "experimental-pdf-resources", "redistributionReviewComplete": False,
            "engine": [clean(e) for e in runtime["engine"] if e["name"].endswith((".js", ".wasm"))],
            "format": {"name": "zellij-lualatex.fmt", **format_recipe["output"]},
            "resources": [clean({**entry, **replacements.get(entry["name"], {})}) for entry in runtime["resources"]]}


def collect(bundle, format_dir, font_dir, runtime, format_recipe, font_recipe):
    for root in (bundle, format_dir, font_dir):
        s.require(root.is_dir() and not root.is_symlink(), "Compact input must be a regular directory")
    manifest = expected_manifest(runtime, format_recipe, font_recipe)
    blobs = {}
    for entry in manifest["engine"] + manifest["resources"] + [manifest["format"]]:
        name = entry["name"]
        if name == "zellij-lualatex.fmt":
            file = format_dir / name
        elif name in ("11/pdftex.map", "51/luaotfload-names.lua"):
            file = font_dir / name.split("/")[1]
        else:
            file = bundle / entry["sha256"]
        data = s.verified(file, entry)
        blobs[entry["sha256"]] = data
    return manifest, blobs


def package(bundle, format_dir, font_dir, output):
    s.require(not output.exists() and not output.is_symlink(), "Compact output already exists")
    runtime = s.read_json(HERE / "lock.json")
    format_recipe = s.read_json(HERE / "format-inputs.json")
    font_recipe = s.read_json(HERE / "font-inputs.json")
    manifest, blobs = collect(bundle, format_dir, font_dir, runtime, format_recipe, font_recipe)
    manifest_bytes = s.encoded(manifest)
    files = {**blobs, "manifest.json": manifest_bytes}
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=".compact-pdf-", dir=output.parent))
    try:
        root = stage / "package"
        root.mkdir()
        for name, data in files.items():
            (root / name).write_bytes(data)
        archive = stage / "experimental-pdf-resources.tar.gz"
        with archive.open("wb") as raw, gzip.GzipFile(filename="", fileobj=raw, mode="wb", compresslevel=9, mtime=0) as gz, \
                tarfile.open(fileobj=gz, mode="w|", format=tarfile.USTAR_FORMAT) as tar:
            for name, data in sorted(files.items()):
                info = tarfile.TarInfo(name)
                info.size, info.mode, info.mtime = len(data), 0o644, 0
                tar.addfile(info, io.BytesIO(data))
        archive_bytes = archive.read_bytes()
        report = {"schemaVersion": 1, "experimental": True, "redistributionReviewComplete": False,
                  "includesFinalNotices": False, "includesPrivateNode": False,
                  "runtimeLockSha256": s.digest(s.encoded(runtime)),
                  "formatRecipeSha256": s.digest(s.encoded(format_recipe)),
                  "fontRecipeSha256": s.digest(s.encoded(font_recipe)),
                  "manifestSha256": s.digest(manifest_bytes), "files": len(files),
                  "uncompressedBytes": sum(map(len, files.values())),
                  "archive": {"name": archive.name, "bytes": len(archive_bytes), "sha256": s.digest(archive_bytes)}}
        (stage / "verification.json").write_bytes(s.encoded(report))
        s.require(not output.exists() and not output.is_symlink(), "Compact output appeared")
        stage.rename(output)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
    print(f"Experimental resource archive: {report['archive']['bytes']} bytes; not release-ready")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", type=Path)
    parser.add_argument("format", type=Path)
    parser.add_argument("fonts", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    package(args.bundle, args.format, args.fonts, args.output)
