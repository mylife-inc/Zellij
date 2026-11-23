"""Private distribution failure cases using small synthetic source archives."""
import copy
import gzip
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import distribution as d


class DistributionTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.cache = self.root / "cache"
        self.cache.mkdir()
        self.originals = {"font.mf": b"synthetic font source\n", "copyrite.txt": b"synthetic notice\n"}
        self.originals["00files.txt"] = b""
        self.originals["00files.txt"] = self.listing()
        self.originals["00files.txt"] = self.listing()
        self.assertEqual(len(self.originals["00files.txt"]), len(self.listing()))
        members = [("doc/" + n, b) for n, b in self.originals.items()]
        entry = self.pack("ec.tar.xz", members)
        self.package = {"name": "ec", "relocated": True, "revision": "1", "license": {"ids": ["synthetic"]},
                        "noticePaths": ["texmf-dist/doc/copyrite.txt"], "archives": [entry]}
        self.file = {"resource": "3/font", "path": "texmf-dist/doc/font.mf", "package": "ec",
                     "bytes": len(self.originals["font.mf"]), "sha256": d.s.digest(self.originals["font.mf"])}
        self.inv = {"packages": [self.package], "files": [self.file], "metadata": [],
                    "supportingNotices": [], "engineSource": {"name": "engine.tar.xz", "sha256": "a" * 64}}

    def listing(self):
        return ("Length Date Name\n------ ---- ----\n" + "".join(
            f"{len(b):8d} 01-01-12 {n}\n" for n, b in sorted(self.originals.items())) + "----\n3\n").encode()

    def pack(self, name, members):
        p = self.cache / name
        with tarfile.open(p, "w:xz") as tar:
            for name, data in members:
                # The real manifest lives under doc/fonts/ec.
                if name == "doc/00files.txt":
                    name = "doc/fonts/ec/00files.txt"
                if isinstance(data, tarfile.TarInfo):
                    tar.addfile(data)
                else:
                    info = tarfile.TarInfo(name)
                    info.size = len(data)
                    tar.addfile(info, io.BytesIO(data))
        return {"name": p.name, "kind": "run", "bytes": p.stat().st_size, "sha512": d.s.digest(p.read_bytes(), "sha512")}

    def test_union_keeps_all_notices_and_rejects_conflicting_provenance(self):
        other = copy.deepcopy(self.inv)
        other["packages"][0]["noticePaths"].append("texmf-dist/doc/additional.txt")
        merged, upstream = d.merge_inventories([self.inv, other])
        self.assertEqual(len(merged["files"]), 1)
        self.assertEqual(len(merged["packages"][0]["noticePaths"]), 2)
        self.assertEqual(set(upstream), {"ec.tar.xz", "engine.tar.xz"})
        for alter, message in [
            (lambda i: i["packages"][0].update(revision="2"), "Conflicting package"),
            (lambda i: i["files"][0].update(sha256="b" * 64), "Conflicting source"),
            (lambda i: i["engineSource"].update(sha256="b" * 64), "Conflicting engine"),
        ]:
            changed = copy.deepcopy(self.inv)
            alter(changed)
            with self.assertRaisesRegex(ValueError, message):
                d.merge_inventories([self.inv, changed])

    def test_union_rejects_ambiguous_or_unsafe_cache_names(self):
        for name, message in [("EC.tar.xz", "Case-colliding"), ("../escape", "Unsafe source"),
                              ("folder/source.xz", "must be flat")]:
            changed = copy.deepcopy(self.inv)
            changed["metadata"] = [{"name": name, "sha256": "b" * 64}]
            with self.assertRaisesRegex(ValueError, message):
                d.merge_inventories([changed])

    def test_ec_complete_originals_and_source_receipts(self):
        files, receipts = d.complete_ec(self.cache, self.package)
        self.assertEqual(files, self.originals)
        self.assertEqual(len(receipts), 3)
        for record in receipts:
            self.assertEqual(record["sha256"], d.s.digest(files[record["name"]]))

    def test_ec_missing_duplicate_changed_and_linked_members_fail(self):
        members = [("doc/" + n, b) for n, b in self.originals.items()]
        link = tarfile.TarInfo("doc/font.mf")
        link.type, link.linkname = tarfile.SYMTYPE, "../../outside"
        cases = [([m for m in members if m[0] != "doc/font.mf"], "missing files"),
                 (members + [("elsewhere/font.mf", self.originals["font.mf"])], "Duplicate EC"),
                 ([(n, b + b"changed" if n == "doc/font.mf" else b) for n, b in members], "Invalid EC source"),
                 ([(n, link if n == "doc/font.mf" else b) for n, b in members], "Invalid EC source")]
        for changed, message in cases:
            self.package["archives"] = [self.pack("ec.tar.xz", changed)]
            with self.assertRaisesRegex(ValueError, message):
                d.complete_ec(self.cache, self.package)

    def test_ec_listing_requires_valid_rows_count_and_unique_safe_names(self):
        original = self.originals["00files.txt"]
        for changed in [original.replace(b"\n3\n", b"\n4\n"), original.replace(b"font.mf", b"../font.mf"),
                        original.replace(b"font.mf", b"COPYRITE.TXT"), original + b"unexpected\n",
                        original.replace(b"01-01-12 font.mf", b"invalid font.mf")]:
            with self.assertRaises(ValueError):
                d.ec_listing(changed)

    def fake_inputs(self):
        notice = {"doc/copyrite.txt": {"bytes": self.originals["copyrite.txt"], "archive": "ec.tar.xz"}}
        ec, receipts = d.complete_ec(self.cache, self.package)
        return (self.inv, {"ec.tar.xz": self.package["archives"][0]}, [self.file], notice,
                {"release": "synthetic"}, {"engine/LICENSE": b"synthetic engine notice\n"}, ec, receipts)

    def test_archives_reproduce_and_retain_all_original_bytes(self):
        with patch.object(d, "verified_inputs", return_value=self.fake_inputs()):
            a = d.assemble(self.cache, self.root / "first")
            b = d.assemble(self.cache, self.root / "second")
        self.assertEqual(a, b)
        for family in ("notices", "sources"):
            record = a[family]["archive"]
            archive = self.root / "first" / record["name"]
            self.assertEqual(archive.read_bytes(), (self.root / "second" / record["name"]).read_bytes())
            with archive.open("rb") as source:
                self.assertEqual(source.read(10)[4:8], b"\0" * 4)
        with tarfile.open(self.root / "first/experimental-pdf-sources.tar.gz", "r:gz") as tar:
            self.assertEqual(tar.extractfile("upstream/ec.tar.xz").read(), (self.cache / "ec.tar.xz").read_bytes())
            manifest = json.load(tar.extractfile("SOURCE-MANIFEST.json"))
            for item in manifest["files"]:
                data = tar.extractfile(item["name"]).read()
                self.assertEqual((len(data), d.s.digest(data)), (item["bytes"], item["sha256"]))
        self.assertFalse(a["redistributionReviewComplete"])

    def test_corruption_during_preparation_and_existing_outputs_are_preserved(self):
        fake = self.fake_inputs()
        output = self.root / "output"
        archive = self.cache / "ec.tar.xz"
        archive.write_bytes(archive.read_bytes() + b"corruption")
        with patch.object(d, "verified_inputs", return_value=fake), self.assertRaisesRegex(ValueError, "size mismatch"):
            d.assemble(self.cache, output)
        self.assertFalse(output.exists())
        self.assertFalse(list(self.root.glob(".pdf-distribution-*")))
        output.mkdir()
        (output / "authored").write_text("keep")
        with patch.object(d, "verified_inputs") as verify, self.assertRaisesRegex(ValueError, "already exists"):
            d.assemble(self.cache, output)
        verify.assert_not_called()
        self.assertEqual((output / "authored").read_text(), "keep")

    def test_late_output_creation_is_not_replaced(self):
        output = self.root / "output"
        real_archive = d.archive
        def appeared(*args):
            result = real_archive(*args)
            if not output.exists():
                output.mkdir()
                (output / "authored").write_text("keep")
            return result
        with patch.object(d, "verified_inputs", return_value=self.fake_inputs()), \
                patch.object(d, "archive", side_effect=appeared), self.assertRaisesRegex(ValueError, "appeared"):
            d.assemble(self.cache, output)
        self.assertEqual((output / "authored").read_text(), "keep")
        self.assertFalse(list(self.root.glob(".pdf-distribution-*")))

    def test_archive_verification_rejects_modified_duplicate_and_linked_content(self):
        root = self.root / "stage"
        root.mkdir()
        (root / "file").write_bytes(b"original")
        _, records = d.archive(root, self.root / "good.tar.gz")
        for variant in ("changed", "duplicate", "link"):
            out = self.root / (variant + ".tar.gz")
            with out.open("wb") as raw, gzip.GzipFile(filename="", fileobj=raw, mode="wb", mtime=0) as gz, \
                    tarfile.open(fileobj=gz, mode="w|") as tar:
                for _ in range(2 if variant == "duplicate" else 1):
                    info = tarfile.TarInfo("file")
                    info.mode = 0o644
                    if variant == "link":
                        info.type, info.linkname = tarfile.SYMTYPE, "../../outside"
                        tar.addfile(info)
                    else:
                        data = b"modified" if variant == "changed" else b"original"
                        info.size = len(data)
                        tar.addfile(info, io.BytesIO(data))
            with self.assertRaises(ValueError):
                d.verify_archive(out, records)


if __name__ == "__main__":
    unittest.main()
