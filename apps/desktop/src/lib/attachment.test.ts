import { describe, expect, it } from "vitest";

import {
  conversationAttachmentCancelRequestSchema,
  conversationAttachmentDropRequestSchema,
  conversationAttachmentSnapshotSchema,
  scaffoldConversationAttachments,
  sharedConversationAttachmentFixture,
} from "./attachment";

describe("conversation attachment contract", () => {
  it("parses the shared opaque attachment fixture", () => {
    expect(
      sharedConversationAttachmentFixture.attachments[0]?.displayName,
    ).toBe("review.png");
    expect(JSON.stringify(sharedConversationAttachmentFixture)).not.toContain(
      "/home/",
    );
  });

  it("accepts the empty scaffold and strict bounded requests", () => {
    expect(
      conversationAttachmentSnapshotSchema.parse(
        scaffoldConversationAttachments,
      ),
    ).toEqual(scaffoldConversationAttachments);
    expect(
      conversationAttachmentDropRequestSchema.parse({
        projectId: sharedConversationAttachmentFixture.projectId,
        files: [
          {
            displayName: "pixel.png",
            declaredMimeType: "image/png",
            base64Data: "iVBORw==",
          },
        ],
      }).files,
    ).toHaveLength(1);
    expect(
      conversationAttachmentCancelRequestSchema.parse({
        projectId: sharedConversationAttachmentFixture.projectId,
        attachmentIds: [
          sharedConversationAttachmentFixture.attachments[0]?.attachmentId,
        ],
      }).attachmentIds,
    ).toHaveLength(1);
  });

  it("rejects paths, duplicate IDs, unknown fields, and inconsistent states", () => {
    expect(() =>
      conversationAttachmentSnapshotSchema.parse({
        ...sharedConversationAttachmentFixture,
        attachments: [
          {
            ...sharedConversationAttachmentFixture.attachments[0],
            displayName: "/private/image.png",
          },
        ],
      }),
    ).toThrow();
    expect(() =>
      conversationAttachmentCancelRequestSchema.parse({
        projectId: sharedConversationAttachmentFixture.projectId,
        attachmentIds: [
          sharedConversationAttachmentFixture.attachments[0]?.attachmentId,
          sharedConversationAttachmentFixture.attachments[0]?.attachmentId,
        ],
      }),
    ).toThrow();
    expect(() =>
      conversationAttachmentSnapshotSchema.parse({
        ...sharedConversationAttachmentFixture,
        nativePath: "/private/image.png",
      }),
    ).toThrow();
    expect(() =>
      conversationAttachmentSnapshotSchema.parse({
        ...sharedConversationAttachmentFixture,
        state: "empty",
      }),
    ).toThrow();
  });
});
