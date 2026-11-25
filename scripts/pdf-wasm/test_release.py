"""Review and paired-publication safeguards; all compiler/source fixtures synthetic."""
import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import release as r
import review


class ReviewTest(unittest.TestCase):
    def test_component_selection_rejects_unknown_ambiguous_and_missing_terms(self):
        component = {"id": "one", "archivePatterns": [r"libone\.a"], "noticeFiles": ["LICENSE"],
                     "sourceNoticePaths": ["source/LICENSE"], "licenseExpression": "MIT"}
        registry = {"components": [component]}
        self.assertEqual(review.select_components(registry, ["libone.a"])[0]["component"], component)
        for archives in ([], ["libone.a", "libone.a"], ["libunknown.a"], ["../libone.a"]):
            with self.assertRaises(ValueError):
                review.select_components(registry, archives)
        duplicate = {**component, "id": "two"}
        with self.assertRaisesRegex(ValueError, "ambiguous"):
            review.select_components({"components": [component, duplicate]}, ["libone.a"])
        for field in ("noticeFiles", "sourceNoticePaths", "licenseExpression"):
            with self.assertRaisesRegex(ValueError, "Incomplete"):
                review.select_components({"components": [{**component, field: []}]}, ["libone.a"])

    def test_review_requires_exact_decision_conditions_and_observed_inputs(self):
        inputs = {"files": {"compiler": "abc"}, "notices": {"LICENSE": "def"}}
        decision = {"schemaVersion": 1, "decision": "reviewed-for-paired-distribution",
                    "conditions": review.CONDITIONS, "inputs": inputs}
        self.assertEqual(review.validate(decision, copy.deepcopy(inputs)), decision)
        for field, value in (("schemaVersion", 2), ("decision", "pending"), ("conditions", []),
                             ("inputs", {**inputs, "files": {"compiler": "changed"}})):
            with self.assertRaises(ValueError):
                review.validate({**decision, field: value}, inputs)
        with self.assertRaisesRegex(ValueError, "incomplete"):
            review.validate({"redistributionReviewComplete": True}, inputs)


class ReleaseTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.output = self.root / "out"
        self.version = "1.2.3-rc.1"

    def source(self, cache, output, version):
        output.mkdir()
        file = output / f"zel-pdf-sources-v{version}.tar.gz"
        file.write_bytes(b"synthetic source archive")
        self.source_record = r.d.file_record(file, file.name)
        return {"sources": {"archive": self.source_record}, "reviewSha256": "a" * 64,
                "sourceManifestSha256": "b" * 64}

    def runtime(self, resources, distribution, output, version, released):
        self.assertTrue(released)
        output.mkdir()
        file = output / f"zel-pdf-v{version}.tar.gz"
        file.write_bytes(b"synthetic runtime archive")
        return {"sourceArchive": self.source_record, "archive": r.d.file_record(file, file.name)}

    def run_package(self, runtime=None):
        with patch.object(r.d, "assemble", side_effect=self.source), \
                patch.object(r.toolchain, "package", side_effect=runtime or self.runtime):
            return r.package(None, None, self.output, self.version)

    def test_package_requires_exact_pair_and_emits_both_checksums(self):
        report = self.run_package()
        self.assertFalse(report["published"])
        self.assertEqual(len(list((self.output / "artifacts").iterdir())), 4)
        for key in ("runtime", "source"):
            record = report[key]
            file = self.output / "artifacts" / record["name"]
            self.assertEqual(r.d.file_record(file, file.name), record)
            self.assertEqual(file.with_name(file.name + ".sha256").read_text(),
                             f"{record['sha256']}  {file.name}\n")

    def test_mismatched_missing_and_changed_assets_leave_no_release(self):
        def broken(kind):
            def build(*args):
                result = self.runtime(*args)
                file = args[2] / result["archive"]["name"]
                if kind == "pair":
                    result["sourceArchive"] = {**result["sourceArchive"], "sha256": "c" * 64}
                elif kind == "missing":
                    file.unlink()
                else:
                    file.write_bytes(b"changed")
                return result
            return build
        for kind in ("pair", "missing", "changed"):
            with self.assertRaises((ValueError, FileNotFoundError)):
                self.run_package(broken(kind))
            self.assertFalse(self.output.exists())
            self.assertFalse(list(self.root.glob(".pdf-release-*")))

    def test_existing_and_late_destination_preserved(self):
        def late(*args):
            result = self.runtime(*args)
            self.output.mkdir()
            (self.output / "keep").write_text("authored")
            return result
        with self.assertRaisesRegex(ValueError, "appeared"):
            self.run_package(late)
        with patch.object(r.d, "assemble") as assemble:
            with self.assertRaisesRegex(ValueError, "already exists"):
                r.package(None, None, self.output, self.version)
            assemble.assert_not_called()
        self.assertEqual((self.output / "keep").read_text(), "authored")
        self.assertFalse(list(self.root.glob(".pdf-release-*")))

    def test_invalid_version_fails_before_build(self):
        with patch.object(r.d, "assemble") as assemble:
            for version in ("../bad", "v1.2.3", "1.2.3/path"):
                with self.assertRaisesRegex(ValueError, "Invalid release version"):
                    r.package(None, None, self.output, version)
            assemble.assert_not_called()
