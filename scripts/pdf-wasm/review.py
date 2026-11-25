#!/usr/bin/env python3
"""Bind an explicit maintainer distribution review to exact original inputs.

Inventory generation does not approve a release. The reviewed decision is a
separate, checked-in record; changes to its input set require renewed review.
"""
import argparse
import json
from pathlib import Path
import re

import sources as s

HERE = Path(__file__).resolve().parent
BOUND_FILES = ("lock.json", "format-inputs.json", "font-inputs.json", "source-policy.json",
               "sources.lock.json", "format-sources.lock.json", "font-sources.lock.json",
               "LICENSE", "LICENSE-SCOPE.md", "NOTICE", "relink-evidence.json", "relink-proof.patch",
               "relink-link-inputs.txt", "relink.mjs",
               "distribution-review.md", "release-notices.md", "release-sources.md",
               "bundle.mjs", "pass.mjs", "probe.mjs", "process.mjs", "compact.py",
               "distribution-inputs.json", "distribution.py", "toolchain.py", "review.py", "release.py")
CONDITIONS = [
    "Publish the exact matching source archive beside every runtime, with equivalent access and checksums.",
    "Retain matching source and notices while either release repository offers the runtime, including after sealing.",
    "Preserve original component terms, complete EC/URL originals and the complete frozen upstream archives.",
    "Retain adapter source and compatible-library rebuild/relink instructions without restricting debugging modifications.",
    "Re-review changed source/resource/license inputs; do not infer approval from a manifest boolean.",
]


def select_components(registry, archives):
    s.require(archives and len(archives) == len(set(archives)), "Invalid linked archive inventory")
    components = registry["components"]
    s.require(len({c["id"] for c in components}) == len(components), "Duplicate engine component")
    result = {}
    for archive in archives:
        s.safe_path(archive)
        matches = [c for c in components if any(re.fullmatch(p, archive) for p in c["archivePatterns"])]
        s.require(len(matches) == 1, f"Unmapped or ambiguous linked archive: {archive}")
        item = matches[0]
        s.require(item["noticeFiles"] and item["sourceNoticePaths"] and item["licenseExpression"],
                  f"Incomplete engine component: {item['id']}")
        result.setdefault(item["id"], {"component": item, "linkedArchives": []})["linkedArchives"].append(archive)
    return [result[k] for k in sorted(result)]


def engine_selection(cache, merged):
    runtime = s.read_json(HERE / "lock.json")
    evidence = s.read_json(HERE / "relink-evidence.json")
    entry = merged["engineSource"]
    s.require(entry == evidence["sourceArchive"], "Relink source archive differs")
    execution = evidence["execution"]
    for suffix in ("js", "wasm"):
        original = next(e for e in runtime["engine"] if e["name"] == "wasmtex-luatex." + suffix)
        s.require(execution["originalEngine"][suffix] == original["sha256"], "Relink original compiler differs")
    s.require(evidence["network"] == "none" and evidence["exitCode"] == 0 and evidence["oomKilled"] is False
              and execution["importMapUnchanged"] is True and len(execution["passes"]) == 3
              and all(p["relinkProof"] == 7 for p in execution["passes"]), "Incomplete relink qualification")
    s.require(s.digest((HERE / "relink-proof.patch").read_bytes()) == evidence["patchSha256"], "Relink patch changed")
    trace = (HERE / "relink-link-inputs.txt").read_bytes()
    s.require(s.digest(trace) == evidence["linkTraceSha256"], "Relink trace changed")
    archives = sorted(set(re.findall(r"(\S+\.a)\(", trace.decode())))
    archives = [p.replace("/build/wasm/", "texlive-build/").replace(
        "/emsdk/upstream/emscripten/cache/sysroot/lib/wasm32-emscripten/", "emscripten-sysroot/") for p in archives]
    s.require(archives == evidence["linkedArchives"], "Relink archive selection differs from trace")
    prefix = "wasmtex-" + runtime["engineRelease"] + "-source/"
    name = prefix + "release/ENGINE-COMPONENTS.json"
    data = s.archive_files(cache / entry["name"], {name})
    s.require(name in data, "Missing original component registry")
    registry = json.loads(data[name])
    s.require(registry["linkInventorySourceRevision"] == evidence["engineGeneration"], "Component generation differs")
    family = [f for f in registry["families"] if f["family"] == "luahbtex"]
    s.require(len(family) == 1 and family[0]["combinedLicense"] == "GPL-2.0-only", "Unreviewed engine family")
    selected = select_components(registry, evidence["linkedArchives"])
    snapshot = prefix + "source/wasmtex/" + evidence["engineGeneration"] + "/"
    wanted = {prefix + s.safe_path(p) for c in selected for p in c["component"]["sourceNoticePaths"]}
    wanted.update(snapshot + s.safe_path(p) for c in selected for p in c["component"]["noticeFiles"])
    originals = s.archive_files(cache / entry["name"], wanted)
    s.require(set(originals) == wanted and all(b.strip() for b in originals.values()), "Missing component source notice")
    result = {"family": family[0], "registrySha256": s.digest(data[name]),
              "engineGeneration": evidence["engineGeneration"], "components": selected,
              "sourceNotices": {n: {"bytes": len(b), "sha256": s.digest(b)} for n, b in sorted(originals.items())}}
    return result, originals, data[name]


def inventory(cache, merged, notices, engine_notices):
    selection, originals, registry = engine_selection(cache, merged)
    all_notices = {**{n: v["bytes"] for n, v in notices.items()}, **engine_notices, **originals}
    records = {n: {"bytes": len(b), "sha256": s.digest(b)} for n, b in sorted(all_notices.items())}
    inputs = {"files": {n: s.digest((HERE / n).read_bytes()) for n in BOUND_FILES},
              "engineSource": merged["engineSource"], "packages": merged["packages"],
              "matchedSources": merged["files"], "notices": records, "engineSelection": selection}
    return inputs, originals, registry


def validate(decision, inputs):
    s.require(decision.get("schemaVersion") == 1 and decision.get("decision") == "reviewed-for-paired-distribution",
              "Distribution review is incomplete")
    s.require(decision.get("conditions") == CONDITIONS, "Distribution conditions differ")
    s.require(decision.get("inputs") == inputs, "Distribution review is stale; review changed inputs before publication")
    return decision


def check(cache, merged, notices, engine_notices):
    inputs, originals, registry = inventory(cache, merged, notices, engine_notices)
    decision = validate(s.read_json(HERE / "distribution-review.json"), inputs)
    return decision, originals, registry


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("cache", type=Path)
    parser.add_argument("output", type=Path, help="new inventory JSON; does not approve distribution")
    args = parser.parse_args()
    import distribution as d
    merged, _, _, notices, _, engine_notices, _, _ = d.verified_inputs(args.cache)
    inputs, _, _ = inventory(args.cache, merged, notices, engine_notices)
    with args.output.open("xb") as out:
        out.write(s.encoded(inputs))
    print("Wrote observed review inputs; no release decision was made")
