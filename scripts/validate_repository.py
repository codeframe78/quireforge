#!/usr/bin/env python3
"""Run dependency-free checks for QuireForge's tracked repository sources."""

from __future__ import annotations

import re
import subprocess
import sys
import urllib.parse
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent

REQUIRED_PATHS = (
    ".editorconfig",
    ".gitignore",
    "AGENTS.md",
    "CHANGELOG.md",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "LICENSE",
    "README.md",
    "SECURITY.md",
    "SUPPORT.md",
    ".github/PULL_REQUEST_TEMPLATE.md",
    ".github/dependabot.yml",
    ".github/workflows/repository-checks.yml",
    "docs/ARCHITECTURE.md",
    "docs/ROADMAP.md",
    "docs/THREAT-MODEL.md",
)

IDENTITY_EXPECTATIONS = {
    "README.md": (
        "QuireForge",
        "Build boldly. Work locally.",
        "unofficial community project",
        "io.github.codeframe78.QuireForge",
        "https://quireforge.jamesjennison.net",
    ),
    "docs/DECISIONS/0003-permanent-quireforge-identity.md": (
        "codeframe78/quireforge",
        "~/.config/quireforge",
        "~/.local/share/quireforge",
        "~/.cache/quireforge",
        "~/.local/state/quireforge",
    ),
}

MARKDOWN_LINK = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
HTML_LINK = re.compile(r"(?:href|src)=[\"']([^\"']+)[\"']", re.IGNORECASE)
IGNORED_SCHEMES = ("http://", "https://", "mailto:", "tel:", "data:")
FORBIDDEN_TRACKED_NAMES = (
    re.compile(r"(^|/)\.env(?:\.|$)"),
    re.compile(r"(^|/)id_(?:rsa|dsa|ecdsa|ed25519)(?:\.pub)?$"),
    re.compile(r"\.(?:key|pem|p12|pfx)$", re.IGNORECASE),
)
SECRET_PATTERNS = (
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    re.compile(r"\bgh[pousr]_[A-Za-z0-9_]{20,}\b"),
    re.compile(r"\bsk-(?:proj-)?[A-Za-z0-9_-]{20,}\b"),
)


def repository_files() -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    )
    return [ROOT / item.decode() for item in result.stdout.split(b"\0") if item]


def link_target(raw: str) -> str:
    target = raw.strip()
    if target.startswith("<") and ">" in target:
        return target[1 : target.index(">")]
    return target.split(maxsplit=1)[0]


def validate() -> list[str]:
    errors: list[str] = []
    files = repository_files()

    for relative in REQUIRED_PATHS:
        if not (ROOT / relative).is_file():
            errors.append(f"missing required file: {relative}")

    for path in files:
        relative = path.relative_to(ROOT).as_posix()
        if any(pattern.search(relative) for pattern in FORBIDDEN_TRACKED_NAMES):
            errors.append(f"credential-like file is tracked: {relative}")

        if path.is_symlink():
            try:
                path.resolve(strict=True).relative_to(ROOT)
            except (FileNotFoundError, ValueError):
                errors.append(f"tracked symlink escapes or is broken: {relative}")
            continue

        data = path.read_bytes()
        if b"\x00" in data:
            continue
        try:
            text = data.decode("utf-8")
        except UnicodeDecodeError:
            errors.append(f"tracked text is not UTF-8: {relative}")
            continue
        if text and not text.endswith("\n"):
            errors.append(f"missing final newline: {relative}")
        for pattern in SECRET_PATTERNS:
            if pattern.search(text):
                errors.append(f"high-confidence secret pattern: {relative}")
        for number, line in enumerate(text.splitlines(), start=1):
            if line.endswith((" ", "\t")) and path.suffix != ".md":
                errors.append(f"trailing whitespace: {relative}:{number}")

        if path.suffix.lower() == ".svg":
            try:
                ET.fromstring(text)
            except ET.ParseError as exc:
                errors.append(f"invalid SVG XML: {relative}: {exc}")

        if path.suffix.lower() != ".md":
            continue
        for raw in MARKDOWN_LINK.findall(text) + HTML_LINK.findall(text):
            target = urllib.parse.unquote(link_target(raw))
            if not target or target.startswith(("#", *IGNORED_SCHEMES)):
                continue
            target_path = target.split("#", 1)[0].split("?", 1)[0]
            if not target_path or target_path.startswith("/"):
                continue
            resolved = (path.parent / target_path).resolve()
            try:
                resolved.relative_to(ROOT)
            except ValueError:
                errors.append(f"link escapes repository: {relative}: {target}")
                continue
            if not resolved.exists():
                errors.append(f"broken local link: {relative}: {target}")

    for relative, expected_values in IDENTITY_EXPECTATIONS.items():
        path = ROOT / relative
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8")
        for value in expected_values:
            if value not in text:
                errors.append(f"missing identity value in {relative}: {value}")

    return errors


def main() -> int:
    errors = validate()
    if errors:
        print("Repository validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("Repository validation passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
