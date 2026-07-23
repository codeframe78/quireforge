#!/usr/bin/env python3
"""Run dependency-free checks for QuireForge's tracked repository sources."""

from __future__ import annotations

import ast
import hashlib
import json
import re
import subprocess
import sys
import urllib.parse
import xml.etree.ElementTree as ET
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
TAURI_VERSION_RE = re.compile(
    r'(?ms)^\[package\]\s.*?^version\s*=\s*"([^"]+)"'
)

REQUIRED_PATHS = (
    ".editorconfig",
    ".cargo/audit.toml",
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
    ".github/workflows/linux-release.yml",
    ".npmrc",
    "Cargo.lock",
    "Cargo.toml",
    "package.json",
    "pnpm-lock.yaml",
    "pnpm-workspace.yaml",
    "apps/website/astro.config.mjs",
    "apps/website/package.json",
    "apps/website/public/.htaccess",
    "apps/website/src/data/site.ts",
    "apps/website/src/data/downloads.ts",
    "apps/website/src/pages/404.astro",
    "apps/website/src/pages/index.astro",
    "apps/desktop/fixtures/desktop-bootstrap.json",
    "apps/desktop/fixtures/codex-model-list-response.json",
    "apps/desktop/fixtures/codex-runtime.json",
    "apps/desktop/fixtures/integration-catalog.json",
    "apps/desktop/fixtures/integration-control.json",
    "apps/desktop/fixtures/integration-mutation.json",
    "apps/desktop/fixtures/file-preview.json",
    "apps/desktop/fixtures/conversation-attachments.json",
    "apps/desktop/fixtures/codex-auth.json",
    "apps/desktop/fixtures/conversation.json",
    "apps/desktop/fixtures/session-lifecycle.json",
    "apps/desktop/fixtures/project-workspace.json",
    "apps/desktop/fixtures/git-workspace.json",
    "apps/desktop/fixtures/git-diff.json",
    "apps/desktop/fixtures/git-mutation-preview.json",
    "apps/desktop/fixtures/git-mutation-result.json",
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
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadListParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadListResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadReadParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadReadResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadResumeParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadResumeResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadForkParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadForkResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadArchiveParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadArchiveResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadUnarchiveParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadUnarchiveResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnStartParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnStartResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnInterruptParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnInterruptResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadStartedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadArchivedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ThreadUnarchivedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnStartedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnCompletedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/AgentMessageDeltaNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ReasoningSummaryTextDeltaNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/TurnPlanUpdatedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ItemStartedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/RequestId.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/CommandExecutionRequestApprovalParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/CommandExecutionRequestApprovalResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/FileChangeRequestApprovalParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/FileChangeRequestApprovalResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/PermissionsRequestApprovalParams.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/PermissionsRequestApprovalResponse.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/CommandExecutionOutputDeltaNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/McpToolCallProgressNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ServerRequestResolvedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ItemCompletedNotification.json",
    "apps/desktop/fixtures/codex-schema/0.144.6/v2/ErrorNotification.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/manifest.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/DynamicToolCallParams.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/DynamicToolCallResponse.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/v2/AppsListResponse.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/v2/PluginListResponse.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/v2/SkillsListResponse.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/v2/ListMcpServerStatusResponse.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/v2/ConfigRequirementsReadResponse.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/v2/GetAccountRateLimitsResponse.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/v2/PermissionProfileListResponse.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/v2/PluginReadParams.json",
    "apps/desktop/fixtures/codex-schema/0.145.0/v2/PluginReadResponse.json",
    "apps/desktop/package.json",
    "apps/desktop/scripts/validate-dist.mjs",
    "apps/desktop/src/AppCrashBoundary.tsx",
    "apps/desktop/src/AppLoader.tsx",
    "apps/desktop/src/App.tsx",
    "apps/desktop/src/ModelSelectionPanel.tsx",
    "apps/desktop/src/ProjectWorkspace.tsx",
    "apps/desktop/src/ScheduledWorkspace.tsx",
    "apps/desktop/src/FilePreviewWorkspace.tsx",
    "apps/desktop/src/ConversationAttachmentTray.tsx",
    "apps/desktop/src/GitWorkspace.tsx",
    "apps/desktop/src/lib/bridge.ts",
    "apps/desktop/src/lib/auth.ts",
    "apps/desktop/src/lib/codex.ts",
    "apps/desktop/src/lib/integration.ts",
    "apps/desktop/src/lib/modelSelection.ts",
    "apps/desktop/src/lib/conversation.ts",
    "apps/desktop/src/lib/session.ts",
    "apps/desktop/src/lib/project.ts",
    "apps/desktop/src/lib/filePreview.ts",
    "apps/desktop/src/lib/attachment.ts",
    "apps/desktop/src/lib/git.ts",
    "apps/desktop/src-tauri/Cargo.toml",
    "apps/desktop/src-tauri/capabilities/main.json",
    "apps/desktop/src-tauri/tauri.conf.json",
    "apps/desktop/src-tauri/desktop-template.desktop",
    "apps/desktop/src-tauri/metainfo/io.github.codeframe78.QuireForge.metainfo.xml",
    "apps/desktop/src-tauri/src/codex/app_server.rs",
    "apps/desktop/src-tauri/src/codex/integration.rs",
    "apps/desktop/src-tauri/src/codex/integration_control.rs",
    "apps/desktop/src-tauri/src/codex/integration_mutation.rs",
    "apps/desktop/src-tauri/src/codex/integration_service.rs",
    "apps/desktop/src-tauri/src/codex/conversation/lifecycle.rs",
    "apps/desktop/src-tauri/src/codex/auth/mod.rs",
    "apps/desktop/src-tauri/src/codex/auth/types.rs",
    "apps/desktop/src-tauri/src/codex/backend.rs",
    "apps/desktop/src-tauri/src/codex/conversation/mod.rs",
    "apps/desktop/src-tauri/src/codex/conversation/presentation.rs",
    "apps/desktop/src-tauri/src/codex/conversation/types.rs",
    "apps/desktop/src-tauri/src/codex/model_selection.rs",
    "apps/desktop/src-tauri/src/codex/probe.rs",
    "apps/desktop/src-tauri/src/project/identity.rs",
    "apps/desktop/src-tauri/src/project/mod.rs",
    "apps/desktop/src-tauri/src/project/storage.rs",
    "apps/desktop/src-tauri/src/project/types.rs",
    "apps/desktop/src-tauri/src/preview/mod.rs",
    "apps/desktop/src-tauri/src/preview/types.rs",
    "apps/desktop/src-tauri/src/attachment/mod.rs",
    "apps/desktop/src-tauri/src/attachment/types.rs",
    "apps/desktop/src-tauri/src/git/mod.rs",
    "apps/desktop/src-tauri/src/git/types.rs",
    "docs/ARCHITECTURE.md",
    "docs/BUILDING.md",
    "docs/DECISIONS/0025-read-only-scheduled-task-catalog.md",
    "docs/DECISIONS/0026-policy-bounded-next-turn-selection.md",
    "docs/LOCAL-BUILD-PERFORMANCE.md",
    "docs/MILESTONE-FORECASTS.md",
    "docs/MILESTONE_19_HARDENING.md",
    "docs/MILESTONE_20_PACKAGING.md",
    "docs/MILESTONE_21A_PRODUCT_READINESS.md",
    "docs/RELEASING.md",
    "docs/MILESTONE_16A_WEBSITE_RECONCILIATION.md",
    "docs/ROADMAP.md",
    "docs/TESTING.md",
    "docs/THREAT-MODEL.md",
    "docs/WEBSITE.md",
    "docs/WEBUZO-DEPLOYMENT.md",
    "docs/DECISIONS/0007-quireforge-metadata-sqlite.md",
    "docs/DECISIONS/0008-native-conversation-runtime.md",
    "docs/DECISIONS/0011-native-approvals-and-activity-contract.md",
    "docs/DECISIONS/0012-read-only-git-review-boundary.md",
    "docs/DECISIONS/0018-normalized-integration-contracts.md",
    "docs/DECISIONS/0019-confirmed-integration-mutations.md",
    "docs/DECISIONS/0020-confirmed-integration-authorization-and-controls.md",
    "docs/DECISIONS/0021-safe-project-file-previews.md",
    "docs/DECISIONS/0022-bounded-conversation-image-attachments.md",
    "docs/DECISIONS/0023-reviewed-desktop-handoffs-and-notifications.md",
    "docs/DECISIONS/0024-webuzo-static-website-hosting.md",
    "docs/MILESTONE_16B_ORIGIN_STAGING.md",
    "docs/MILESTONE_16C_PRODUCTION_ACTIVATION.md",
    "docs/MILESTONE_16D_AUTOMATIC_SSL.md",
    "scripts/generate_codex_schema_fixtures.py",
    "scripts/fetch_tauri_linux_tools.py",
    "scripts/package_linux.py",
    "scripts/release_contract.py",
    "scripts/run_linux_package_container.sh",
    "scripts/tests/test_package_contract.py",
    "scripts/validate_release_artifacts.py",
    "packaging/linux/Dockerfile",
    "packaging/linux/tauri-tools.json",
    "packaging/release-manifest.schema.json",
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
        "James-Jennison/quireforge",
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
        '"id": "safe-file-previews"',
        '"id": "conversation-attachments"',
    ),
    "apps/desktop/fixtures/file-preview.json": (
        '"schemaVersion": 1',
        '"state": "ready"',
        '"rendering": "normalized-text"',
        '"displayPath": "docs/preview.md"',
        '"diagnosticCode": null',
    ),
    "apps/desktop/fixtures/conversation-attachments.json": (
        '"schemaVersion": 1',
        '"state": "ready"',
        '"source": "drag-drop"',
        '"mimeType": "image/png"',
        '"diagnosticCode": null',
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
        '"schemaVersion": 3',
        '"state": "empty"',
        '"conversationId": null',
        '"modelSelection": null',
        '"events": []',
    ),
    "apps/desktop/src-tauri/src/lib.rs": (
        "codex_runtime_probe",
        "CodexRuntimeService::default()",
        "integration_catalog_read",
        "integration_catalog_refresh",
        "IntegrationCatalogService::default()",
        "integration_control_preview",
        "integration_control_confirm",
        "integration_control_open_browser",
        "integration_control_status",
        "IntegrationControlService::default()",
        "integration_mutation_preview",
        "integration_mutation_confirm",
        "IntegrationMutationService::default()",
        "codex_auth_status",
        "codex_auth_start",
        "codex_auth_cancel",
        "codex_auth_logout",
        "CodexAuthService::default()",
        "project_workspace_status",
        "project_pick_directory",
        "project_preflight",
        "file_preview_pick",
        "FilePreviewService",
        "conversation_attachment_status",
        "conversation_attachment_pick",
        "conversation_attachment_stage_drop",
        "conversation_attachment_cancel",
        "ConversationAttachmentService",
        "ProjectService::open",
        "conversation_status",
        "conversation_start",
        "conversation_poll",
        "conversation_interrupt",
        "conversation_approval_decide",
        "model_selection_update",
        "ConversationService::default()",
        "git_status",
        "git_diff",
        "git_open_file",
        "git_mutation_preview",
        "git_mutation_confirm",
        "git_mutation_recover",
        "GitService",
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
FORBIDDEN_FRONTEND_PATTERNS = (
    re.compile(r"\bdangerouslySetInnerHTML\b"),
    re.compile(r"\b(?:innerHTML|outerHTML)\s*="),
    re.compile(r"\bdocument\.write\s*\("),
    re.compile(r"\beval\s*\("),
    re.compile(r"\bnew\s+Function\s*\("),
    re.compile(r"\bfetch\s*\("),
    re.compile(r"\bXMLHttpRequest\b"),
    re.compile(r"\bWebSocket\s*\("),
)


def repository_files() -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    )
    paths = [ROOT / item.decode() for item in result.stdout.split(b"\0") if item]
    return [path for path in paths if path.exists() or path.is_symlink()]


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
        if (
            relative.startswith("apps/desktop/src/")
            and path.suffix in {".ts", ".tsx"}
            and ".test." not in path.name
            and "/test/" not in relative
        ):
            for pattern in FORBIDDEN_FRONTEND_PATTERNS:
                if pattern.search(text):
                    errors.append(
                        f"forbidden direct frontend capability in {relative}: "
                        f"{pattern.pattern}"
                    )
        if relative.startswith(".github/workflows/") and path.suffix in {
            ".yaml",
            ".yml",
        }:
            for action in re.findall(r"(?m)^\s*uses:\s*([^\s#]+)", text):
                if action.startswith("./"):
                    continue
                reference = action.rsplit("@", 1)[-1] if "@" in action else ""
                if not re.fullmatch(r"[0-9a-f]{40}", reference):
                    errors.append(
                        f"GitHub Action must use a full commit SHA: {relative}: {action}"
                    )
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

    package_path = ROOT / "package.json"
    if package_path.is_file():
        package = json.loads(package_path.read_text(encoding="utf-8"))
        scripts = package.get("scripts", {})
        if scripts.get("security:audit:node") != "pnpm audit --audit-level high":
            errors.append("root package must retain the high-severity Node audit gate")
        if scripts.get("security:audit:rust") != "cargo audit --deny warnings":
            errors.append("root package must retain the warning-denying RustSec audit gate")

    workspace_path = ROOT / "pnpm-workspace.yaml"
    if workspace_path.is_file():
        workspace_text = workspace_path.read_text(encoding="utf-8")
        if not re.search(r"(?m)^overrides:\n  fast-uri: 3\.1\.4$", workspace_text):
            errors.append("workspace must retain the reviewed fast-uri 3.1.4 override")

    audit_config_path = ROOT / ".cargo/audit.toml"
    if audit_config_path.is_file():
        audit_config_text = audit_config_path.read_text(encoding="utf-8")
        expected_advisory_exceptions = {
            "RUSTSEC-2024-0370",
            "RUSTSEC-2024-0411",
            "RUSTSEC-2024-0412",
            "RUSTSEC-2024-0413",
            "RUSTSEC-2024-0414",
            "RUSTSEC-2024-0415",
            "RUSTSEC-2024-0416",
            "RUSTSEC-2024-0417",
            "RUSTSEC-2024-0418",
            "RUSTSEC-2024-0419",
            "RUSTSEC-2024-0420",
            "RUSTSEC-2024-0429",
            "RUSTSEC-2025-0075",
            "RUSTSEC-2025-0080",
            "RUSTSEC-2025-0081",
            "RUSTSEC-2025-0098",
            "RUSTSEC-2025-0100",
        }
        recorded_advisory_exceptions = set(
            re.findall(r'"(RUSTSEC-\d{4}-\d{4})"', audit_config_text)
        )
        if recorded_advisory_exceptions != expected_advisory_exceptions:
            errors.append("RustSec exceptions must match the reviewed Tauri graph")

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
        app_config = tauri_config.get("app", {})
        security = app_config.get("security", {})
        production_csp = security.get("csp", "")
        if not production_csp:
            errors.append("desktop production CSP must be explicit")
        required_csp_directives = (
            "default-src 'none'",
            "script-src 'self'",
            "connect-src ipc: http://ipc.localhost",
            "img-src 'self' data:",
            "style-src 'self' 'unsafe-inline'",
            "object-src 'none'",
            "base-uri 'none'",
            "form-action 'none'",
            "frame-src 'none'",
            "frame-ancestors 'none'",
            "worker-src 'none'",
        )
        for directive in required_csp_directives:
            if directive not in production_csp:
                errors.append(f"desktop production CSP missing directive: {directive}")
        for forbidden_source in ("asset:", "http://asset.localhost", "unsafe-eval"):
            if forbidden_source in production_csp:
                errors.append(
                    f"desktop production CSP must not include {forbidden_source}"
                )
        if app_config.get("withGlobalTauri") is not False:
            errors.append("desktop must not expose the global Tauri API")
        if tauri_config.get("build", {}).get("removeUnusedCommands") is not True:
            errors.append("desktop release builds must prune unused plugin commands")
        if security.get("freezePrototype") is not False:
            errors.append(
                "desktop Object.prototype freezing must remain disabled for the verified frontend bundle"
            )
        if security.get("dangerousDisableAssetCspModification") is not False:
            errors.append("desktop must retain Tauri's asset CSP injection")
        if security.get("assetProtocol") != {"enable": False, "scope": []}:
            errors.append("desktop asset protocol must remain explicitly disabled")
        expected_headers = {
            "Cross-Origin-Opener-Policy": "same-origin",
            "Cross-Origin-Resource-Policy": "same-origin",
            "Permissions-Policy": (
                "camera=(), display-capture=(), geolocation=(), "
                "microphone=(), payment=(), usb=()"
            ),
            "X-Content-Type-Options": "nosniff",
        }
        if security.get("headers") != expected_headers:
            errors.append("desktop security response headers must match the reviewed set")
        bundle = tauri_config.get("bundle", {})
        if bundle.get("active") is not True:
            errors.append("desktop packaging must remain active after Milestone 20")
        if bundle.get("targets") != ["appimage", "deb"]:
            errors.append("desktop package targets must remain AppImage and Debian")
        expected_bundle_metadata = {
            "publisher": "QuireForge contributors",
            "homepage": "https://quireforge.jamesjennison.net",
            "license": "Apache-2.0",
            "licenseFile": "../../../LICENSE",
            "category": "DeveloperTool",
            "shortDescription": "An unofficial native Linux workspace for Codex",
        }
        for field, expected in expected_bundle_metadata.items():
            if bundle.get(field) != expected:
                errors.append(f"desktop bundle {field} must match the release contract")
        linux_bundle = bundle.get("linux", {})
        if linux_bundle.get("deb", {}).get("desktopTemplate") != (
            "desktop-template.desktop"
        ):
            errors.append("Debian bundles must use the reviewed desktop template")
        if linux_bundle.get("appimage", {}).get("files", {}).get(
            "/quireforge.png"
        ) != "icons/icon.png":
            errors.append("AppImage bundles must retain the canonical root icon")

    version_paths = (
        ROOT / "package.json",
        ROOT / "apps/desktop/package.json",
        ROOT / "apps/website/package.json",
    )
    if all(path.is_file() for path in version_paths) and TAURI_VERSION_RE.search(
        (ROOT / "apps/desktop/src-tauri/Cargo.toml").read_text(encoding="utf-8")
    ):
        versions = {
            json.loads(path.read_text(encoding="utf-8")).get("version")
            for path in version_paths
        }
        cargo_version = TAURI_VERSION_RE.search(
            (ROOT / "apps/desktop/src-tauri/Cargo.toml").read_text(
                encoding="utf-8"
            )
        ).group(1)
        versions.add(cargo_version)
        if len(versions) != 1:
            errors.append("root, desktop, website, and Cargo versions must match")

    dockerfile_path = ROOT / "packaging/linux/Dockerfile"
    if dockerfile_path.is_file():
        dockerfile = dockerfile_path.read_text(encoding="utf-8")
        from_lines = re.findall(r"(?m)^FROM\s+(.+)$", dockerfile)
        if len(from_lines) != 3 or any(
            not re.search(r"@sha256:[0-9a-f]{64}(?:\s+AS\s+\w+)?$", line)
            for line in from_lines
        ):
            errors.append("packaging container images must use immutable digests")

    tool_manifest_path = ROOT / "packaging/linux/tauri-tools.json"
    if tool_manifest_path.is_file():
        tool_manifest = json.loads(tool_manifest_path.read_text(encoding="utf-8"))
        tools = tool_manifest.get("tools", [])
        if tool_manifest.get("schemaVersion") != 1 or len(tools) != 6:
            errors.append("Tauri Linux tool manifest must contain the reviewed set")
        for tool in tools:
            url = tool.get("url", "")
            digest = tool.get("sha256", "")
            if (
                not isinstance(url, str)
                or not url.startswith("https://")
                or re.search(r"https://[^/]*@", url)
                or not re.fullmatch(r"[0-9a-f]{64}", str(digest))
            ):
                errors.append("Tauri Linux tool source or checksum is invalid")

    release_workflow_path = ROOT / ".github/workflows/linux-release.yml"
    if release_workflow_path.is_file():
        release_workflow = release_workflow_path.read_text(encoding="utf-8")
        required_release_controls = (
            "workflow_dispatch:",
            "permissions:\n  contents: read",
            "inputs.operation == 'publish-approved-beta'",
            "inputs.publication-confirmation == 'PUBLISH-QUIREFORGE-BETA'",
            "startsWith(github.ref, 'refs/tags/v')",
            "environment: quireforge-release",
            "attestations: write",
            "--require-publishable",
            "--verify-tag",
        )
        for control in required_release_controls:
            if control not in release_workflow:
                errors.append(f"Linux release workflow is missing guard: {control}")
        if "pull_request_target:" in release_workflow:
            errors.append("Linux release workflow must never use pull_request_target")

    downloads_path = ROOT / "apps/website/src/data/downloads.ts"
    if downloads_path.is_file():
        downloads = downloads_path.read_text(encoding="utf-8")
        required_inactive_downloads = (
            'state: "unavailable"',
            "release: null",
            'plannedFormats: ["appimage", "deb"]',
        )
        for marker in required_inactive_downloads:
            if marker not in downloads:
                errors.append(f"website download data must remain inactive: {marker}")

    schema_root = ROOT / "apps/desktop/fixtures/codex-schema/0.144.6"
    manifest_path = schema_root / "manifest.json"
    if manifest_path.is_file():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        expected_schema_paths = {
            "RequestId.json",
            "CommandExecutionRequestApprovalParams.json",
            "CommandExecutionRequestApprovalResponse.json",
            "FileChangeRequestApprovalParams.json",
            "FileChangeRequestApprovalResponse.json",
            "PermissionsRequestApprovalParams.json",
            "PermissionsRequestApprovalResponse.json",
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
            "v2/ThreadListParams.json",
            "v2/ThreadListResponse.json",
            "v2/ThreadReadParams.json",
            "v2/ThreadReadResponse.json",
            "v2/ThreadResumeParams.json",
            "v2/ThreadResumeResponse.json",
            "v2/ThreadForkParams.json",
            "v2/ThreadForkResponse.json",
            "v2/ThreadArchiveParams.json",
            "v2/ThreadArchiveResponse.json",
            "v2/ThreadUnarchiveParams.json",
            "v2/ThreadUnarchiveResponse.json",
            "v2/TurnStartParams.json",
            "v2/TurnStartResponse.json",
            "v2/TurnInterruptParams.json",
            "v2/TurnInterruptResponse.json",
            "v2/ThreadStartedNotification.json",
            "v2/ThreadArchivedNotification.json",
            "v2/ThreadUnarchivedNotification.json",
            "v2/TurnStartedNotification.json",
            "v2/TurnCompletedNotification.json",
            "v2/AgentMessageDeltaNotification.json",
            "v2/ReasoningSummaryTextDeltaNotification.json",
            "v2/TurnPlanUpdatedNotification.json",
            "v2/ItemStartedNotification.json",
            "v2/ItemCompletedNotification.json",
            "v2/CommandExecutionOutputDeltaNotification.json",
            "v2/McpToolCallProgressNotification.json",
            "v2/ServerRequestResolvedNotification.json",
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

    generator_path = ROOT / "scripts/generate_codex_schema_fixtures.py"
    current_schema_root = ROOT / "apps/desktop/fixtures/codex-schema/0.145.0"
    current_manifest_path = current_schema_root / "manifest.json"
    if generator_path.is_file() and current_manifest_path.is_file():
        generator_tree = ast.parse(generator_path.read_text(encoding="utf-8"))
        selection_node = next(
            (
                node.value
                for node in generator_tree.body
                if isinstance(node, ast.Assign)
                and any(
                    isinstance(target, ast.Name)
                    and target.id == "SELECTED_SCHEMAS"
                    for target in node.targets
                )
            ),
            None,
        )
        if selection_node is None:
            errors.append("Codex schema generator must define SELECTED_SCHEMAS")
            current_expected_paths: set[str] = set()
        else:
            current_expected_paths = set(ast.literal_eval(selection_node))

        current_manifest = json.loads(
            current_manifest_path.read_text(encoding="utf-8")
        )
        current_manifest_files = current_manifest.get("files", [])
        current_recorded_paths = {
            entry.get("path")
            for entry in current_manifest_files
            if isinstance(entry, dict)
        }
        if current_manifest.get("codexCliVersion") != "0.145.0":
            errors.append("Current Codex schema manifest must record CLI 0.145.0")
        if current_recorded_paths != current_expected_paths:
            errors.append(
                "Current Codex schema manifest must match the reviewed generator subset"
            )

        for entry in current_manifest_files:
            if not isinstance(entry, dict):
                errors.append("Current Codex schema manifest contains a malformed entry")
                continue
            relative = entry.get("path")
            digest = entry.get("sha256")
            if not isinstance(relative, str) or relative not in current_expected_paths:
                continue
            current_schema_path = current_schema_root / relative
            if not current_schema_path.is_file():
                errors.append(f"Current Codex schema is missing: {relative}")
                continue
            actual = hashlib.sha256(current_schema_path.read_bytes()).hexdigest()
            if digest != actual:
                errors.append(f"Current Codex schema hash mismatch: {relative}")

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

    integration_fixture_path = ROOT / "apps/desktop/fixtures/integration-catalog.json"
    if integration_fixture_path.is_file():
        integration_fixture = json.loads(
            integration_fixture_path.read_text(encoding="utf-8")
        )
        serialized_fixture = json.dumps(integration_fixture)
        for forbidden_field in (
            "accountId",
            "authorizationUrl",
            "accessToken",
            "apiKey",
            "rawProtocolPayload",
            "arguments",
            "threadId",
            "turnId",
            "requestId",
            "marketplacePath",
        ):
            if forbidden_field in serialized_fixture:
                errors.append(
                    f"Integration fixture contains raw field: {forbidden_field}"
                )

    integration_mutation_fixture_path = (
        ROOT / "apps/desktop/fixtures/integration-mutation.json"
    )
    if integration_mutation_fixture_path.is_file():
        integration_mutation_fixture = json.loads(
            integration_mutation_fixture_path.read_text(encoding="utf-8")
        )
        serialized_fixture = json.dumps(integration_mutation_fixture)
        for forbidden_field in (
            "accountId",
            "authorizationUrl",
            "accessToken",
            "apiKey",
            "rawProtocolPayload",
            "arguments",
            "sourcePath",
            "sourceUrl",
            "installedPath",
            "marketplaceRoot",
        ):
            if forbidden_field in serialized_fixture:
                errors.append(
                    "Integration mutation fixture contains raw field: "
                    f"{forbidden_field}"
                )

    integration_control_fixture_path = (
        ROOT / "apps/desktop/fixtures/integration-control.json"
    )
    if integration_control_fixture_path.is_file():
        integration_control_fixture = json.loads(
            integration_control_fixture_path.read_text(encoding="utf-8")
        )
        serialized_fixture = json.dumps(integration_control_fixture)
        for forbidden_field in (
            "accountId",
            "authorizationUrl",
            "accessToken",
            "apiKey",
            "rawProtocolPayload",
            "arguments",
            "sourcePath",
            "skillPath",
            "appPath",
            "mcpServerName",
            "threadId",
            "turnId",
        ):
            if forbidden_field in serialized_fixture:
                errors.append(
                    "Integration control fixture contains raw field: "
                    f"{forbidden_field}"
                )

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
            "requestId",
            "itemId",
            "arguments",
            "diff",
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
