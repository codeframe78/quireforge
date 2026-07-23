#!/usr/bin/env python3
"""Populate Tauri's Linux tool cache from a checksum-pinned manifest."""

from __future__ import annotations

import hashlib
import json
import os
import tempfile
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = ROOT / "packaging/linux/tauri-tools.json"
MAX_TOOL_BYTES = 64 * 1024 * 1024


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def cache_root() -> Path:
    override = os.environ.get("QUIRE_FORGE_TAURI_CACHE_DIR")
    if override:
        return Path(override).expanduser().resolve()
    return Path.home() / ".cache/tauri"


def download(url: str, destination: Path) -> None:
    request = urllib.request.Request(
        url,
        headers={"User-Agent": "QuireForge packaging tool fetcher"},
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        declared_length = response.headers.get("Content-Length")
        if declared_length and int(declared_length) > MAX_TOOL_BYTES:
            raise RuntimeError(f"refusing oversized packaging tool: {url}")
        written = 0
        with destination.open("wb") as output:
            while chunk := response.read(1024 * 1024):
                written += len(chunk)
                if written > MAX_TOOL_BYTES:
                    raise RuntimeError(f"packaging tool exceeded size limit: {url}")
                output.write(chunk)


def main() -> int:
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    if manifest.get("schemaVersion") != 1:
        raise RuntimeError("unsupported Tauri tool manifest schema")

    destination_root = cache_root()
    destination_root.mkdir(parents=True, exist_ok=True, mode=0o700)

    for entry in manifest.get("tools", []):
        filename = entry["filename"]
        expected = entry["sha256"]
        url = entry["url"]
        destination = destination_root / filename

        if destination.is_file() and sha256(destination) == expected:
            print(f"verified cached Tauri tool: {filename}")
        else:
            with tempfile.NamedTemporaryFile(
                dir=destination_root,
                prefix=f".{filename}.",
                delete=False,
            ) as temporary:
                temporary_path = Path(temporary.name)
            try:
                download(url, temporary_path)
                actual = sha256(temporary_path)
                if actual != expected:
                    raise RuntimeError(
                        f"checksum mismatch for {filename}: expected {expected}, got {actual}"
                    )
                os.replace(temporary_path, destination)
                print(f"downloaded and verified Tauri tool: {filename}")
            finally:
                temporary_path.unlink(missing_ok=True)

        destination.chmod(0o755 if entry.get("executable") else 0o644)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
