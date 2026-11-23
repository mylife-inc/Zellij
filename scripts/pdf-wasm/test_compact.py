"""Package integrity, reproducibility and output preservation with tiny inputs."""
from pathlib import Path
import tarfile
import tempfile
from unittest import TestCase
from unittest.mock import patch

import compact as c
import sources as s


class CompactTest(TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.bundle, self.format, self.fonts = [self.root / n for n in ("bundle", "format", "fonts")]
        for p in (self.bundle, self.format, self.fonts):
            p.mkdir()
        entry = lambda name, data: {"name": name, "bytes": len(data), "sha256": s.digest(data)}
        self.runtime = {"engine": [entry("engine.js", b"engine"), entry("old.fmt.gz", b"unused")],
                        "resources": [entry("26/source.tex", b"source"), entry("51/luaotfload-names.lua", b"old") ,
                                      entry("11/pdftex.map", b"old-map")]}
        self.format_recipe = {"output": {"bytes": 6, "sha256": s.digest(b"format")}}
        self.font_recipe = {"output": {"bytes": 5, "sha256": s.digest(b"fonts")}}
        for data in (b"engine", b"source"):
            (self.bundle / s.digest(data)).write_bytes(data)
        (self.format / "zellij-lualatex.fmt").write_bytes(b"format")
        (self.fonts / "luaotfload-names.lua").write_bytes(b"fonts")
        (self.fonts / "pdftex.map").write_bytes(c.FONT_MAP)

    def package(self, output):
        def read(path):
            return {"lock.json": self.runtime, "format-inputs.json": self.format_recipe,
                    "font-inputs.json": self.font_recipe}[path.name]
        with patch.object(s, "read_json", side_effect=read):
            c.package(self.bundle, self.format, self.fonts, output)

    def test_archive_is_reproducible_and_every_member_matches_staged_bytes(self):
        a, b = self.root / "one", self.root / "two"
        self.package(a)
        self.package(b)
        archive = "experimental-pdf-resources.tar.gz"
        self.assertEqual((a / archive).read_bytes(), (b / archive).read_bytes())
        with tarfile.open(a / archive) as tar:
            self.assertEqual({m.name for m in tar}, {p.name for p in (a / "package").iterdir()})
            for member in tar:
                self.assertTrue(member.isfile())
                self.assertEqual(member.mtime, 0)
                self.assertEqual(tar.extractfile(member).read(), (a / "package" / member.name).read_bytes())

    def test_corrupt_generated_or_original_bytes_never_publish(self):
        target = self.fonts / "luaotfload-names.lua"
        target.write_bytes(b"other")
        output = self.root / "new"
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            self.package(output)
        self.assertFalse(output.exists())
        target.write_bytes(b"fonts")
        (self.bundle / s.digest(b"source")).write_bytes(b"other!")
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            self.package(output)
        self.assertFalse(output.exists())

    def test_existing_output_is_preserved(self):
        output = self.root / "existing"
        output.mkdir()
        (output / "keep").write_bytes(b"authored")
        with self.assertRaisesRegex(ValueError, "output already exists"):
            self.package(output)
        self.assertEqual((output / "keep").read_bytes(), b"authored")
