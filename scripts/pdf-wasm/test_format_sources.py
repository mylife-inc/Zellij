"""Format selection and archive preparation failures, using synthetic originals."""
import copy
from unittest import TestCase
from unittest.mock import patch

import format_sources as f
import sources as s
import test_sources as fixtures


class FormatSourcesTest(TestCase):
    def setUp(self):
        self.fixture = fixtures.SourcesTest("test_original_bytes_notices_and_relocated_paths_match")
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.root = self.fixture.root
        self.recipe = {"entrypoint": "lualatex.ini", "sourceDateEpoch": 1787702400,
                       "resources": ["26/lualatex.ini"]}
        self.provenance = copy.deepcopy(self.fixture.provenance)
        self.provenance["files"][0]["key"] = "pdftex/26/lualatex.ini"

    def test_selection_requires_original_unique_sources_and_an_epoch(self):
        resources = f.select_resources(self.fixture.runtime, self.provenance, self.recipe)
        self.assertEqual(resources[0]["sha256"], s.digest(self.fixture.tex))
        for mutate in [lambda p: p["resources"].append("26/lualatex.ini"),
                       lambda p: p["resources"].append("26/missing.tex"),
                       lambda p: p.update(sourceDateEpoch=True),
                       lambda p: p.update(sourceDateEpoch=2**31),
                       lambda p: p.update(entrypoint="other.ini")]:
            recipe = copy.deepcopy(self.recipe)
            mutate(recipe)
            with self.assertRaises(ValueError):
                f.select_resources(self.fixture.runtime, self.provenance, recipe)
        del self.provenance["files"][0]["source"]
        with self.assertRaisesRegex(ValueError, "original format source"):
            f.select_resources(self.fixture.runtime, self.provenance, self.recipe)

    def test_preparation_rechecks_archive_and_file_bytes(self):
        inv = self.fixture.inventory()
        self.assertEqual(f.input_bytes(inv, self.root), {"26/sample.sty": self.fixture.tex})
        self.fixture.pack([("tex/sample.sty", b"Changed source"), ("doc/NOTICE", self.fixture.notice)])
        with self.assertRaisesRegex(ValueError, "checksum mismatch|size mismatch"):
            f.input_bytes(inv, self.root)
        self.fixture.refresh_archive(inv)
        with self.assertRaisesRegex(ValueError, "source differs"):
            f.input_bytes(inv, self.root)

    def test_existing_output_and_missing_notice_never_publish_sources(self):
        output = self.root / "output"
        output.mkdir()
        sentinel = output / "keep"
        sentinel.write_text("authored")
        with self.assertRaisesRegex(ValueError, "output already exists"):
            f.prepare(self.root, output)
        self.assertEqual(sentinel.read_text(), "authored")
        output = self.root / "new-output"
        self.fixture.pack([("tex/sample.sty", self.fixture.tex)])
        inv = self.fixture.inventory()
        self.fixture.refresh_archive(inv)
        with patch.object(f, "inventory", return_value=inv), patch.object(s, "read_json", return_value=inv):
            with self.assertRaisesRegex(ValueError, "Notice missing"):
                f.prepare(self.root, output)
        self.assertFalse(output.exists())

    def test_missing_member_and_stale_inventory_fail(self):
        inv = self.fixture.inventory()
        self.fixture.pack([("doc/NOTICE", self.fixture.notice)])
        self.fixture.refresh_archive(inv)
        with self.assertRaisesRegex(ValueError, "incomplete"):
            f.input_bytes(inv, self.root)
        with patch.object(f, "inventory", return_value=inv), patch.object(s, "read_json", return_value={}):
            with self.assertRaisesRegex(ValueError, "inventory is stale"):
                f.prepare(self.root, self.root / "new-output")

    def test_preparation_is_offline_and_collects_exact_originals(self):
        inv = self.fixture.inventory()
        output = self.root / "prepared"
        with patch.object(f, "inventory", return_value=inv), patch.object(s, "read_json", return_value=inv), \
                patch.object(s, "fetch_one", side_effect=AssertionError("unexpected download")):
            f.prepare(self.root, output)
        self.assertEqual((output / s.digest(self.fixture.tex)).read_bytes(), self.fixture.tex)
        self.assertIn(self.fixture.notice, (output / "COLLECTED-NOTICES.txt").read_bytes())
