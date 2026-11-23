"""Font source preparation failures, using synthetic original archives."""
import copy
from unittest import TestCase
from unittest.mock import patch

import font_sources as f
import sources as s
import test_sources as fixtures


class FontSourcesTest(TestCase):
    def setUp(self):
        self.fixture = fixtures.SourcesTest("test_original_bytes_notices_and_relocated_paths_match")
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.root = self.fixture.root

    def test_selection_requires_exact_runtime_fonts_original_bytes_and_an_epoch(self):
        runtime = copy.deepcopy(self.fixture.runtime)
        runtime["resources"] = [{"name": "47/font.otf"}]
        provenance = copy.deepcopy(self.fixture.provenance)
        record = provenance["files"][0]
        record["key"] = "pdftex/51/luaotfload-tool.lua"
        font = copy.deepcopy(record)
        font["key"] = "pdftex/47/font.otf"
        provenance["files"].append(font)
        recipe = {"entrypoint": "luaotfload-tool.lua", "sourceDateEpoch": 1787702400,
                  "resources": ["51/luaotfload-tool.lua", "47/font.otf"]}
        self.assertEqual(len(f.select_resources(runtime, provenance, recipe)), 2)
        for mutate in [lambda p: p["resources"].append("47/font.otf"),
                       lambda p: p["resources"].remove("47/font.otf"),
                       lambda p: p["resources"].append("47/extra.otf"),
                       lambda p: p.update(sourceDateEpoch=True),
                       lambda p: p.update(sourceDateEpoch=2**31),
                       lambda p: p.update(entrypoint="other.lua")]:
            changed = copy.deepcopy(recipe)
            mutate(changed)
            with self.assertRaises(ValueError):
                f.select_resources(runtime, provenance, changed)
        del provenance["files"][0]["source"]
        with self.assertRaisesRegex(ValueError, "original font source"):
            f.select_resources(runtime, provenance, recipe)

    def test_preparation_is_offline_and_preserves_sources_notices_and_existing_output(self):
        inv = self.fixture.inventory()
        output = self.root / "prepared"
        with patch.object(f, "inventory", return_value=inv), patch.object(s, "read_json", return_value=inv), \
                patch.object(s, "fetch_one", side_effect=AssertionError("unexpected download")):
            f.prepare(self.root, output)
        self.assertEqual((output / s.digest(self.fixture.tex)).read_bytes(), self.fixture.tex)
        self.assertIn(self.fixture.notice, (output / "COLLECTED-NOTICES.txt").read_bytes())
        receipt = s.read_json(output / "verification.json")
        self.assertEqual(receipt["fontSourceLockSha256"], s.digest(s.encoded(inv)))
        before = (output / "verification.json").read_bytes()
        with self.assertRaisesRegex(ValueError, "output already exists"):
            f.prepare(self.root, output)
        self.assertEqual((output / "verification.json").read_bytes(), before)

    def test_missing_notice_and_changed_archive_fail_before_publication(self):
        inv = self.fixture.inventory()
        output = self.root / "prepared"
        self.fixture.pack([("tex/sample.sty", self.fixture.tex)])
        with patch.object(f, "inventory", return_value=inv), patch.object(s, "read_json", return_value=inv):
            with self.assertRaisesRegex(ValueError, "checksum mismatch|size mismatch"):
                f.prepare(self.root, output)
        self.assertFalse(output.exists())
        self.fixture.refresh_archive(inv)
        with patch.object(f, "inventory", return_value=inv), patch.object(s, "read_json", return_value=inv):
            with self.assertRaisesRegex(ValueError, "Notice missing"):
                f.prepare(self.root, output)
        self.assertFalse(output.exists())
        with patch.object(f, "inventory", return_value=inv), patch.object(s, "read_json", return_value={}):
            with self.assertRaisesRegex(ValueError, "inventory is stale"):
                f.prepare(self.root, output)
