#!/usr/bin/env python3
"""Run dependency-free checks for QuireForge's tracked repository sources."""

from __future__ import annotations

import hashlib
import json
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
    ".npmrc",
    "Cargo.lock",
    "Cargo.toml",
    "package.json",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "apps/website/astro.config.mjs",
    "apps/website/package.json",
    "apps/website/public/_headers",
    "apps/website/src/data/site.ts",
    "apps/website/src/pages/404.astro",
    "apps/website/src/pages/index.astro",
    "apps/desktop/fixtures/desktop-bootstrap.json",
    "apps/desktop/fixtures/codex-model-list-response.json",
    "apps/desktop/fixtures/codex-runtime.json",
    "apps/desktop/fixtures/codex-auth.json",
    "apps/desktop/fixtures/conversation.json",
    "apps/desktop/fixtures/project-workspace.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/manifest.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v1/InitializeParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v1/InitializeResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ModelListParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ModelListResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/AccountLoginCompletedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/AccountUpdatedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/CancelLoginAccountParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/CancelLoginAccountResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/GetAccountParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/GetAccountResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/LoginAccountParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/LoginAccountResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/LogoutAccountResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadStartParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadStartResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnStartParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnStartResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnInterruptParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnInterruptResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadStartedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnStartedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnCompletedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/AgentMessageDeltaNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ReasoningSummaryTextDeltaNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnPlanUpdatedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ItemStartedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ItemCompletedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ErrorNotification.json",
    "apps/desktop/package.json",
    "apps/desktop/src/App.tsx",
    "apps/desktop/src/ProjectWorkspace.tsx",
    "apps/desktop/src/lib/bridge.ts",
    "apps/desktop/src/lib/auth.ts",
    "apps/desktop/src/lib/codex.ts",
    "apps/desktop/src/lib/conversation.ts",
    "apps/desktop/src/lib/project.ts",
    "apps/desktop/src-tauri/Cargo.toml",
    "apps/desktop/src-tauri/capabilities/main.json",
    "apps/desktop/src-tauri/tauri.conf.json",
    "apps/desktop/src-tauri/src/codex/app_server.rs",
    "apps/desktop/src-tauri/src/codex/auth/mod.rs",
    "apps/desktop/src-tauri/src/codex/auth/types.rs",
    "apps/desktop/src-tauri/src/codex/backend.rs",
    "apps/desktop/src-tauri/src/codex/conversation/mod.rs",
    "apps/desktop/src-tauri/src/codex/conversation/types.rs",
    "apps/desktop/src-tauri/src/codex/probe.rs",
    "apps/desktop/src-tauri/src/project/identity.rs",
    "apps/desktop/src-tauri/src/project/mod.rs",
    "apps/desktop/src-tauri/src/project/storage.rs",
    "apps/desktop/src-tauri/src/project/types.rs",
    "docs/ARCHITECTURE.md",
    "docs/BUILDING.md",
    "docs/LOCAL-BUILD-PERFORMANCE.md",
    "docs/MILESTONE-FORECASTS.md",
    "docs/ROADMAP.md",
    "docs/TESTING.md",
    "docs/THREAT-MODEL.md",
    "docs/WEBSITE.md",
    "docs/DECISIONS/0007-quireforge-metadata-sqlite.md",
    "docs/DECISIONS/0008-native-conversation-runtime.md",
    "scripts/generate_codex_schema_fixtures.py",
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
    "apps/desktop/src-tauri/tauri.conf.json": (
        '"productName": "QuireForge"',
        '"mainBinaryName": "quireforge"',
        '"identifier": "io.github.codeframe78.QuireForge"',
        '"enableGTKAppId": true',
        '"capabilities": ["main"]',
    ),
    "apps/desktop/fixtures/desktop-bootstrap.json": (
        '"name": "QuireForge"',
        '"executable": "quireforge"',
        '"identifier": "io.github.codeframe78.QuireForge"',
        '"id": "project-attachments"',
        '"state": "ready"',
        '"id": "conversation-runtime"',
    ),
    "apps/desktop/fixtures/codex-runtime.json": (
        '"schemaVersion": 1',
        '"adapterVersion": "codex-app-server-v2"',
        '"backend": "app-server-stdio"',
        '"diagnosticCode": null',
    ),
    "apps/desktop/fixtures/project-workspace.json": (
        '"schemaVersion": 1',
        '"state": "empty"',
        '"pendingAttachment": null',
    ),
    "apps/desktop/fixtures/conversation.json": (
        '"schemaVersion": 1',
        '"state": "empty"',
        '"conversationId": null',
        '"events": []',
    ),
    "apps/desktop/src-tauri/src/lib.rs": (
        "codex_runtime_probe",
        "CodexRuntimeService::default()",
        "codex_auth_status",
        "codex_auth_start",
        "codex_auth_cancel",
        "codex_auth_logout",
        "CodexAuthService::default()",
        "project_workspace_status",
        "project_pick_directory",
        "project_preflight",
        "ProjectService::open",
        "conversation_status",
        "conversation_start",
        "conversation_poll",
        "conversation_interrupt",
        "ConversationService::default()",
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

    capability_path = ROOT / "apps/desktop/src-tauri/capabilities/main.json"
    if capability_path.is_file():
        capability = json.loads(capability_path.read_text(encoding="utf-8"))
        if capability.get("windows") != ["main"]:
            errors.append("desktop capability must target only the main window")
        if capability.get("platforms") != ["linux"]:
            errors.append("desktop capability must target only Linux")
        if capability.get("permissions") != []:
            errors.append("desktop capability must grant no broad plugin permissions")

    tauri_path = ROOT / "apps/desktop/src-tauri/tauri.conf.json"
    if tauri_path.is_file():
        tauri_config = json.loads(tauri_path.read_text(encoding="utf-8"))
        security = tauri_config.get("app", {}).get("security", {})
        if not security.get("csp"):
            errors.append("desktop production CSP must be explicit")
        if tauri_config.get("bundle", {}).get("active") is not False:
            errors.append("desktop packaging must remain disabled before Milestone 19")

    schema_root = ROOT / "apps/desktop/fixtures/codex-schema/0.144.6"
    manifest_path = schema_root / "manifest.json"
    if manifest_path.is_file():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        expected_schema_paths = {
            "v1/InitializeParams.json",
            "v1/InitializeResponse.json",
            "v2/AccountLoginCompletedNotification.json",
            "v2/AccountUpdatedNotification.json",
            "v2/CancelLoginAccountParams.json",
            "v2/CancelLoginAccountResponse.json",
            "v2/GetAccountParams.json",
            "v2/GetAccountResponse.json",
            "v2/LoginAccountParams.json",
            "v2/LoginAccountResponse.json",
            "v2/LogoutAccountResponse.json",
            "v2/ModelListParams.json",
            "v2/ModelListResponse.json",
            "v2/ThreadStartParams.json",
            "v2/ThreadStartResponse.json",
            "v2/TurnStartParams.json",
            "v2/TurnStartResponse.json",
            "v2/TurnInterruptParams.json",
            "v2/TurnInterruptResponse.json",
            "v2/ThreadStartedNotification.json",
            "v2/TurnStartedNotification.json",
            "v2/TurnCompletedNotification.json",
            "v2/AgentMessageDeltaNotification.json",
            "v2/ReasoningSummaryTextDeltaNotification.json",
            "v2/TurnPlanUpdatedNotification.json",
            "v2/ItemStartedNotification.json",
            "v2/ItemCompletedNotification.json",
            "v2/ErrorNotification.json",
        }
        manifest_files = manifest.get("files", [])
        recorded_paths = {
            entry.get("path") for entry in manifest_files if isinstance(entry, dict)
        }
        if manifest.get("codexCliVersion") != "0.144.6":
            errors.append("Codex schema manifest must record CLI 0.144.6")
        if recorded_paths != expected_schema_paths:
            errors.append("Codex schema manifest must contain only the reviewed subset")

        for entry in manifest_files:
            if not isinstance(entry, dict):
                errors.append("Codex schema manifest contains a malformed entry")
                continue
            relative = entry.get("path")
            digest = entry.get("sha256")
            if not isinstance(relative, str) or relative not in expected_schema_paths:
                continue
            schema_path = schema_root / relative
            if not schema_path.is_file():
                continue
            actual = hashlib.sha256(schema_path.read_bytes()).hexdigest()
            if digest != actual:
                errors.append(f"Codex schema hash mismatch: {relative}")

    runtime_fixture_path = ROOT / "apps/desktop/fixtures/codex-runtime.json"
    if runtime_fixture_path.is_file():
        runtime_fixture = json.loads(runtime_fixture_path.read_text(encoding="utf-8"))
        serialized_fixture = json.dumps(runtime_fixture)
        for forbidden_field in ("accountId", "codexHome", "installationId", "userAgent"):
            if forbidden_field in serialized_fixture:
                errors.append(f"Codex runtime fixture contains raw field: {forbidden_field}")

    auth_fixture_path = ROOT / "apps/desktop/fixtures/codex-auth.json"
    if auth_fixture_path.is_file():
        auth_fixture = json.loads(auth_fixture_path.read_text(encoding="utf-8"))
        serialized_fixture = json.dumps(auth_fixture)
        for forbidden_field in (
            "accountId",
            "email",
            "loginId",
            "accessToken",
            "apiKey",
        ):
            if forbidden_field in serialized_fixture:
                errors.append(f"Codex auth fixture contains raw field: {forbidden_field}")

    conversation_fixture_path = ROOT / "apps/desktop/fixtures/conversation.json"
    if conversation_fixture_path.is_file():
        conversation_fixture = json.loads(
            conversation_fixture_path.read_text(encoding="utf-8")
        )
        serialized_fixture = json.dumps(conversation_fixture)
        for forbidden_field in (
            "threadId",
            "turnId",
            "cwd",
            "selectedPath",
            "resolvedPath",
            "rawMessage",
        ):
            if forbidden_field in serialized_fixture:
                errors.append(
                    f"Conversation fixture contains raw field: {forbidden_field}"
                )

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
