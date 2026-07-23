#!/usr/bin/env python3
"""Validate QuireForge Linux package artifacts and disposable lifecycle."""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import tempfile
from pathlib import Path

from release_contract import (
    CANONICAL_DESKTOP,
    DEBIAN_PACKAGE,
    EXPECTED_IMAGE,
    LEGACY_DESKTOP,
    ROOT,
    appstream_validation_command,
    debian_artifact_filename,
    debian_version,
    package_output_dir,
    replace_control_field,
    run,
    sha256,
    source_version,
)


EXPECTED_DEPENDENCIES = {"libgtk-3-0", "libwebkit2gtk-4.1-0"}
GLIBC_BASELINE = (2, 35)
EXPECTED_FILES = {
    "release-manifest.json",
    "SHA256SUMS",
}
DESKTOP_FIELDS = {
    "Categories": "Development;IDE;",
    "Comment": "An unofficial native Linux workspace for Codex",
    "Exec": "quireforge",
    "Icon": "quireforge",
    "Name": "QuireForge",
    "StartupNotify": "true",
    "StartupWMClass": "quireforge",
    "Terminal": "false",
    "Type": "Application",
}


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--artifact-dir",
        type=Path,
        default=package_output_dir(),
    )
    parser.add_argument("--lifecycle", action="store_true")
    parser.add_argument("--smoke", action="store_true")
    parser.add_argument("--require-publishable", action="store_true")
    parser.add_argument("--expected-tag")
    return parser.parse_args()


def parse_desktop(path: Path) -> dict[str, str]:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != "[Desktop Entry]":
        raise RuntimeError(f"invalid desktop entry header: {path}")
    result = {}
    for line in lines[1:]:
        if not line or line.startswith("#"):
            continue
        key, separator, value = line.partition("=")
        if separator != "=" or key in result:
            raise RuntimeError(f"invalid desktop entry line: {line}")
        result[key] = value
    return result


def validate_desktop(path: Path) -> None:
    fields = parse_desktop(path)
    if fields != DESKTOP_FIELDS:
        raise RuntimeError(f"desktop entry fields do not match contract: {fields}")
    validator = shutil.which("desktop-file-validate")
    if validator:
        run([validator, str(path)])


def validate_metainfo(path: Path) -> None:
    validator = shutil.which("appstreamcli")
    if validator:
        run(appstream_validation_command(validator, path))


def validate_glibc_baseline(path: Path) -> None:
    result = run(["readelf", "--version-info", str(path)], capture=True)
    versions = {
        (int(major), int(minor))
        for major, minor in re.findall(r"GLIBC_(\d+)\.(\d+)", result.stdout)
    }
    if not versions:
        raise RuntimeError(f"no GLIBC version contract found in {path}")
    newest = max(versions)
    if newest > GLIBC_BASELINE:
        rendered = ".".join(str(component) for component in newest)
        raise RuntimeError(f"{path} requires GLIBC {rendered}, newer than Ubuntu 22.04")


def validate_manifest(
    artifact_dir: Path,
    require_publishable: bool,
    expected_tag: str | None,
) -> tuple[dict[str, object], dict[str, Path]]:
    manifest_path = artifact_dir / "release-manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    version = source_version()
    if manifest.get("schemaVersion") != 1 or manifest.get("version") != version:
        raise RuntimeError("release manifest schema or version mismatch")

    source = manifest.get("source")
    builder = manifest.get("builder")
    if not isinstance(source, dict) or not isinstance(builder, dict):
        raise RuntimeError("release manifest source or builder is malformed")
    if not re.fullmatch(r"[0-9a-f]{40}", str(source.get("commit", ""))):
        raise RuntimeError("release manifest source commit is invalid")
    if source.get("treeState") not in {"clean", "dirty"}:
        raise RuntimeError("release manifest tree state is invalid")

    if require_publishable:
        if manifest.get("state") != "release-candidate":
            raise RuntimeError("publication requires a clean release candidate")
        if source.get("treeState") != "clean" or "diffSha256" in source:
            raise RuntimeError("publication requires a clean source tree")
        if builder != {
            "distribution": "ubuntu",
            "version": "22.04",
            "architecture": "x86_64",
            "image": EXPECTED_IMAGE,
        }:
            raise RuntimeError("publication requires the pinned Ubuntu 22.04 builder")
        if expected_tag != f"v{version}":
            raise RuntimeError(
                f"release tag must be v{version}, not {expected_tag or 'unset'}"
            )

    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, list) or len(artifacts) != 2:
        raise RuntimeError("release manifest must contain exactly two artifacts")
    by_format: dict[str, Path] = {}
    for entry in artifacts:
        if not isinstance(entry, dict):
            raise RuntimeError("release artifact entry is malformed")
        artifact_format = entry.get("format")
        filename = entry.get("filename")
        if artifact_format not in {"appimage", "deb"} or not isinstance(filename, str):
            raise RuntimeError("release artifact format or filename is invalid")
        path = artifact_dir / filename
        if path.parent != artifact_dir or not path.is_file():
            raise RuntimeError(f"release artifact is missing: {filename}")
        if entry.get("sha256") != sha256(path):
            raise RuntimeError(f"release artifact checksum mismatch: {filename}")
        if entry.get("size") != path.stat().st_size:
            raise RuntimeError(f"release artifact size mismatch: {filename}")
        if entry.get("architecture") != "x86_64":
            raise RuntimeError("release artifact architecture mismatch")
        by_format[artifact_format] = path

    if set(by_format) != {"appimage", "deb"}:
        raise RuntimeError("release manifest artifact formats are incomplete")
    expected_names = EXPECTED_FILES | {path.name for path in by_format.values()}
    actual_names = {path.name for path in artifact_dir.iterdir() if path.is_file()}
    if actual_names != expected_names:
        raise RuntimeError(
            f"unexpected release artifact set: {sorted(actual_names ^ expected_names)}"
        )

    checksum_lines = (artifact_dir / "SHA256SUMS").read_text(
        encoding="utf-8"
    ).splitlines()
    expected_lines = [
        f"{sha256(path)}  {path.name}" for path in sorted(by_format.values())
    ]
    if checksum_lines != expected_lines:
        raise RuntimeError("SHA256SUMS does not match the release artifacts")
    return manifest, by_format


