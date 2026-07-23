"""Shared, dependency-free QuireForge release-contract helpers."""

from __future__ import annotations

import hashlib
import json
import os
import platform
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
TAURI_CONFIG = ROOT / "apps/desktop/src-tauri/tauri.conf.json"
TAURI_CARGO = ROOT / "apps/desktop/src-tauri/Cargo.toml"
TOOL_MANIFEST = ROOT / "packaging/linux/tauri-tools.json"
SCHEMA_PATH = ROOT / "packaging/release-manifest.schema.json"
CANONICAL_DESKTOP = "io.github.codeframe78.QuireForge.desktop"
LEGACY_DESKTOP = "QuireForge.desktop"
APPIMAGE_BASENAME = "QuireForge"
DEBIAN_PACKAGE = "quireforge"
EXPECTED_IMAGE = (
    "ubuntu:22.04@sha256:"
    "0e0a0fc6d18feda9db1590da249ac93e8d5abfea8f4c3c0c849ce512b5ef8982"
)
SEMVER_RE = re.compile(
    r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\."
    r"(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>[0-9A-Za-z.-]+))?$"
)


def run(
    arguments: list[str],
    *,
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    capture: bool = False,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        arguments,
        cwd=cwd,
        env=env,
        check=True,
        text=True,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
    )


def appimagetool_command(
    appimagetool: Path,
    runtime: Path,
    appdir: Path,
    output: Path,
) -> list[str]:
    return [
        str(appimagetool),
        "--no-appstream",
        "--runtime-file",
        str(runtime),
        str(appdir),
        str(output),
    ]


def appstream_validation_command(validator: str, metadata: Path) -> list[str]:
    return [validator, "validate", "--no-net", str(metadata)]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def source_version() -> str:
    versions = {
        "root package": json.loads(
            (ROOT / "package.json").read_text(encoding="utf-8")
        )["version"],
        "desktop package": json.loads(
            (ROOT / "apps/desktop/package.json").read_text(encoding="utf-8")
        )["version"],
        "website package": json.loads(
            (ROOT / "apps/website/package.json").read_text(encoding="utf-8")
        )["version"],
    }
    cargo_text = TAURI_CARGO.read_text(encoding="utf-8")
    cargo_match = re.search(
        r"(?ms)^\[package\]\s.*?^version\s*=\s*\"([^\"]+)\"",
        cargo_text,
    )
    if not cargo_match:
        raise RuntimeError("Cargo package version is missing")
    versions["Cargo package"] = cargo_match.group(1)

    distinct = set(versions.values())
    if len(distinct) != 1:
        details = ", ".join(f"{name}={value}" for name, value in versions.items())
        raise RuntimeError(f"release versions disagree: {details}")
    version = distinct.pop()
    if not SEMVER_RE.fullmatch(version):
        raise RuntimeError(f"unsupported release version: {version}")
    return version


def debian_version(version: str) -> str:
    match = SEMVER_RE.fullmatch(version)
    if not match:
        raise RuntimeError(f"unsupported release version: {version}")
    base = f"{match.group('major')}.{match.group('minor')}.{match.group('patch')}"
    prerelease = match.group("prerelease")
    return f"{base}~{prerelease}" if prerelease else base


def debian_artifact_filename(version: str, architecture: str = "amd64") -> str:
    """Return the GitHub-safe outer filename for a Debian package.

    Debian prereleases retain ``~`` in their control metadata so they sort
    before the corresponding stable version. GitHub Releases normalizes ``~``
    in uploaded asset names, so the outer filename deliberately uses ``.``.
    """
    artifact_version = debian_version(version).replace("~", ".")
    return f"{DEBIAN_PACKAGE}_{artifact_version}_{architecture}.deb"


def architectures() -> tuple[str, str, str]:
    machine = platform.machine()
    if machine not in {"x86_64", "amd64"}:
        raise RuntimeError(
            f"Milestone 20 packages support only x86_64, not {machine or 'unknown'}"
        )
    return ("x86_64", "amd64", "amd64")


def cargo_target_dir() -> Path:
    configured = os.environ.get("CARGO_TARGET_DIR")
    if not configured:
        return ROOT / "target"
    path = Path(configured)
    return path.resolve() if path.is_absolute() else (ROOT / path).resolve()


def package_output_dir() -> Path:
    override = os.environ.get("QUIRE_FORGE_PACKAGE_DIR")
    if override:
        path = Path(override)
        return path.resolve() if path.is_absolute() else (ROOT / path).resolve()
    return cargo_target_dir() / "release/packages"


