import tempfile
import unittest
from pathlib import Path

from scripts.generate_homebrew import render_formula


class FormulaTests(unittest.TestCase):
    def test_formula_uses_both_real_archives_and_mac_only_requirement(self):
        with tempfile.TemporaryDirectory() as directory:
            dist = Path(directory)
            (dist / "zcco_0.1.0_darwin_arm64.tar.gz").write_bytes(b"hello")
            (dist / "zcco_0.1.0_darwin_amd64.tar.gz").write_bytes(b"world")
            formula = render_formula("0.1.0", dist)
            self.assertIn('class Zcco < Formula', formula)
            self.assertIn('depends_on :macos', formula)
            self.assertIn('on_arm do', formula)
            self.assertIn('on_intel do', formula)
            self.assertIn('https://github.com/zhangbyby/zcco/releases/download/v0.1.0/zcco_0.1.0_darwin_arm64.tar.gz', formula)
            self.assertIn('https://github.com/zhangbyby/zcco/releases/download/v0.1.0/zcco_0.1.0_darwin_amd64.tar.gz', formula)
            self.assertIn('on_arm do\n      url "https://github.com/zhangbyby/zcco/releases/download/v0.1.0/zcco_0.1.0_darwin_arm64.tar.gz"\n      sha256 "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"', formula)
            self.assertIn('on_intel do\n      url "https://github.com/zhangbyby/zcco/releases/download/v0.1.0/zcco_0.1.0_darwin_amd64.tar.gz"\n      sha256 "486ea46224d1bb4fb680f34f7c9ad96a8f24ec88be73ea8e5a6c65260e9cb8a7"', formula)
            self.assertIn('bin.install "zcco"', formula)

    def test_missing_archive_prevents_formula_publication(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(FileNotFoundError):
                render_formula("0.1.0", Path(directory))

    def test_unstable_or_malformed_version_is_not_published_to_stable_tap(self):
        for version in ["0.1.0-rc.1", "v0.1.0", "bad\"version", ""]:
            with self.subTest(version=version), self.assertRaises(ValueError):
                render_formula(version, Path("not-used"))


if __name__ == "__main__":
    unittest.main()
