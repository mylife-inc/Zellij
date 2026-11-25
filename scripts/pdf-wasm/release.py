#!/usr/bin/env python3
"""Build paired, reviewed compiler/source assets. This command never publishes."""
import argparse
from pathlib import Path
import re
import shutil
import tempfile

import distribution as d
import sources as s
import toolchain


def package(cache, resources, output, version):
    s.require(re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?", version), "Invalid release version")
    s.require(not output.exists() and not output.is_symlink(), "Release output already exists")
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=".pdf-release-", dir=output.parent))
    try:
        source = d.assemble(cache, stage / "distribution", version)
        runtime = toolchain.package(resources, stage / "distribution", stage / "toolchain", version, True)
        s.require(runtime["sourceArchive"] == source["sources"]["archive"], "Runtime/source pairing failed")
        assets = stage / "artifacts"
        assets.mkdir()
        for directory, record in [("distribution", source["sources"]["archive"]), ("toolchain", runtime["archive"])]:
            file = stage / directory / record["name"]
            s.require(d.file_record(file, file.name) == record, "Release asset changed")
            shutil.copyfile(file, assets / file.name)
            d.write_bytes(assets, file.name + ".sha256", f"{record['sha256']}  {file.name}\n".encode())
        report = {"schemaVersion": 1, "version": version, "redistributionReviewComplete": True,
                  "published": False, "reviewSha256": source["reviewSha256"], "runtime": runtime["archive"],
                  "source": source["sources"]["archive"], "sourceManifestSha256": source["sourceManifestSha256"]}
        d.write_bytes(stage, "verification.json", s.encoded(report))
        s.require(not output.exists() and not output.is_symlink(), "Release output appeared")
        stage.rename(output)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
    print("Built matching reviewed runtime and source assets; not published")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cache", type=Path)
    parser.add_argument("resources", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--version", required=True)
    args = parser.parse_args()
    package(args.cache, args.resources, args.output, args.version)
