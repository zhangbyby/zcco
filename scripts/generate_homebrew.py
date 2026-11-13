#!/usr/bin/env python3
"""Generate the stable macOS Formula from GoReleaser's actual archives."""

import argparse
import hashlib
import re
from pathlib import Path


def render_formula(version: str, dist: Path) -> str:
    if not re.fullmatch(r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)", version):
        raise ValueError("Homebrew stable releases require a numeric major.minor.patch version")
    blocks = []
    for arch, condition in [("arm64", "on_arm"), ("amd64", "on_intel")]:
        archive = dist / f"zcco_{version}_darwin_{arch}.tar.gz"
        with archive.open("rb") as source:
            checksum = hashlib.file_digest(source, "sha256").hexdigest()
        url = f"https://github.com/zhangbyby/zcco/releases/download/v{version}/{archive.name}"
        blocks.append(f'''    {condition} do
      url "{url}"
      sha256 "{checksum}"
    end''')
    architectures = "\n".join(blocks)
    return f'''class Zcco < Formula
  desc "Copy command output to the macOS clipboard"
  homepage "https://github.com/zhangbyby/zcco"
  version "{version}"

  depends_on :macos

  on_macos do
{architectures}
  end

  def install
    bin.install "zcco"
  end

  test do
    assert_match version.to_s, shell_output("#{{bin}}/zcco --version")
  end
end
'''


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version", required=True)
    parser.add_argument("--dist", type=Path, default=Path("dist"))
    parser.add_argument("--output", type=Path, default=Path("dist/zcco.rb"))
    args = parser.parse_args()
    formula = render_formula(args.version, args.dist)
    args.output.write_text(formula, encoding="utf-8")


if __name__ == "__main__":
    main()
