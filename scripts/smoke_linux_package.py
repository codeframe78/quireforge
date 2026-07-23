#!/usr/bin/env python3
"""Prove that a packaged QuireForge executable opens a visible X11 window."""

from __future__ import annotations

import argparse
import os
import re
import signal
import shutil
import subprocess
import tempfile
import time
from pathlib import Path


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--label", required=True)
    parser.add_argument("--screenshot", type=Path)
    parser.add_argument("--settle-seconds", type=float, default=2)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    arguments = parser.parse_args()
    if not arguments.command:
        parser.error("a packaged executable command is required")
    return arguments


def stop(process: subprocess.Popen[str]) -> None:
    if process.poll() is not None:
        return
    os.killpg(process.pid, signal.SIGTERM)
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=5)


def find_window() -> str | None:
    if shutil.which("xdotool"):
        probe = subprocess.run(
            [
                "xdotool",
                "search",
                "--onlyvisible",
                "--name",
                "^QuireForge$",
            ],
            check=False,
            capture_output=True,
            text=True,
        )
        if probe.returncode == 0 and probe.stdout.strip():
            return probe.stdout.splitlines()[0]
        return None
    if shutil.which("xwininfo"):
        probe = subprocess.run(
            ["xwininfo", "-root", "-tree"],
            check=False,
            capture_output=True,
            text=True,
        )
        match = re.search(
            r'^\s*(0x[0-9a-f]+)\s+\"QuireForge\"',
            probe.stdout,
            flags=re.MULTILINE,
        )
        return match.group(1) if match else None
    raise RuntimeError("xdotool or xwininfo is required for package launch smoke")


def capture_screenshot(window: str, destination: Path) -> None:
    if not shutil.which("import") or not shutil.which("identify"):
        raise RuntimeError("ImageMagick import and identify are required for a screenshot")
    destination.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["import", "-silent", "-window", window, str(destination)],
        check=True,
    )
    colors = subprocess.run(
        ["identify", "-format", "%k", str(destination)],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    if not colors.isdigit() or int(colors) < 8:
        raise RuntimeError(
            f"{destination} contains only {colors or 'an unknown number of'} colors"
        )


def main() -> int:
    arguments = parse_arguments()
    with tempfile.TemporaryDirectory(prefix="quireforge-package-smoke-") as temporary:
        root = Path(temporary)
        environment = os.environ.copy()
        environment.update(
            {
                "GDK_BACKEND": "x11",
                "HOME": str(root / "home"),
                "XDG_CACHE_HOME": str(root / "cache"),
                "XDG_CONFIG_HOME": str(root / "config"),
                "XDG_DATA_HOME": str(root / "data"),
                "XDG_STATE_HOME": str(root / "state"),
            }
        )
        for name in ("home", "cache", "config", "data", "state"):
            (root / name).mkdir()
        log_path = root / "launch.log"
        with log_path.open("w+", encoding="utf-8") as log:
            process = subprocess.Popen(
                arguments.command,
                env=environment,
                start_new_session=True,
                stdout=log,
                stderr=subprocess.STDOUT,
                text=True,
            )
            window_found = False
            window_id = None
            try:
                deadline = time.monotonic() + 20
                while time.monotonic() < deadline:
                    status = process.poll()
                    if status is not None:
                        break
                    window_id = find_window()
                    if window_id:
                        window_found = True
                        time.sleep(arguments.settle_seconds)
                        if process.poll() is not None:
                            window_found = False
                        elif arguments.screenshot:
                            capture_screenshot(window_id, arguments.screenshot.resolve())
                        break
                    time.sleep(0.25)
            finally:
                stop(process)
            log.seek(0)
            output = log.read()

        lowered = output.lower()
        forbidden = ("127.0.0.1", "connection refused")
        if any(item in lowered for item in forbidden):
            raise RuntimeError(
                f"{arguments.label} launch attempted a refused loopback connection"
            )
        if not window_found:
            rendered = output[-4000:] if output else "(no launch output)"
            raise RuntimeError(
                f"{arguments.label} did not open a stable visible window:\n{rendered}"
            )
        print(f"visible package launch passed: {arguments.label}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
