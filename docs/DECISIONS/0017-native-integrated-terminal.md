# ADR 0017: Own Integrated Terminals Through a Native PTY Boundary

- Status: Accepted and implemented in Milestone 12
- Date: 2026-07-20
- Decision owners: Project maintainers

## Context

QuireForge needs project-rooted interactive terminals without scraping the
Codex TUI, routing terminal traffic through Codex approval requests, or letting
the React webview select a working directory, executable, environment, process,
or operating-system signal target. A terminal must preserve byte-oriented PTY
output, handle resize and concurrent input, and reliably stop foreground and
background jobs when its app-owned tab is closed.

GUI-launched Linux applications also cannot assume that their inherited
`PATH`, shell, or environment matches an interactive login. Terminal startup
therefore needs an explicit environment contract and a freshly revalidated
project directory.

## Decision

- Use `portable-pty` behind a QuireForge-owned `TerminalService`. It supplies
  the native PTY pair, controlling terminal, session leader, cloned reader,
  writer, resize operation, and child wait/termination handles.
- Use stable scoped `@xterm/xterm` and `@xterm/addon-fit` packages in React.
  Do not install the web-links or WebGL addons, enable proposed APIs, or make
  terminal-generated links actionable.
- Key every terminal by an app-owned UUIDv7. React may submit only that ID,
  bounded input bytes, a bounded output cursor, and validated row/column
  dimensions after initial project selection.
- Start the shell only after `ProjectService` reserves and revalidates the
  attached directory. No cwd fallback is allowed. Native shell selection uses
  the current user's executable login shell with `/bin/sh` as the library's
  documented fallback; React cannot choose or observe the executable.
- Clear the inherited child environment, add a fixed system `PATH`, preserve a
  narrow set of desktop/session variables, and set QuireForge's terminal
  identity explicitly. Credential-shaped variables and QuireForge/Codex
  process configuration are not inherited. User login files may intentionally
  extend the shell environment after startup.
- Keep at most eight live or recoverable tabs. Retain at most one MiB of
  byte-preserving output per live terminal and return bounded base64 chunks
  through strict snapshots. Output loss is explicit when a slow consumer falls
  behind the retained window.
- On Linux, close the PTY session with a bounded HUP/TERM/KILL sequence while
  the unreaped session-leader handle prevents PID reuse. Inspect only numeric
  `/proc` stat identity needed to find members of the owned session; never read
  their arguments or environment.
- Persist only tab title, project reference, state, dimensions, exit code, and
  timestamps. Never persist terminal input, output, scrollback, shell history,
  cwd, environment, TTY name, PID, process-group ID, or session ID. Mark live
  metadata interrupted at application restart because process ownership cannot
  be recovered safely.
- Keep integrated terminals separate from Codex approvals. Terminal input runs
  with the user's account privileges and the UI must state that boundary.

## Consequences

- Terminal emulation stays in a mature renderer while process and filesystem
  authority remain native.
- Closing a terminal is deliberately stronger than closing a visual panel: it
  ends the owned session and removes its recoverable metadata.
- Shell startup remains useful on desktop launches without broadly inheriting
  credential-bearing process state.
- A deliberately daemonized process that creates a new session has left the
  terminal's ownership boundary and cannot be represented as an attached tab.
  Tests cover normal foreground and background jobs within the owned session.
- No GPU renderer is required. The DOM renderer and ordinary Rust/TypeScript
  builds remain CPU- and system-memory workloads.
