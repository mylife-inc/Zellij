"""Original-release provenance and complete distribution failure cases."""
import copy
import io
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import sources as s
import originals
import distribution as d


class OriginalsTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.bytes = b"Unchanged synthetic original; not upstream code.\n"
        self.path = "texmf-dist/tex/latex/synthetic/example.sty"
        archive = self.root / "original.tar.xz"
        with tarfile.open(archive, "w:xz") as tar:
            for name in [self.path.removeprefix("texmf-dist/"), "doc/latex/synthetic/README"]:
                item = tarfile.TarInfo(name)
                item.size = len(self.bytes)
                tar.addfile(item, io.BytesIO(self.bytes))
        entry = {"name": archive.name, "kind": "run", "url": "https://example.test/original.tar.xz",
                 "bytes": archive.stat().st_size, "sha256": s.digest(archive.read_bytes())}
        self.file = {"resource": "26/example.sty", "path": self.path, "package": "synthetic",
                     "bytes": len(self.bytes), "sha256": s.digest(self.bytes)}
        self.package = {"name": "synthetic", "revision": "1", "relocated": True,
                        "license": {"ids": ["synthetic"]},
                        "noticePaths": [self.path], "archives": [entry],
                        "completePaths": [self.path, "texmf-dist/doc/latex/synthetic/README"]}
        self.policy = {"originalPackages": [{"id": "synthetic-original", "package": self.package, "files": [self.file]}]}
        self.runtime = {"resources": [{"name": self.file["resource"], "original": "synthetic-original",
                                      "bytes": len(self.bytes), "sha256": self.file["sha256"]}]}

    def test_exact_original_selection_and_correspondence(self):
        inv = s.original_selection(self.runtime, self.policy)
        self.assertEqual(inv, {"packages": [self.package], "files": [self.file]})
        records, notices = s.verify_packages(inv, self.root)
        self.assertEqual(records[0]["sha256"], self.file["sha256"])
        self.assertEqual(notices[self.path]["bytes"], self.bytes)
        complete, receipts = d.complete_originals(self.root, inv["packages"])
        self.assertEqual(len(complete), 2)
        self.assertTrue(all(data == self.bytes for data in complete.values()))
        self.assertEqual(len(receipts), 2)

    def test_missing_changed_and_conflicting_provenance_are_rejected(self):
        for change in (lambda r: r["resources"][0].update(original="unknown"),
                       lambda r: r["resources"][0].update(url="https://example.test/not-original"),
                       lambda r: r["resources"][0].update(sha256="a" * 64),
                       lambda r: r["resources"].append(r["resources"][0])):
            runtime = copy.deepcopy(self.runtime)
            change(runtime)
            with self.assertRaises(ValueError):
                s.original_selection(runtime, self.policy)
        policy = copy.deepcopy(self.policy)
        policy["originalPackages"].append(policy["originalPackages"][0])
        with self.assertRaisesRegex(ValueError, "Duplicate original package"):
            s.original_selection(self.runtime, policy)
        self.assertEqual(s.original_selection({"resources": []}, self.policy), {"packages": [], "files": []})

    def test_complete_distribution_rejects_missing_and_duplicate_components(self):
        for alter, message in [(lambda p: p["completePaths"].append("texmf-dist/missing"), "Incomplete original"),
                               (lambda p: p["completePaths"].append(p["completePaths"][0]), "Duplicate complete-original"),
                               (lambda p: p["archives"].append(p["archives"][0]), "Duplicate complete original")]:
            package = copy.deepcopy(self.package)
            alter(package)
            with self.assertRaisesRegex(ValueError, message):
                d.complete_originals(self.root, [package])

    def test_preparation_preserves_original_bytes_and_existing_outputs(self):
        real_json = s.read_json
        def read_json(path):
            if path.name == "lock.json": return self.runtime
            if path.name == "source-policy.json": return self.policy
            return real_json(path)
        output = self.root / "output"
        with patch.object(s, "read_json", side_effect=read_json):
            originals.prepare(self.root, output)
            self.assertEqual((output / self.file["sha256"]).read_bytes(), self.bytes)
            with self.assertRaisesRegex(ValueError, "already exists"):
                originals.prepare(self.root, output)
        self.assertFalse(list(self.root.glob('.pdf-originals-*')))

    def test_changed_prepared_archive_never_creates_an_output(self):
        archive = self.root / "original.tar.xz"
        archive.write_bytes(archive.read_bytes() + b"changed")
        inv = s.original_selection(self.runtime, self.policy)
        with self.assertRaisesRegex(ValueError, "size mismatch"):
            s.verify_packages(inv, self.root)
