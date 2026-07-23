#!/usr/bin/env python3
"""Normalize Tauri's Linux bundles into QuireForge release candidates."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import tempfile
from pathlib import Path

from release_contract import (
    APPIMAGE_BASENAME,
    CANONICAL_DESKTOP,
    DEBIAN_PACKAGE,
    LEGACY_DESKTOP,
    ROOT,
    appimagetool_command,
    architectures,
    builder_record,
    cargo_target_dir,
    debian_version,
    package_output_dir,
    replace_control_field,
    run,
    set_tree_timestamp,
    sha256,
    source_date_epoch,
    source_record,
    source_version,
    tauri_cache_root,
    verify_tauri_tools,
    write_sha256sums,
)


def one_match(root: Path, pattern: str, label: str) -> Path:
    matches = sorted(root.glob(pattern))
    if len(matches) != 1:
        rendered = ", ".join(path.name for path in matches) or "none"
        raise RuntimeError(f"expected one {label}, found {rendered}")
    return matches[0]


def rebuild_debian(
    raw_package: Path,
    output: Path,
    version: str,
    timestamp: int,
) -> None:
    with tempfile.TemporaryDirectory(prefix="quireforge-deb-") as temporary:
        root = Path(temporary) / "root"
        run(["dpkg-deb", "--raw-extract", str(raw_package), str(root)])

        control_path = root / "DEBIAN/control"
        control = control_path.read_text(encoding="utf-8")
        control = replace_control_field(control, "Package", DEBIAN_PACKAGE)
        control = replace_control_field(control, "Version", debian_version(version))
        control_path.write_text(control, encoding="utf-8")

        applications = root / "usr/share/applications"
        generated_desktop = applications / LEGACY_DESKTOP
        canonical_desktop = applications / CANONICAL_DESKTOP
        if not generated_desktop.is_file() or canonical_desktop.exists():
            raise RuntimeError("unexpected Debian desktop-entry layout")
        generated_desktop.rename(canonical_desktop)

        package_files = sorted(
            path
            for path in root.rglob("*")
            if path.is_file() and root / "DEBIAN" not in path.parents
        )
        md5_lines = []
        for path in package_files:
            digest = hashlib.md5(path.read_bytes(), usedforsecurity=False).hexdigest()
            md5_lines.append(f"{digest}  {path.relative_to(root).as_posix()}")
        (root / "DEBIAN/md5sums").write_text(
            "\n".join(md5_lines) + "\n",
            encoding="utf-8",
        )

        set_tree_timestamp(root, timestamp)
        environment = os.environ.copy()
        environment["SOURCE_DATE_EPOCH"] = str(timestamp)
        run(
            [
                "dpkg-deb",
                "--root-owner-group",
                "--uniform-compression",
                "-Zxz",
                "-z9",
                "--build",
                str(root),
                str(output),
            ],
            env=environment,
        )


def rebuild_appimage(
    raw_appimage: Path,
    output: Path,
    version: str,
    timestamp: int,
) -> None:
    with tempfile.TemporaryDirectory(prefix="quireforge-appimage-") as temporary:
        temporary_root = Path(temporary)
        raw_copy = temporary_root / raw_appimage.name
        shutil.copy2(raw_appimage, raw_copy)
        raw_copy.chmod(0o755)
        run([str(raw_copy), "--appimage-extract"], cwd=temporary_root)

        appdir = temporary_root / "squashfs-root"
        applications = appdir / "usr/share/applications"
        generated_desktop = applications / LEGACY_DESKTOP
        canonical_desktop = applications / CANONICAL_DESKTOP
        if not generated_desktop.is_file() or canonical_desktop.exists():
            raise RuntimeError("unexpected AppImage desktop-entry layout")
        generated_desktop.rename(canonical_desktop)

        legacy_link = appdir / LEGACY_DESKTOP
        if not legacy_link.is_symlink():
            raise RuntimeError("AppImage root desktop entry is not the expected symlink")
        legacy_link.unlink()
        (appdir / CANONICAL_DESKTOP).symlink_to(
            f"usr/share/applications/{CANONICAL_DESKTOP}"
        )

        if not (appdir / "quireforge.png").is_file():
            raise RuntimeError("AppImage root icon is missing")
        metainfo = (
            appdir
            / "usr/share/metainfo/io.github.codeframe78.QuireForge.metainfo.xml"
        )
        appdata = metainfo.with_name(
            "io.github.codeframe78.QuireForge.appdata.xml"
        )
        if not metainfo.is_file() or appdata.exists():
            raise RuntimeError("unexpected AppImage AppStream metadata layout")
        metainfo.rename(appdata)

        set_tree_timestamp(appdir, timestamp)
        cache_root = tauri_cache_root()
        plugin = cache_root / "linuxdeploy-plugin-appimage.AppImage"
        plugin_root = temporary_root / "plugin"
        plugin_root.mkdir()
        run([str(plugin), "--appimage-extract"], cwd=plugin_root)
        appimagetool = plugin_root / "squashfs-root/appimagetool-prefix/AppRun"
        runtime = cache_root / "runtime-x86_64"
        if not appimagetool.is_file() or not runtime.is_file():
            raise RuntimeError("pinned AppImage repacking tools are incomplete")
        environment = os.environ.copy()
        environment.update(
            {
                "ARCH": "x86_64",
                "SOURCE_DATE_EPOCH": str(timestamp),
            }
        )
        generated = temporary_root / output.name
        run(
            appimagetool_command(appimagetool, runtime, appdir, generated),
            cwd=temporary_root,
            env=environment,
        )
        if not generated.is_file():
            raise RuntimeError("normalized AppImage was not created")
        shutil.move(generated, output)
        output.chmod(0o755)


def main() -> int:
    # tauri-bundler clears linuxdeploy's three-byte AppImage marker after it
    # verifies and extracts the reviewed tool. Accept only that exact mutation.
    verify_tauri_tools(allow_linuxdeploy_marker_cleared=True)
    version = source_version()
    release_arch, tauri_arch, deb_arch = architectures()
    target = cargo_target_dir()
    bundle_root = target / "release/bundle"
    raw_deb = one_match(
        bundle_root / "deb",
        f"{APPIMAGE_BASENAME}_{version}_{tauri_arch}.deb",
        "raw Debian package",
    )
    raw_appimage = one_match(
        bundle_root / "appimage",
        f"{APPIMAGE_BASENAME}_{version}_{tauri_arch}.AppImage",
        "raw AppImage",
    )

    output_dir = package_output_dir()
    output_dir.mkdir(parents=True, exist_ok=True)
    for existing in output_dir.iterdir():
        if existing.is_file() and existing.name in {
            "SHA256SUMS",
            "release-manifest.json",
        }:
            existing.unlink()
        elif existing.is_file() and (
            existing.suffix == ".deb" or existing.name.endswith(".AppImage")
        ):
            existing.unlink()
        else:
            raise RuntimeError(f"refusing unexpected package output: {existing}")

    timestamp = source_date_epoch()
    deb_output = output_dir / f"{DEBIAN_PACKAGE}_{debian_version(version)}_{deb_arch}.deb"
    appimage_output = (
        output_dir / f"{APPIMAGE_BASENAME}-{version}-{release_arch}.AppImage"
    )
    rebuild_debian(raw_deb, deb_output, version, timestamp)
    rebuild_appimage(raw_appimage, appimage_output, version, timestamp)

    commit, tree_state, diff_digest = source_record()
    artifacts = []
    for artifact_format, path, package_version in (
        ("appimage", appimage_output, version),
        ("deb", deb_output, debian_version(version)),
    ):
        artifacts.append(
            {
                "format": artifact_format,
                "filename": path.name,
                "architecture": release_arch,
                "packageVersion": package_version,
                "sha256": sha256(path),
                "size": path.stat().st_size,
            }
        )

    source = {"commit": commit, "treeState": tree_state}
    if diff_digest:
        source["diffSha256"] = diff_digest
    manifest = {
        "schemaVersion": 1,
        "state": (
            "release-candidate" if tree_state == "clean" else "local-candidate"
        ),
        "version": version,
        "source": source,
        "builder": builder_record(),
        "artifacts": artifacts,
    }
    (output_dir / "release-manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    write_sha256sums(output_dir, [appimage_output, deb_output])
    print(f"normalized Linux release candidates: {output_dir.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
