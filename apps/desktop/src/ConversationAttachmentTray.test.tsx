import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ConversationAttachmentTray } from "./ConversationAttachmentTray";
import {
  scaffoldConversationAttachments,
  sharedConversationAttachmentFixture,
} from "./lib/attachment";

const projectId = "018f6f24-8b71-7c72-9b41-4e0b8ce4c61a";

function renderTray(
  overrides: Partial<
    React.ComponentProps<typeof ConversationAttachmentTray>
  > = {},
) {
  const onPick = vi.fn().mockResolvedValue(undefined);
  const onDrop = vi.fn().mockResolvedValue(undefined);
  const onCancel = vi.fn().mockResolvedValue(undefined);
  render(
    <ConversationAttachmentTray
      availability="native"
      projectId={projectId}
      snapshot={sharedConversationAttachmentFixture}
      busy={false}
      disabled={false}
      actionError={false}
      onPick={onPick}
      onDrop={onDrop}
      onCancel={onCancel}
      {...overrides}
    />,
  );
  return { onPick, onDrop, onCancel };
}

describe("ConversationAttachmentTray", () => {
  it("reviews only opaque staged metadata and removes an exact attachment", () => {
    const { onPick, onCancel } = renderTray();
    expect(screen.getByText("review.png")).toBeInTheDocument();
    expect(document.body.textContent).not.toContain("/private/");
    fireEvent.click(screen.getByRole("button", { name: "Choose images" }));
    expect(onPick).toHaveBeenCalledWith(projectId);
    fireEvent.click(screen.getByRole("button", { name: "Remove review.png" }));
    expect(onCancel).toHaveBeenCalledWith(
      projectId,
      sharedConversationAttachmentFixture.attachments[0]?.attachmentId,
    );
  });

  it("stages a bounded dropped image only in native mode", async () => {
    const { onDrop } = renderTray({
      snapshot: scaffoldConversationAttachments,
    });
    const file = new File([new Uint8Array([1, 2, 3])], "pixel.png", {
      type: "image/png",
    });
    const zone = screen.getByText("Drop PNG/JPEG images here").parentElement!;
    fireEvent.drop(zone, { dataTransfer: { files: [file] } });
    await waitFor(() =>
      expect(onDrop).toHaveBeenCalledWith({
        projectId,
        files: [
          {
            displayName: "pixel.png",
            declaredMimeType: "image/png",
            base64Data: "AQID",
          },
        ],
      }),
    );
  });

  it("claims a native-only drop when WebKitGTK supplies no browser files", async () => {
    const { onDrop } = renderTray({
      snapshot: scaffoldConversationAttachments,
    });
    const zone = screen.getByText("Drop PNG/JPEG images here").parentElement!;
    fireEvent.drop(zone, { dataTransfer: { files: [] } });
    await waitFor(() =>
      expect(onDrop).toHaveBeenCalledWith({ projectId, files: [] }),
    );
    expect(document.body.textContent).not.toContain("/mnt/");
  });

  it("keeps browser preview and oversized drops unavailable", () => {
    const { onDrop } = renderTray({
      availability: "preview",
      snapshot: scaffoldConversationAttachments,
    });
    expect(
      screen.getByRole("button", { name: "Choose images" }),
    ).toBeDisabled();
    expect(screen.getByText(/never reads dropped files/u)).toBeInTheDocument();
    const zone = screen.getByText("Drop PNG/JPEG images here").parentElement!;
    fireEvent.drop(zone, {
      dataTransfer: {
        files: [
          new File([new Uint8Array(1)], "notes.txt", {
            type: "text/plain",
          }),
        ],
      },
    });
    expect(onDrop).not.toHaveBeenCalled();
  });
});
