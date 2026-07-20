#!/usr/bin/env python3
"""Generate the small, reviewed Codex app-server schema subset QuireForge uses."""

from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
OUTPUT_ROOT = ROOT / "apps/desktop/fixtures/codex-schema"
SELECTED_SCHEMAS = (
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
    "v2/ErrorNotification.json",
)


def codex_version() -> str:
    result = subprocess.run(
        ["codex", "--version"],
        check=True,
        capture_output=True,
        text=True,
        timeout=10,
    )
    prefix = "codex-cli "
    value = result.stdout.strip()
    if not value.startswith(prefix):
        raise RuntimeError("Codex CLI returned an unexpected version shape")
    version = value.removeprefix(prefix)
    core = version.split("-", maxsplit=1)[0].split("+", maxsplit=1)[0]
    segments = core.split(".")
    if (
        not version
        or len(segments) != 3
        or any(not segment.isdigit() for segment in segments)
        or any(
            character
            not in "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-+"
            for character in version
        )
    ):
        raise RuntimeError("Codex CLI version contains unsupported characters")
    return version


def main() -> None:
    version = codex_version()
    destination = OUTPUT_ROOT / version
    destination.resolve().relative_to(OUTPUT_ROOT.resolve())

    with tempfile.TemporaryDirectory(prefix="quireforge-codex-schema-") as temp:
        generated = Path(temp)
        subprocess.run(
            [
                "codex",
                "app-server",
                "generate-json-schema",
                "--experimental",
                "--out",
                str(generated),
            ],
            check=True,
            timeout=30,
        )

        manifest_files: list[dict[str, str]] = []
        for relative in SELECTED_SCHEMAS:
            source = generated / relative
            if not source.is_file():
                raise RuntimeError(f"Codex schema bundle is missing {relative}")

            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(
                source.read_text(encoding="utf-8").rstrip("\n") + "\n",
                encoding="utf-8",
            )
            digest = hashlib.sha256(target.read_bytes()).hexdigest()
            manifest_files.append({"path": relative, "sha256": digest})

        manifest = {
            "schemaVersion": 1,
            "codexCliVersion": version,
            "generator": "codex app-server generate-json-schema --experimental",
            "selection": (
                "initialize, model/list, stable account lifecycle, and the "
                "Milestone 7 conversation runtime and Milestone 8A reviewed "
                "thread lifecycle subset only"
            ),
            "files": manifest_files,
        }
        destination.mkdir(parents=True, exist_ok=True)
        (destination / "manifest.json").write_text(
            json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
        )

    print(f"Generated {len(SELECTED_SCHEMAS)} reviewed schemas for Codex {version}.")


if __name__ == "__main__":
    main()
