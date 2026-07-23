from __future__ import annotations

import hashlib
import json
import re
import tempfile
import unittest
from pathlib import Path

from scripts.release_contract import (
    EXPECTED_IMAGE,
    ROOT,
    appimagetool_command,
    appstream_validation_command,
    architectures,
    debian_artifact_filename,
    debian_version,
    matches_cleared_appimage_marker,
    replace_control_field,
    source_version,
)


class PackageContractTests(unittest.TestCase):
    def test_all_source_versions_match_the_beta_candidate(self) -> None:
        self.assertEqual(source_version(), "0.1.0-beta.2")

    def test_debian_metadata_and_artifact_versions_are_deliberately_distinct(
        self,
    ) -> None:
        self.assertEqual(debian_version("0.1.0-beta.2"), "0.1.0~beta.2")
        self.assertEqual(
            debian_artifact_filename("0.1.0-beta.2"),
            "quireforge_0.1.0.beta.2_amd64.deb",
        )
        self.assertNotIn("~", debian_artifact_filename("0.1.0-beta.2"))
        self.assertEqual(debian_version("0.1.0"), "0.1.0")
        self.assertEqual(
            debian_artifact_filename("0.1.0"),
            "quireforge_0.1.0_amd64.deb",
        )
        self.assertEqual(architectures(), ("x86_64", "amd64", "amd64"))

    def test_control_replacement_is_exact_and_fails_closed(self) -> None:
        control = "Package: quire-forge\nVersion: 0.1.0-beta.2\n"
        self.assertEqual(
            replace_control_field(control, "Package", "quireforge"),
            "Package: quireforge\nVersion: 0.1.0-beta.2\n",
        )
        with self.assertRaises(RuntimeError):
            replace_control_field(control, "Architecture", "amd64")

    def test_only_tauris_exact_linuxdeploy_marker_mutation_is_accepted(self) -> None:
        reviewed = b"\x7fELF\0\0\0\0AI\x02reviewed-linuxdeploy"
        expected = hashlib.sha256(reviewed).hexdigest()
        cleared = reviewed[:8] + b"\0\0\0" + reviewed[11:]
        tampered = cleared[:-1] + b"!"
        with tempfile.TemporaryDirectory() as temporary:
            candidate = Path(temporary) / "linuxdeploy-x86_64.AppImage"
            candidate.write_bytes(cleared)
            self.assertTrue(matches_cleared_appimage_marker(candidate, expected))
            candidate.write_bytes(tampered)
            self.assertFalse(matches_cleared_appimage_marker(candidate, expected))

    def test_appstream_validation_is_offline_and_not_skipped(self) -> None:
        metadata = Path("/app/usr/share/metainfo/quireforge.appdata.xml")
        self.assertEqual(
            appstream_validation_command("/usr/bin/appstreamcli", metadata),
            [
                "/usr/bin/appstreamcli",
                "validate",
                "--no-net",
                str(metadata),
            ],
        )
        command = appimagetool_command(
            Path("/tools/appimagetool"),
            Path("/tools/runtime"),
            Path("/app"),
            Path("/out/QuireForge.AppImage"),
        )
        self.assertEqual(
            command,
            [
                "/tools/appimagetool",
                "--no-appstream",
                "--runtime-file",
                "/tools/runtime",
                "/app",
                "/out/QuireForge.AppImage",
            ],
        )

    def test_packaging_images_and_tools_are_digest_pinned(self) -> None:
        dockerfile = (ROOT / "packaging/linux/Dockerfile").read_text(
            encoding="utf-8"
        )
        from_lines = [
            line for line in dockerfile.splitlines() if line.startswith("FROM ")
        ]
        self.assertEqual(len(from_lines), 3)
        for line in from_lines:
            self.assertRegex(line, r"@sha256:[0-9a-f]{64}(?:\s+AS\s+\w+)?$")
        self.assertIn(EXPECTED_IMAGE, dockerfile)
        self.assertIn("FROM rust:1.95.0-slim-bookworm@", dockerfile)
        cargo_manifest = (
            ROOT / "apps/desktop/src-tauri/Cargo.toml"
        ).read_text(encoding="utf-8")
        self.assertIn('rust-version = "1.95"', cargo_manifest)

        manifest = json.loads(
            (ROOT / "packaging/linux/tauri-tools.json").read_text(encoding="utf-8")
        )
        self.assertEqual(manifest["schemaVersion"], 1)
        self.assertEqual(len(manifest["tools"]), 6)
        for tool in manifest["tools"]:
            self.assertTrue(tool["url"].startswith("https://"))
            self.assertIsNone(re.search(r"https://[^/]*@", tool["url"]))
            self.assertRegex(tool["sha256"], r"^[0-9a-f]{64}$")
        raw_urls = [
            tool["url"]
            for tool in manifest["tools"]
            if "raw.githubusercontent.com" in tool["url"]
        ]
        self.assertTrue(all(re.search(r"/[0-9a-f]{40}/", url) for url in raw_urls))

    def test_tauri_bundle_contract_is_active_and_canonical(self) -> None:
        config = json.loads(
            (ROOT / "apps/desktop/src-tauri/tauri.conf.json").read_text(
                encoding="utf-8"
            )
        )
        bundle = config["bundle"]
        self.assertTrue(bundle["active"])
        self.assertEqual(bundle["targets"], ["appimage", "deb"])
        self.assertEqual(bundle["category"], "DeveloperTool")
        self.assertEqual(bundle["license"], "Apache-2.0")
        self.assertEqual(
            bundle["homepage"], "https://quireforge.jamesjennison.net"
        )
        self.assertEqual(
            bundle["linux"]["deb"]["desktopTemplate"],
            "desktop-template.desktop",
        )
        self.assertEqual(
            bundle["linux"]["appimage"]["files"]["/quireforge.png"],
            "icons/icon.png",
        )
        metainfo = (
            ROOT
            / "apps/desktop/src-tauri/metainfo"
            / "io.github.codeframe78.QuireForge.metainfo.xml"
        )
        self.assertTrue(metainfo.is_file())


if __name__ == "__main__":
    unittest.main()
