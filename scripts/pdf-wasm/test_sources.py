"""Synthetic archives: test correspondence checks without network/TeX binaries."""
import copy
import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("sources", Path(__file__).with_name("sources.py"))
s = importlib.util.module_from_spec(spec)
spec.loader.exec_module(s)


class SourcesTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.tex = b"% Original upstream source\n"
        self.notice = b"Synthetic fixture notice\n"
        self.archive = self.root / "sample.tar.xz"
        self.pack([("tex/sample.sty", self.tex), ("doc/NOTICE", self.notice)])
        self.policy = {"repository": "https://example.invalid/frozen/", "provenance": {"sha256": "a" * 64},
                       "tlpdb": {"sha256": "b" * 64}, "generatedManifest": {"sha256": "c" * 64},
                       "additionalNotices": {}, "generatedResources": [], "openItems": ["Fixture remains unapproved."]}
        self.runtime = {"mirror": "https://example.invalid/mirror/", "resources": [{"name": "26/sample.sty",
                        "url": "https://example.invalid/mirror/pdftex/26/sample.sty", "bytes": len(self.tex), "sha256": s.digest(self.tex)}],
                        "correspondingSource": {"url": "https://example.invalid/source.tar.xz", "sha256": "d" * 64}}
        self.provenance = {"mirrorRevision": "frozen", "source": {"repository": {"url": self.policy["repository"]},
                           "tlpdb": {"sha256": "b" * 64}}, "files": [{"key": "pdftex/26/sample.sty", "bytes": len(self.tex),
                           "sha256": s.digest(self.tex), "source": {"path": "texmf-dist/tex/sample.sty", "package": "sample",
                           "packageRevision": "1", "license": {"ids": ["MIT"]}, "noticePaths": ["texmf-dist/doc/NOTICE"]}}]}
        self.artifacts = {"provenanceSha256": "a" * 64, "mirrorRevision": "frozen", "artifacts": []}
        self.tlpdb = {"sample": {"revision": "1", "relocated": "1", "containersize": str(self.archive.stat().st_size),
                               "containerchecksum": s.digest(self.archive.read_bytes(), "sha512")}}

    def pack(self, members):
        with tarfile.open(self.archive, "w:xz") as archive:
            for name, content in members:
                info = tarfile.TarInfo(name)
                if isinstance(content, tarfile.TarInfo):
                    archive.addfile(content)
                else:
                    info.size = len(content)
                    archive.addfile(info, io.BytesIO(content))

    def inventory(self):
        return s.build_inventory(self.runtime, self.provenance, self.tlpdb, self.policy, self.artifacts)

    def refresh_archive(self, inventory):
        entry = inventory["packages"][0]["archives"][0]
        entry.update(bytes=self.archive.stat().st_size, sha512=s.digest(self.archive.read_bytes(), "sha512"))

    def test_original_bytes_notices_and_relocated_paths_match(self):
        inventory = self.inventory()
        receipts, notices = s.verify_packages(inventory, self.root)
        self.assertEqual(receipts[0]["sha256"], s.digest(self.tex))
        self.assertEqual(notices["texmf-dist/doc/NOTICE"]["bytes"], self.notice)
        self.assertFalse(inventory["redistributionReviewComplete"])
        self.assertEqual(inventory["openItems"], self.policy["openItems"])

    def test_revision_and_missing_archive_checksum_rejected(self):
        self.tlpdb["sample"]["revision"] = "2"
        with self.assertRaisesRegex(ValueError, "revision mismatch"):
            self.inventory()
        self.tlpdb["sample"]["revision"] = "1"
        del self.tlpdb["sample"]["containerchecksum"]
        with self.assertRaisesRegex(ValueError, "archive checksum"):
            self.inventory()

    def test_unmapped_changed_and_duplicate_resources_rejected(self):
        original = copy.deepcopy(self.provenance)
        self.provenance["files"][0]["key"] = "another"
        with self.assertRaisesRegex(ValueError, "provenance missing"):
            self.inventory()
        self.provenance = copy.deepcopy(original)
        self.provenance["files"][0]["sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "provenance mismatch"):
            self.inventory()
        self.provenance = original
        self.provenance["files"].append(copy.deepcopy(original["files"][0]))
        with self.assertRaisesRegex(ValueError, "Duplicate provenance"):
            self.inventory()

    def test_generated_files_need_matching_pinned_manifest_and_explicit_policy(self):
        record = self.provenance["files"].pop()
        del record["source"]
        self.artifacts["artifacts"].append(record)
        with self.assertRaisesRegex(ValueError, "Unreviewed generated"):
            self.inventory()
        self.policy["generatedResources"] = ["26/sample.sty"]
        self.assertEqual(len(self.inventory()["generatedResources"]), 1)
        self.artifacts["provenanceSha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "Generated resource provenance"):
            self.inventory()

    def test_missing_notice_locations_and_unsafe_paths_rejected(self):
        source = self.provenance["files"][0]["source"]
        source["noticePaths"] = []
        with self.assertRaisesRegex(ValueError, "Missing notice locations"):
            self.inventory()
        source["noticePaths"] = ["../NOTICE"]
        with self.assertRaisesRegex(ValueError, "Unsafe source path"):
            self.inventory()

    def test_corrupt_archive_rejected_before_reading_members(self):
        inventory = self.inventory()
        data = bytearray(self.archive.read_bytes())
        data[40] ^= 1
        self.archive.write_bytes(data)
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            s.verify_packages(inventory, self.root)

    def test_valid_archive_with_different_runtime_bytes_is_rejected(self):
        inventory = self.inventory()
        self.pack([("tex/sample.sty", b"changed source"), ("doc/NOTICE", self.notice)])
        self.refresh_archive(inventory)
        with self.assertRaisesRegex(ValueError, "Runtime source differs"):
            s.verify_packages(inventory, self.root)

    def test_missing_and_empty_notices_rejected(self):
        inventory = self.inventory()
        for entries, error in [([("tex/sample.sty", self.tex)], "Notice missing"),
                               ([("tex/sample.sty", self.tex), ("doc/NOTICE", b"\n")], "Empty notice")]:
            self.pack(entries)
            self.refresh_archive(inventory)
            with self.assertRaisesRegex(ValueError, error):
                s.verify_packages(inventory, self.root)

    def test_source_links_and_duplicate_members_rejected_without_extraction(self):
        inventory = self.inventory()
        for kind in (tarfile.SYMTYPE, tarfile.LNKTYPE):
            info = tarfile.TarInfo("tex/sample.sty")
            info.type, info.linkname = kind, "../../outside"
            self.pack([("tex/sample.sty", info), ("doc/NOTICE", self.notice)])
            self.refresh_archive(inventory)
            with self.assertRaisesRegex(ValueError, "Unsafe source member"):
                s.verify_packages(inventory, self.root)
        self.pack([("tex/sample.sty", self.tex), ("tex/sample.sty", self.tex)])
        self.refresh_archive(inventory)
        with self.assertRaisesRegex(ValueError, "Duplicate source member"):
            s.verify_packages(inventory, self.root)
        self.assertFalse((self.root.parent / "outside").exists())

    def test_cache_symlink_rejected(self):
        data = self.archive.read_bytes()
        self.archive.rename(self.root / "outside.tar.xz")
        self.archive.symlink_to(self.root / "outside.tar.xz")
        with self.assertRaisesRegex(ValueError, "regular source file"):
            s.verified(self.archive, {"bytes": len(data), "sha256": s.digest(data)})

    def test_database_rejects_duplicate_package_and_checksum_fields(self):
        for text in ("name sample\nrevision 1\n\nname sample\nrevision 1\n",
                     "name sample\ncontainerchecksum a\ncontainerchecksum b\n"):
            with self.assertRaisesRegex(ValueError, "Duplicate TeX"):
                s.parse_tlpdb(text)

    def test_engine_format_and_build_receipt_must_match_runtime_pins(self):
        release = "test-release"
        prefix = "wasmtex-" + release + "-source/"
        runtime = {"engineRelease": release, "engine": [
            {"name": n, "bytes": 1, "sha256": s.digest(b"x")}
            for n in ("wasmtex-luatex.js", "wasmtex-luatex.wasm", "wasmtex-luatex.fmt.gz")]}
        manifest = {"releaseId": release, "files": copy.deepcopy(runtime["engine"])}
        receipt = s.encoded({"generationReceipts": {"engine": {"sourceRevision": "one"}, "formats": {"sourceRevision": "two"}}})
        runtime["engine"].append({"name": "BUILD-RECEIPT.luahbtex.json", "bytes": len(receipt), "sha256": s.digest(receipt)})
        source = {"releaseId": release, "engineAssetManifest": {"sha256": s.digest(s.encoded(manifest))},
                  "sources": {"wasmtex": [{"commit": n, "path": "source/" + n} for n in ("one", "two")]}}
        members = [("release/manifest.json", s.encoded(manifest)), ("SOURCE-MANIFEST.json", s.encoded(source)),
                   ("release/BUILD-RECEIPT.luahbtex.json", receipt), ("REBUILD.md", b"Build instructions"),
                   ("RELINK.md", b"Relink instructions"), ("source/one/LICENSE", b"Fixture notice"),
                   ("source/one/THIRD_PARTY_NOTICES.md", b"Fixture notice")]
        members += [("source/one/LICENSES/" + str(i), b"Fixture notice") for i in range(11)]
        self.pack([(prefix + name, data) for name, data in members])
        inventory = {"engineSource": {"name": self.archive.name, "sha256": s.digest(self.archive.read_bytes())}}
        report, notices = s.verify_engine(inventory, runtime, self.root)
        self.assertEqual(report["formatSourceRevision"], "two")
        self.assertFalse(report["rebuildVerified"])
        runtime["engine"][2]["sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "Engine source does not match"):
            s.verify_engine(inventory, runtime, self.root)
        runtime["engine"][2]["sha256"] = s.digest(b"x")
        runtime["engine"][-1]["sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "Engine build receipt mismatch"):
            s.verify_engine(inventory, runtime, self.root)

    def test_incomplete_download_retries_without_publishing_partial_cache(self):
        data = b"complete pinned source"
        entry = {"name": "download.tar.xz", "url": "https://example.invalid/source", "bytes": len(data), "sha256": s.digest(data)}
        with patch.object(s.urllib.request, "urlopen", side_effect=[io.BytesIO(b"partial"), io.BytesIO(data)]) as request, patch.object(s.time, "sleep"):
            s.fetch_one(self.root, entry)
            self.assertEqual(request.call_args.args[0].get_header("User-agent"), "Zellij-PDF-source-verifier/1")
        self.assertEqual((self.root / entry["name"]).read_bytes(), data)
        self.assertEqual(list(self.root.glob(".source-download-*")), [])
        (self.root / entry["name"]).write_bytes(b"corrupt")
        with patch.object(s.urllib.request, "urlopen") as download:
            with self.assertRaisesRegex(ValueError, "size mismatch"):
                s.fetch_one(self.root, entry)
            download.assert_not_called()
        self.assertEqual((self.root / entry["name"]).read_bytes(), b"corrupt")


if __name__ == "__main__":
    unittest.main()