def source_date_epoch() -> int:
    configured = os.environ.get("SOURCE_DATE_EPOCH")
    if configured:
        if not configured.isdigit():
            raise RuntimeError("SOURCE_DATE_EPOCH must be a positive integer")
        return int(configured)
    result = run(
        ["git", "log", "-1", "--format=%ct"],
        cwd=ROOT,
        capture=True,
    )
    value = result.stdout.strip()
    if not value.isdigit():
        raise RuntimeError("could not derive SOURCE_DATE_EPOCH from Git")
    return int(value)


def source_record() -> tuple[str, str, str | None]:
    commit = run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        capture=True,
    ).stdout.strip()
    status = run(
        ["git", "status", "--short", "--untracked-files=all"],
        cwd=ROOT,
        capture=True,
    ).stdout
    if not status:
        return commit, "clean", None
    diff = run(
        ["git", "diff", "--binary", "HEAD", "--", "."],
        cwd=ROOT,
        capture=True,
    ).stdout.encode("utf-8")
    untracked = "\n".join(
        line[3:]
        for line in status.splitlines()
        if line.startswith("?? ")
    ).encode("utf-8")
    digest = hashlib.sha256(diff + b"\0" + untracked).hexdigest()
    return commit, "dirty", digest


def builder_record() -> dict[str, str]:
    distribution = os.environ.get("QUIRE_FORGE_BUILD_DISTRIBUTION", "ubuntu")
    version = os.environ.get("QUIRE_FORGE_BUILD_VERSION", "host")
    image = os.environ.get("QUIRE_FORGE_BUILD_IMAGE", "unverified-host")
    return {
        "distribution": distribution,
        "version": version,
        "architecture": "x86_64",
        "image": image,
    }


def tauri_cache_root() -> Path:
    override = os.environ.get("QUIRE_FORGE_TAURI_CACHE_DIR")
    if override:
        return Path(override).expanduser().resolve()
    return Path.home() / ".cache/tauri"


def matches_cleared_appimage_marker(path: Path, expected: str) -> bool:
    """Recognize the exact three-byte marker change made by tauri-bundler."""
    digest = hashlib.sha256()
    first_chunk = True
    with path.open("rb") as source:
        for raw_chunk in iter(lambda: source.read(1024 * 1024), b""):
            chunk = bytearray(raw_chunk)
            if first_chunk:
                if len(chunk) < 11 or chunk[:4] != b"\x7fELF":
                    return False
                if chunk[8:11] != b"\0\0\0":
                    return False
                chunk[8:11] = b"AI\x02"
                first_chunk = False
            digest.update(chunk)
    return not first_chunk and digest.hexdigest() == expected


def verify_tauri_tools(*, allow_linuxdeploy_marker_cleared: bool = False) -> None:
    manifest = json.loads(TOOL_MANIFEST.read_text(encoding="utf-8"))
    if manifest.get("schemaVersion") != 1:
        raise RuntimeError("unsupported Tauri tool manifest schema")
    root = tauri_cache_root()
    for entry in manifest.get("tools", []):
        path = root / entry["filename"]
        if not path.is_file():
            raise RuntimeError(f"pinned Tauri tool is missing: {path}")
        actual = sha256(path)
        if actual != entry["sha256"]:
            if (
                allow_linuxdeploy_marker_cleared
                and entry["filename"] == "linuxdeploy-x86_64.AppImage"
                and matches_cleared_appimage_marker(path, entry["sha256"])
            ):
                continue
            raise RuntimeError(
                f"pinned Tauri tool checksum mismatch: {entry['filename']}"
            )


def replace_control_field(text: str, field: str, value: str) -> str:
    pattern = re.compile(rf"(?m)^{re.escape(field)}:\s*.*$")
    updated, count = pattern.subn(f"{field}: {value}", text, count=1)
    if count != 1:
        raise RuntimeError(f"Debian control field is missing or duplicated: {field}")
    return updated


def set_tree_timestamp(root: Path, timestamp: int) -> None:
    for path in sorted(root.rglob("*"), key=lambda item: len(item.parts), reverse=True):
        try:
            os.utime(path, (timestamp, timestamp), follow_symlinks=False)
        except (NotImplementedError, PermissionError):
            if not path.is_symlink():
                raise
    os.utime(root, (timestamp, timestamp))


def write_sha256sums(output_dir: Path, artifacts: list[Path]) -> None:
    lines = [f"{sha256(path)}  {path.name}" for path in sorted(artifacts)]
    (output_dir / "SHA256SUMS").write_text(
        "\n".join(lines) + "\n",
        encoding="utf-8",
    )