def deb_field(package: Path, field: str) -> str:
    result = run(
        ["dpkg-deb", "--field", str(package), field],
        capture=True,
    )
    return result.stdout.strip()


def validate_debian(package: Path, version: str) -> None:
    expected_name = debian_artifact_filename(version)
    if package.name != expected_name:
        raise RuntimeError(f"Debian filename mismatch: {package.name}")
    expected_fields = {
        "Package": DEBIAN_PACKAGE,
        "Version": debian_version(version),
        "Architecture": "amd64",
        "Homepage": "https://quireforge.jamesjennison.net",
        "Section": "devel",
        "Priority": "optional",
    }
    for field, expected in expected_fields.items():
        actual = deb_field(package, field)
        if actual != expected:
            raise RuntimeError(f"Debian {field} mismatch: {actual}")
    dependencies = {
        item.strip().split(" ", 1)[0]
        for item in deb_field(package, "Depends").split(",")
    }
    if not EXPECTED_DEPENDENCIES.issubset(dependencies):
        raise RuntimeError(f"Debian dependencies are incomplete: {dependencies}")

    with tempfile.TemporaryDirectory(prefix="quireforge-validate-deb-") as temporary:
        root = Path(temporary)
        run(["dpkg-deb", "--raw-extract", str(package), str(root)])
        if any((root / "DEBIAN").glob("*inst")) or any(
            (root / "DEBIAN").glob("*rm")
        ):
            raise RuntimeError("QuireForge packages must not contain maintainer scripts")
        canonical = root / "usr/share/applications" / CANONICAL_DESKTOP
        legacy = root / "usr/share/applications" / LEGACY_DESKTOP
        if legacy.exists() or not canonical.is_file():
            raise RuntimeError("Debian desktop filename does not match the canonical ID")
        validate_desktop(canonical)
        binary = root / "usr/bin/quireforge"
        if not binary.is_file() or not os.access(binary, os.X_OK):
            raise RuntimeError("Debian executable is missing or not executable")
        validate_glibc_baseline(binary)
        metainfo = (
            root
            / "usr/share/metainfo/io.github.codeframe78.QuireForge.metainfo.xml"
        )
        if not metainfo.is_file():
            raise RuntimeError("Debian AppStream metadata is missing")
        validate_metainfo(metainfo)
        md5sums = (root / "DEBIAN/md5sums").read_text(encoding="utf-8")
        md5_paths = {
            line.split(maxsplit=1)[1]
            for line in md5sums.splitlines()
            if len(line.split(maxsplit=1)) == 2
        }
        canonical_md5_path = f"usr/share/applications/{CANONICAL_DESKTOP}"
        legacy_md5_path = f"usr/share/applications/{LEGACY_DESKTOP}"
        if (
            canonical_md5_path not in md5_paths
            or legacy_md5_path in md5_paths
        ):
            raise RuntimeError("Debian md5sums retain the wrong desktop filename")


