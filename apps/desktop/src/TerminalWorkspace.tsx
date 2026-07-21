import { useEffect, useMemo, useRef, useState } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";

import type { ProjectWorkspaceSnapshot } from "./lib/project";
import type {
  TerminalCloseRequest,
  TerminalPollRequest,
  TerminalRegistrySnapshot,
  TerminalResizeRequest,
  TerminalSnapshot,
  TerminalStartRequest,
  TerminalWriteRequest,
} from "./lib/terminal";

type TerminalAvailability = "checking" | "native" | "preview";

interface TerminalWorkspaceProps {
  availability: TerminalAvailability;
  registry: TerminalRegistrySnapshot;
  projects: ProjectWorkspaceSnapshot;
  busy: boolean;
  actionError: boolean;
  onStart: (request: TerminalStartRequest) => Promise<TerminalSnapshot>;
  onPoll: (request: TerminalPollRequest) => Promise<TerminalSnapshot>;
  onWrite: (request: TerminalWriteRequest) => Promise<TerminalSnapshot>;
  onResize: (request: TerminalResizeRequest) => Promise<TerminalSnapshot>;
  onClose: (request: TerminalCloseRequest) => Promise<TerminalRegistrySnapshot>;
  onSnapshot: (snapshot: TerminalSnapshot) => void;
}

const diagnosticMessages: Record<
  NonNullable<TerminalSnapshot["diagnosticCode"]>,
  string
> = {
  "invalid-request": "The terminal request was outside its supported bounds.",
  "capacity-reached": "Close a terminal before opening another one.",
  "terminal-not-found": "That terminal is no longer owned by this app process.",
  "project-unavailable": "The selected project is no longer available.",
  "project-identity-changed":
    "The project directory changed identity. Reverify it before starting a terminal.",
  "project-not-writable": "The selected project is not writable.",
  "project-busy":
    "That project has active controlled work. Finish it before starting a terminal.",
  "metadata-unavailable": "Terminal metadata is unavailable.",
  "pty-unavailable": "Linux could not create the terminal device.",
  "shell-unavailable": "The current user shell could not be started.",
  "input-too-large": "The terminal input exceeded its bounded request size.",
  "input-unavailable": "The terminal stopped accepting input.",
  "resize-unavailable": "The terminal could not apply that window size.",
  "output-unavailable": "Terminal output stopped unexpectedly.",
  "cleanup-incomplete":
    "QuireForge could not prove that every owned terminal process stopped.",
};

const stateLabels: Record<TerminalSnapshot["state"], string> = {
  running: "Running",
  closing: "Closing",
  exited: "Exited",
  interrupted: "Interrupted",
  failed: "Failed",
  unavailable: "Unavailable",
};

function bytesToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (let index = 0; index < bytes.length; index += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(index, index + 0x8000));
  }
  return btoa(binary);
}

function base64ToBytes(value: string): Uint8Array {
  const binary = atob(value);
  return Uint8Array.from(binary, (character) => character.charCodeAt(0));
}

function writeRenderedOutput(
  terminal: Terminal,
  output: Uint8Array,
): Promise<void> {
  return new Promise((resolve) => terminal.write(output, resolve));
}

interface TerminalPaneProps {
  snapshot: TerminalSnapshot;
  visible: boolean;
  onPoll: TerminalWorkspaceProps["onPoll"];
  onWrite: TerminalWorkspaceProps["onWrite"];
  onResize: TerminalWorkspaceProps["onResize"];
  onSnapshot: TerminalWorkspaceProps["onSnapshot"];
}

