"""Versioned optional package tests; synthetic fixtures contain no upstream code."""
import copy
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import toolchain as t


class ToolchainTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.report = {"sources": {"archive": {"name": "experimental-pdf-sources.tar.gz", "bytes": 100,
                                               "sha256": "a" * 64}}, "sourceManifestSha256": "b" * 64}
        self.files = {"runner/process.mjs": b"synthetic runner", "resources/manifest.json": b"{}",
                      "notices/README.md": b"original notice"}

    def test_versioned_deterministic_package_binds_separate_source(self):
        with patch.object(t, "collect", return_value=(self.report, self.files)):
            first = t.package(None, None, self.root / "first", "0.2.0-rc.19")
            second = t.package(None, None, self.root / "second", "0.2.0-rc.19")
        self.assertEqual(first, second)
        archive = first["archive"]
        self.assertLess(archive["bytes"], 10000)
        self.assertEqual((self.root / "first" / (archive["name"] + ".sha256")).read_text(),
                         f"{archive['sha256']}  {archive['name']}\n")
        with tarfile.open(self.root / "first" / archive["name"]) as tar:
            manifest = json.load(tar.extractfile("manifest.json"))
            self.assertEqual(manifest["source_archive"], self.report["sources"]["archive"])
            self.assertFalse(manifest["redistribution_review_complete"])
            self.assertEqual(set(tar.getnames()), set(self.files) | {"manifest.json"})
            for name, record in manifest["files"].items():
                data = tar.extractfile(name).read()
                self.assertEqual(record, {"bytes": len(data), "sha256": t.s.digest(data)})

    def test_version_and_existing_destination_fail_before_collecting(self):
        output = self.root / "out"
        output.mkdir()
        (output / "keep").write_text("authored")
        with patch.object(t, "collect") as collect:
            for version in ("../bad", "v1.2.3", "1.2.3/elsewhere"):
                with self.assertRaisesRegex(ValueError, "Invalid toolchain version"):
                    t.package(None, None, self.root / "new", version)
            with self.assertRaisesRegex(ValueError, "already exists"):
                t.package(None, None, output, "1.2.3")
            collect.assert_not_called()
        self.assertEqual((output / "keep").read_text(), "authored")

    def test_failed_archive_and_late_destination_preserve_outputs(self):
        output = self.root / "out"
        original = t.d.archive

        def late(*args):
            value = original(*args)
            output.mkdir()
            (output / "keep").write_text("authored")
            return value

        with patch.object(t, "collect", return_value=(self.report, self.files)), patch.object(t.d, "archive", side_effect=late):
            with self.assertRaisesRegex(ValueError, "output appeared"):
                t.package(None, None, output, "1.2.3")
        self.assertEqual((output / "keep").read_text(), "authored")
        self.assertFalse(list(self.root.glob(".pdf-toolchain-*")))

    def source_fixture(self, released=False):
        source = self.root / "source"
        source.mkdir()
        selected = t.s.read_json(t.HERE / "distribution-inputs.json")["buildFiles"]
        for name in selected:
            t.d.write_bytes(source, "scripts/pdf-wasm/" + name, (t.HERE / name).read_bytes())
        t.d.write_bytes(source, "notices/original.txt", b"unchanged notice")
        files = [t.d.file_record(p, p.relative_to(source).as_posix()) for p in sorted(source.rglob("*")) if p.is_file()]
        metadata = {}
        if released:
            decision = (t.HERE / "distribution-review.json").read_bytes()
            t.d.write_bytes(source, "notices/DISTRIBUTION-REVIEW.json", decision)
            files.append(t.d.file_record(source / "notices/DISTRIBUTION-REVIEW.json", "notices/DISTRIBUTION-REVIEW.json"))
            metadata = {"releaseVersion": "1.2.3", "reviewSha256": t.s.digest(decision)}
        manifest = t.s.encoded({"files": files, **metadata})
        t.d.write_bytes(source, "SOURCE-MANIFEST.json", manifest)
        name = "zel-pdf-sources-v1.2.3.tar.gz" if released else "experimental-pdf-sources.tar.gz"
        record, _ = t.d.archive(source, self.root / name)
        report = {"redistributionReviewComplete": released, "sources": {"archive": record},
                  "sourceManifestSha256": t.s.digest(manifest), **metadata}
        t.d.write_bytes(self.root, "verification.json", t.s.encoded(report))
        return report

    def test_release_profile_verifies_real_review_and_rejects_identity_changes(self):
        report = self.source_fixture(True)
        actual, _ = t.qualified_sources(self.root, True)
        self.assertEqual(actual, report)
        for field, value in (("redistributionReviewComplete", False), ("releaseVersion", "../escape"),
                             ("reviewSha256", "c" * 64)):
            (self.root / "verification.json").write_bytes(t.s.encoded({**report, field: value}))
            with self.assertRaises(ValueError):
                t.qualified_sources(self.root, True)
        (self.root / "verification.json").write_bytes(t.s.encoded(report))
        import review
        with patch.object(review, "BOUND_FILES", review.BOUND_FILES + ("README.md",)):
            with self.assertRaisesRegex(ValueError, "review is stale"):
                t.qualified_sources(self.root, True)

    def test_release_package_requires_matching_version_and_source_profile(self):
        report = {**self.report, "releaseVersion": "1.2.3"}
        with patch.object(t, "collect", return_value=(report, self.files)):
            with self.assertRaisesRegex(ValueError, "versions differ"):
                t.package(None, None, self.root / "out", "1.2.4", True)
        self.source_fixture()
        with self.assertRaisesRegex(ValueError, "review state"):
            t.qualified_sources(self.root, True)

    def test_original_source_archive_is_verified_and_stale_recipes_rejected(self):
        report = self.source_fixture()
        actual, selected = t.qualified_sources(self.root)
        self.assertEqual(actual, report)
        self.assertEqual(selected["notices/original.txt"], b"unchanged notice")
        stale = self.root / "stale-recipes"
        stale.mkdir()
        (stale / "distribution-inputs.json").write_text('{"buildFiles":["process.mjs"]}')
        (stale / "process.mjs").write_text("changed")
        with patch.object(t, "HERE", stale), self.assertRaisesRegex(ValueError, "Source archive is stale"):
            t.qualified_sources(self.root)
        archive = self.root / report["sources"]["archive"]["name"]
        archive.write_bytes(archive.read_bytes() + b"corrupt")
        with self.assertRaisesRegex(ValueError, "Source archive changed"):
            t.qualified_sources(self.root)

    def test_resource_inventory_and_bytes_are_verified(self):
        resources = self.root / "resources"
        resources.mkdir()
        data = b"synthetic resource"
        entry = {"name": "test", "bytes": len(data), "sha256": t.s.digest(data)}
        manifest = {"engine": [entry], "resources": [], "format": entry}
        (resources / "manifest.json").write_text(json.dumps(manifest))
        blob = resources / entry["sha256"]
        blob.write_bytes(data)
        originals = {"scripts/pdf-wasm/" + n: b"synthetic runner" for n in t.RUNNER_FILES}
        originals["notices/README.md"] = b"notice"
        with patch.object(t, "qualified_sources", return_value=(self.report, originals)), \
                patch.object(t.compact, "expected_manifest", return_value=manifest):
            _, files = t.collect(resources, None)
            self.assertIn("notices/README.md", files)
            blob.write_bytes(b"x" * len(data))
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                t.collect(resources, None)
            blob.write_bytes(data)
            (resources / "extra").mkdir()
            with self.assertRaisesRegex(ValueError, "Unlisted resource"):
                t.collect(resources, None)
            changed = copy.deepcopy(manifest)
            changed["extra"] = "unknown"
            (resources / "manifest.json").write_text(json.dumps(changed))
            with self.assertRaisesRegex(ValueError, "differs from reviewed"):
                t.collect(resources, None)
