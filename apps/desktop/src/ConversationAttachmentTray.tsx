import { useState, type DragEvent } from "react";

import type {
  ConversationAttachmentDropRequest,
  ConversationAttachmentSnapshot,
} from "./lib/attachment";

interface ConversationAttachmentTrayProps {
  availability: "checking" | "native" | "preview";
  projectId: string | null;
  snapshot: ConversationAttachmentSnapshot;
  busy: boolean;
  disabled: boolean;
  actionError: boolean;
  onPick: (projectId: string) => Promise<void>;
  onDrop: (request: ConversationAttachmentDropRequest) => Promise<void>;
  onCancel: (projectId: string, attachmentId: string) => Promise<void>;
}

const diagnostics: Record<
  NonNullable<ConversationAttachmentSnapshot["diagnosticCode"]>,
  string
> = {
  "invalid-request": "The attachment request was invalid.",
  "project-not-found": "The selected project is no longer available.",
  "project-unavailable": "The selected project directory is unavailable.",
  "project-identity-changed":
    "The project directory identity changed. Relink it before attaching images.",
  "project-not-writable":
    "Conversation attachments require a writable verified project.",
  "staging-unavailable": "Private attachment staging is unavailable.",
  "too-many-files": "Attach at most four images to one turn.",
  "file-too-large": "Each attachment must be 4 MiB or smaller.",
  "unsupported-type": "Only static PNG and JPEG images are supported.",
  "invalid-content": "An attachment did not match its declared image format.",
  "unsafe-name": "An attachment name could not be represented safely.",
  "read-failed": "An attachment could not be read safely.",
  "attachment-not-found": "That staged attachment is no longer available.",
  "attachment-expired": "The attachment draft expired. Add the image again.",
  "cleanup-failed": "Private attachment cleanup could not be verified.",
};

function formatBytes(value: number): string {
  if (value < 1024) return `${value} B`;
  return `${(value / 1024).toFixed(1)} KiB`;
}

function bytesToBase64(bytes: Uint8Array): string {
  const chunks: string[] = [];
  const chunkSize = 32 * 1024;
  for (let index = 0; index < bytes.length; index += chunkSize) {
    chunks.push(
      String.fromCharCode(...bytes.subarray(index, index + chunkSize)),
    );
  }
  return window.btoa(chunks.join(""));
}

export function ConversationAttachmentTray({
  availability,
  projectId,
  snapshot,
  busy,
  disabled,
  actionError,
  onPick,
  onDrop,
  onCancel,
}: ConversationAttachmentTrayProps) {
  const [dragActive, setDragActive] = useState(false);
  const [dropError, setDropError] = useState<string | null>(null);
  const enabled =
    availability === "native" && projectId !== null && !busy && !disabled;
  const attachments =
    snapshot.projectId === projectId && snapshot.state === "ready"
      ? snapshot.attachments
      : [];

  async function stageDrop(event: DragEvent<HTMLDivElement>) {
    event.preventDefault();
    setDragActive(false);
    setDropError(null);
    if (!enabled || !projectId) return;
    const files = [...event.dataTransfer.files];
    if (files.length === 0) {
      try {
        await onDrop({ projectId, files: [] });
      } catch {
        // App state owns the native-capture failure message.
      }
      return;
    }
    if (files.length + attachments.length > 4) {
      setDropError("Drop one to four images; a turn can hold four total.");
      return;
    }
    if (
      files.some(
        (file) =>
          !["image/png", "image/jpeg"].includes(file.type) ||
          file.size === 0 ||
          file.size > 4 * 1024 * 1024,
      )
    ) {
      setDropError("Only static PNG/JPEG images up to 4 MiB can be dropped.");
      return;
    }
    try {
      await onDrop({
        projectId,
        files: await Promise.all(
          files.map(async (file) => ({
            displayName: file.name,
            declaredMimeType: file.type as "image/png" | "image/jpeg",
            base64Data: bytesToBase64(new Uint8Array(await file.arrayBuffer())),
          })),
        ),
      });
    } catch {
      // App state owns the bounded bridge failure message.
    }
  }

  return (
    <div className="conversation-attachments">
      <div className="conversation-attachments__heading">
        <div>
          <strong>Images for this turn</strong>
          <span>Private draft · maximum 4 · 4 MiB each</span>
        </div>
        <button
          type="button"
          disabled={!enabled || attachments.length >= 4}
          onClick={() => projectId && void onPick(projectId)}
        >
          Choose images
        </button>
      </div>

      <div
        className={`conversation-drop-zone${dragActive ? " is-active" : ""}`}
        aria-disabled={!enabled}
        onDragEnter={(event) => {
          if (!enabled) return;
          event.preventDefault();
          setDragActive(true);
        }}
        onDragOver={(event) => {
          if (!enabled) return;
          event.preventDefault();
          event.dataTransfer.dropEffect = "copy";
        }}
        onDragLeave={(event) => {
          if (
            !event.currentTarget.contains(event.relatedTarget as Node | null)
          ) {
            setDragActive(false);
          }
        }}
        onDrop={(event) => void stageDrop(event)}
      >
        <span>Drop PNG/JPEG images here</span>
        <small>
          {availability === "preview"
            ? "Browser preview never reads dropped files."
            : "Files are copied into private, short-lived native staging only after review."}
        </small>
      </div>

      {attachments.length > 0 && (
        <ul className="conversation-attachment-list" aria-label="Staged images">
          {attachments.map((attachment) => (
            <li key={attachment.attachmentId}>
              <div>
                <strong>{attachment.displayName}</strong>
                <span>
                  {formatBytes(attachment.byteSize)} · {attachment.imageWidth} ×{" "}
                  {attachment.imageHeight} ·{" "}
                  {attachment.source.replace("-", " ")}
                </span>
              </div>
              <button
                type="button"
                aria-label={`Remove ${attachment.displayName}`}
                disabled={busy || disabled}
                onClick={() =>
                  projectId && void onCancel(projectId, attachment.attachmentId)
                }
              >
                Remove
              </button>
            </li>
          ))}
        </ul>
      )}

      {snapshot.state === "unavailable" && snapshot.diagnosticCode && (
        <p className="inline-error" role="alert">
          {diagnostics[snapshot.diagnosticCode]}
        </p>
      )}
      {dropError && (
        <p className="inline-error" role="alert">
          {dropError}
        </p>
      )}
      {actionError && (
        <p className="inline-error" role="alert">
          The native attachment operation failed. No draft was sent; any
          existing drafts remain held.
        </p>
      )}
      <p className="conversation-attachments__policy">
        Images are sent only with Start, Resume, or Fork. Cancelled, consumed,
        expired, and startup-stale copies are removed; generic files remain
        unsupported by the reviewed Codex interface.
      </p>
    </div>
  );
}