function TerminalPane({
  snapshot,
  visible,
  onPoll,
  onWrite,
  onResize,
  onSnapshot,
}: TerminalPaneProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const snapshotRef = useRef(snapshot);
  const cursorRef = useRef(0);
  const writeQueueRef = useRef(Promise.resolve());
  const callbacksRef = useRef({ onPoll, onWrite, onResize, onSnapshot });

  useEffect(() => {
    snapshotRef.current = snapshot;
    callbacksRef.current = { onPoll, onWrite, onResize, onSnapshot };
  }, [onPoll, onResize, onSnapshot, onWrite, snapshot]);

  useEffect(() => {
    const host = hostRef.current;
    if (!host || !snapshot.terminalId) return;
    const terminal = new Terminal({
      allowProposedApi: false,
      convertEol: false,
      cursorBlink: true,
      cursorStyle: "bar",
      disableStdin: snapshotRef.current.state !== "running",
      fontFamily: '"JetBrains Mono", "Cascadia Code", "Ubuntu Mono", monospace',
      fontSize: 13,
      lineHeight: 1.2,
      screenReaderMode: true,
      scrollback: 5_000,
      theme: {
        background: "#11100e",
        foreground: "#f3efe5",
        cursor: "#e8a85d",
        selectionBackground: "#7e5b3566",
      },
    });
    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);
    terminal.open(host);
    terminalRef.current = terminal;

    const inputDisposable = terminal.onData((data) => {
      if (snapshotRef.current.state !== "running") return;
      const terminalId = snapshotRef.current.terminalId;
      if (!terminalId) return;
      const encoded = new TextEncoder().encode(data);
      for (let offset = 0; offset < encoded.length; offset += 48 * 1024) {
        const chunk = encoded.subarray(offset, offset + 48 * 1024);
        writeQueueRef.current = writeQueueRef.current
          .then(() =>
            callbacksRef.current.onWrite({
              terminalId,
              dataBase64: bytesToBase64(chunk),
            }),
          )
          .then((result) => {
            snapshotRef.current = result;
            callbacksRef.current.onSnapshot({ ...result, output: [] });
          })
          .catch(() => undefined);
      }
    });
    const resizeDisposable = terminal.onResize(({ cols, rows }) => {
      const current = snapshotRef.current;
      if (
        current.state !== "running" ||
        !current.terminalId ||
        (current.columns === cols && current.rows === rows)
      )
        return;
      void callbacksRef.current
        .onResize({ terminalId: current.terminalId, columns: cols, rows })
        .then((result) => {
          snapshotRef.current = result;
          callbacksRef.current.onSnapshot({ ...result, output: [] });
        })
        .catch(() => undefined);
    });
    const fit = () => {
      try {
        fitAddon.fit();
      } catch {
        // The native resize contract remains unchanged if layout is unavailable.
      }
    };
    const resizeObserver =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(() => fit());
    resizeObserver?.observe(host);
    window.requestAnimationFrame(fit);
    terminal.focus();

    return () => {
      resizeObserver?.disconnect();
      inputDisposable.dispose();
      resizeDisposable.dispose();
      terminal.dispose();
      terminalRef.current = null;
    };
  }, [snapshot.terminalId]);

  useEffect(() => {
    if (!snapshot.live || !snapshot.terminalId) return;
    let cancelled = false;
    let timer: number | undefined;
    const poll = async () => {
      if (cancelled) return;
      try {
        const result = await callbacksRef.current.onPoll({
          terminalId: snapshot.terminalId!,
          afterSequence: cursorRef.current,
        });
        if (cancelled) return;
        const terminal = terminalRef.current;
        if (terminal && result.truncated) {
          await writeRenderedOutput(
            terminal,
            new TextEncoder().encode(
              "\r\n[QuireForge: earlier output exceeded the retained window]\r\n",
            ),
          );
        }
        if (terminal) {
          for (const chunk of result.output) {
            await writeRenderedOutput(
              terminal,
              base64ToBytes(chunk.dataBase64),
            );
            cursorRef.current = chunk.sequence;
          }
        }
        snapshotRef.current = result;
        callbacksRef.current.onSnapshot({ ...result, output: [] });
        const settled = !["running", "closing"].includes(result.state);
        timer = window.setTimeout(
          poll,
          result.hasMore ? 0 : settled ? 1_000 : visible ? 90 : 350,
        );
      } catch {
        timer = window.setTimeout(poll, visible ? 500 : 1_000);
      }
    };
    void poll();
    return () => {
      cancelled = true;
      if (timer !== undefined) window.clearTimeout(timer);
    };
  }, [snapshot.live, snapshot.terminalId, visible]);

  useEffect(() => {
    if (visible) {
      window.requestAnimationFrame(() => terminalRef.current?.focus());
    }
  }, [visible]);

  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.options.disableStdin = snapshot.state !== "running";
    }
  }, [snapshot.state]);

  return (
    <div
      className="terminal-pane"
      hidden={!visible}
      role="tabpanel"
      aria-label={`${snapshot.title ?? "Terminal"} output`}
    >
      <div className="terminal-pane__viewport" ref={hostRef} />
    </div>
  );
}