def validate_appimage(package: Path, version: str) -> None:
    expected_name = f"QuireForge-{version}-x86_64.AppImage"
    if package.name != expected_name:
        raise RuntimeError(f"AppImage filename mismatch: {package.name}")
    if not os.access(package, os.X_OK):
        raise RuntimeError("AppImage is not executable")

    with tempfile.TemporaryDirectory(
        prefix="quireforge-validate-appimage-"
    ) as temporary:
        root = Path(temporary)
        run([str(package), "--appimage-extract"], cwd=root)
        appdir = root / "squashfs-root"
        canonical = appdir / "usr/share/applications" / CANONICAL_DESKTOP
        legacy = appdir / "usr/share/applications" / LEGACY_DESKTOP
        root_link = appdir / CANONICAL_DESKTOP
        if legacy.exists() or not canonical.is_file():
            raise RuntimeError("AppImage desktop filename does not match canonical ID")
        if not root_link.is_symlink() or root_link.resolve() != canonical.resolve():
            raise RuntimeError("AppImage root desktop symlink is invalid")
        if (appdir / LEGACY_DESKTOP).exists():
            raise RuntimeError("AppImage retains the legacy root desktop entry")
        validate_desktop(canonical)
        binary = appdir / "usr/bin/quireforge"
        if not binary.is_file() or not os.access(binary, os.X_OK):
            raise RuntimeError("AppImage executable is missing or not executable")
        validate_glibc_baseline(binary)
        metainfo = (
            appdir
            / "usr/share/metainfo/io.github.codeframe78.QuireForge.metainfo.xml"
        )
        appdata = metainfo.with_name(
            "io.github.codeframe78.QuireForge.appdata.xml"
        )
        if metainfo.exists() or not appdata.is_file():
            raise RuntimeError("AppImage AppStream metadata layout is invalid")
        validate_metainfo(appdata)


def smoke_packages(debian: Path, appimage: Path) -> None:
    helper = ROOT / "scripts/smoke_linux_package.py"
    with tempfile.TemporaryDirectory(prefix="quireforge-smoke-deb-") as temporary:
        root = Path(temporary)
        run(["dpkg-deb", "--extract", str(debian), str(root)])
        debian_binary = root / "usr/bin/quireforge"
        run(
            [
                "xvfb-run",
                "--auto-servernum",
                "python3",
                str(helper),
                "--label",
                "Debian package",
                str(debian_binary),
            ]
        )
    run(
        [
            "xvfb-run",
            "--auto-servernum",
            "python3",
            str(helper),
            "--label",
            "AppImage",
            str(appimage),
            "--appimage-extract-and-run",
        ]
    )


def build_previous_package(current: Path, output: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="quireforge-previous-deb-") as temporary:
        root = Path(temporary)
        run(["dpkg-deb", "--raw-extract", str(current), str(root)])
        control = root / "DEBIAN/control"
        updated = replace_control_field(
            control.read_text(encoding="utf-8"),
            "Version",
            "0.0.0",
        )
        control.write_text(updated, encoding="utf-8")
        run(
            [
                "dpkg-deb",
                "--root-owner-group",
                "--build",
                str(root),
                str(output),
            ]
        )


def lifecycle(package: Path, expected_version: str) -> None:
    with tempfile.TemporaryDirectory(prefix="quireforge-lifecycle-") as temporary:
        root = Path(temporary) / "root"
        admin = root / "var/lib/dpkg"
        admin.mkdir(parents=True)
        (admin / "status").write_text("", encoding="utf-8")
        project = root / "home/tester/project/README.md"
        metadata = root / "home/tester/.local/share/quireforge/metadata.db"
        project.parent.mkdir(parents=True)
        metadata.parent.mkdir(parents=True)
        project.write_text("preserve project\n", encoding="utf-8")
        metadata.write_text("preserve metadata\n", encoding="utf-8")
        previous = Path(temporary) / "quireforge_0.0.0_amd64.deb"
        build_previous_package(package, previous)

        base = [
            "dpkg",
            f"--root={root}",
            "--force-not-root",
            "--force-depends",
            "--force-script-chrootless",
        ]
        run([*base, "--install", str(previous)])
        query = ["dpkg-query", f"--admindir={admin}", "--showformat=${Version}", "--show"]
        if run([*query, DEBIAN_PACKAGE], capture=True).stdout != "0.0.0":
            raise RuntimeError("disposable initial package installation failed")

        run([*base, "--install", str(package)])
        if run([*query, DEBIAN_PACKAGE], capture=True).stdout != expected_version:
            raise RuntimeError("disposable package upgrade failed")
        if not (root / "usr/bin/quireforge").is_file():
            raise RuntimeError("upgraded disposable executable is missing")

        run([*base, "--remove", DEBIAN_PACKAGE])
        if (root / "usr/bin/quireforge").exists():
            raise RuntimeError("package uninstall retained the executable")
        if (root / "usr/share/applications" / CANONICAL_DESKTOP).exists():
            raise RuntimeError("package uninstall retained the desktop entry")
        if project.read_text(encoding="utf-8") != "preserve project\n":
            raise RuntimeError("package uninstall altered the attached project")
        if metadata.read_text(encoding="utf-8") != "preserve metadata\n":
            raise RuntimeError("package uninstall altered application metadata")


def main() -> int:
    arguments = parse_arguments()
    artifact_dir = arguments.artifact_dir.resolve()
    manifest, artifacts = validate_manifest(
        artifact_dir,
        arguments.require_publishable,
        arguments.expected_tag,
    )
    version = str(manifest["version"])
    validate_debian(artifacts["deb"], version)
    validate_appimage(artifacts["appimage"], version)
    if arguments.lifecycle:
        lifecycle(artifacts["deb"], debian_version(version))
    if arguments.smoke:
        smoke_packages(artifacts["deb"], artifacts["appimage"])
    print(f"validated Linux release artifacts: {artifact_dir.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