export function TerminalWorkspace({
  availability,
  registry,
  projects,
  busy,
  actionError,
  onStart,
  onPoll,
  onWrite,
  onResize,
  onClose,
  onSnapshot,
}: TerminalWorkspaceProps) {
  const availableProjects = useMemo(
    () =>
      projects.projects.filter(
        (project) =>
          !project.archived &&
          project.directory?.state === "connected-accessible",
      ),
    [projects.projects],
  );
  const [requestedProjectId, setRequestedProjectId] = useState("");
  const projectId = availableProjects.some(
    (project) => project.id === requestedProjectId,
  )
    ? requestedProjectId
    : (availableProjects[0]?.id ?? "");
  const [requestedTerminalId, setRequestedTerminalId] = useState<string | null>(
    null,
  );
  const activeTerminalId = registry.terminals.some(
    (terminal) => terminal.terminalId === requestedTerminalId,
  )
    ? requestedTerminalId
    : (registry.terminals[0]?.terminalId ?? null);
  const [confirmCloseId, setConfirmCloseId] = useState<string | null>(null);
  const nativeReady = availability === "native";

  async function start() {
    if (!projectId) return;
    try {
      const result = await onStart({ projectId, columns: 100, rows: 30 });
      if (result.terminalId) setRequestedTerminalId(result.terminalId);
    } catch {
      // The app-level error state presents the bounded failure.
    }
  }

  async function close(terminalId: string) {
    try {
      const result = await onClose({ terminalId });
      setConfirmCloseId(null);
      setRequestedTerminalId(result.terminals[0]?.terminalId ?? null);
    } catch {
      // Keep the review open so the user can inspect or retry safely.
    }
  }

  const selectedTerminal = registry.terminals.find(
    (terminal) => terminal.terminalId === activeTerminalId,
  );

  return (
    <section
      className="integrated-terminal"
      id="terminal"
      aria-labelledby="terminal-title"
    >
      <div className="integrated-terminal__heading">
        <div>
          <p className="eyebrow">Integrated terminal</p>
          <h2 id="terminal-title">A real shell, rooted where you work.</h2>
          <p>
            Every tab starts in a freshly verified attached directory. Terminal
            input runs directly with your Linux account privileges and is
            separate from Codex approvals.
          </p>
        </div>
        <div className="terminal-launch">
          <label htmlFor="terminal-project">Project</label>
          <select
            id="terminal-project"
            value={projectId}
            disabled={!nativeReady || busy || availableProjects.length === 0}
            onChange={(event) => setRequestedProjectId(event.target.value)}
          >
            {availableProjects.length === 0 && (
              <option value="">No writable project</option>
            )}
            {availableProjects.map((project) => (
              <option value={project.id} key={project.id}>
                {project.displayName}
              </option>
            ))}
          </select>
          <button
            className="auth-button auth-button--primary"
            type="button"
            disabled={
              !nativeReady ||
              busy ||
              !projectId ||
              registry.terminals.length >= registry.capacity
            }
            onClick={() => void start()}
          >
            New terminal
          </button>
        </div>
      </div>

      <div className="terminal-status" aria-live="polite">
        {availability === "checking" && <p>Reading terminal metadata.</p>}
        {availability === "preview" && (
          <p>Browser preview cannot start or simulate a native Linux shell.</p>
        )}
        {registry.diagnosticCode && (
          <p className="project-message project-message--warning" role="alert">
            Terminal metadata is unavailable.
          </p>
        )}
        {actionError && (
          <p className="project-message project-message--warning" role="alert">
            The terminal action did not complete. Review the terminal state
            before retrying.
          </p>
        )}
        {selectedTerminal?.diagnosticCode && (
          <p className="project-message project-message--warning" role="alert">
            {diagnosticMessages[selectedTerminal.diagnosticCode]}
          </p>
        )}
      </div>

      {registry.terminals.length === 0 ? (
        <div className="terminal-empty">
          <span aria-hidden="true">›_</span>
          <div>
            <h3>No terminal open</h3>
            <p>
              Select a writable attached project and open a bounded native PTY
              session. No shell history or output is stored by QuireForge.
            </p>
          </div>
        </div>
      ) : (
        <div className="terminal-surface">
          <div className="terminal-tabs" role="tablist" aria-label="Terminals">
            {registry.terminals.map((terminal) => {
              const terminalId = terminal.terminalId!;
              return (
                <div className="terminal-tab" key={terminalId}>
                  <button
                    type="button"
                    role="tab"
                    aria-selected={terminalId === activeTerminalId}
                    className={
                      terminalId === activeTerminalId
                        ? "terminal-tab__select terminal-tab__select--active"
                        : "terminal-tab__select"
                    }
                    onClick={() => setRequestedTerminalId(terminalId)}
                  >
                    <span>{terminal.title}</span>
                    <em>{stateLabels[terminal.state]}</em>
                  </button>
                  <button
                    type="button"
                    className="terminal-tab__close"
                    aria-label={`Close ${terminal.title}`}
                    disabled={busy}
                    onClick={() => setConfirmCloseId(terminalId)}
                  >
                    ×
                  </button>
                </div>
              );
            })}
          </div>

          <div className="terminal-panels">
            {registry.terminals.map((terminal) => {
              const terminalId = terminal.terminalId!;
              if (!terminal.live) {
                return (
                  <div
                    className="terminal-recovered"
                    role="tabpanel"
                    hidden={terminalId !== activeTerminalId}
                    key={terminalId}
                  >
                    <strong>{stateLabels[terminal.state]} terminal</strong>
                    <p>
                      Process ownership ended with the previous app session.
                      QuireForge does not persist terminal output or shell
                      history.
                    </p>
                  </div>
                );
              }
              return (
                <TerminalPane
                  snapshot={terminal}
                  visible={terminalId === activeTerminalId}
                  onPoll={onPoll}
                  onWrite={onWrite}
                  onResize={onResize}
                  onSnapshot={onSnapshot}
                  key={terminalId}
                />
              );
            })}
          </div>
        </div>
      )}

      {confirmCloseId && (
        <div
          className="terminal-close-review"
          role="alertdialog"
          aria-modal="true"
        >
          <div>
            <strong>
              Close{" "}
              {registry.terminals.find(
                (terminal) => terminal.terminalId === confirmCloseId,
              )?.title ?? "terminal"}
              ?
            </strong>
            <p>
              This ends the shell and its owned foreground and background jobs.
              It does not delete project files.
            </p>
          </div>
          <div className="project-actions">
            <button
              className="auth-button auth-button--danger"
              type="button"
              disabled={busy}
              onClick={() => void close(confirmCloseId)}
            >
              End processes and close
            </button>
            <button
              className="auth-button"
              type="button"
              disabled={busy}
              onClick={() => setConfirmCloseId(null)}
            >
              Keep terminal
            </button>
          </div>
        </div>
      )}
    </section>
  );
}
