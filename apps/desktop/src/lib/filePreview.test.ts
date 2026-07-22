import { describe, expect, it } from "vitest";

import {
  filePreviewSchema,
  scaffoldFilePreview,
  sharedFilePreviewFixture,
} from "./filePreview";

describe("file preview contract", () => {
  it("parses the shared normalized text fixture", () => {
    expect(sharedFilePreviewFixture.kind).toBe("text");
    expect(sharedFilePreviewFixture.displayPath).toBe("docs/preview.md");
    expect(sharedFilePreviewFixture.openActionId).toMatch(/^[0-9a-f-]+$/u);
  });

  it("accepts the empty scaffold", () => {
    expect(filePreviewSchema.parse(scaffoldFilePreview)).toEqual(
      scaffoldFilePreview,
    );
  });

  it("rejects absolute paths and inconsistent active payloads", () => {
    expect(() =>
      filePreviewSchema.parse({
        ...sharedFilePreviewFixture,
        displayPath: "/home/private.txt",
      }),
    ).toThrow();
    expect(() =>
      filePreviewSchema.parse({
        ...sharedFilePreviewFixture,
        kind: "pdf",
        rendering: "metadata-only",
        mimeType: "application/pdf",
      }),
    ).toThrow();
    expect(() =>
      filePreviewSchema.parse({
        ...sharedFilePreviewFixture,
        kind: "image",
        rendering: "bounded-image",
        mimeType: "image/png",
        textContent: null,
        imageDataUrl: "data:image/jpeg;base64,/9g=",
        imageWidth: 1,
        imageHeight: 1,
      }),
    ).toThrow();
    expect(() =>
      filePreviewSchema.parse({
        ...sharedFilePreviewFixture,
        kind: "image",
        rendering: "bounded-image",
        mimeType: "image/jpeg",
        byteSize: 3,
        textContent: null,
        imageDataUrl: "data:image/jpeg;base64,/9g=",
        imageWidth: 1,
        imageHeight: 1,
      }),
    ).toThrow();
    expect(() =>
      filePreviewSchema.parse({
        ...sharedFilePreviewFixture,
        openActionId: null,
      }),
    ).toThrow();
    expect(() =>
      filePreviewSchema.parse({
        ...sharedFilePreviewFixture,
        openActionId: "/private/file",
      }),
    ).toThrow();
    expect(() =>
      filePreviewSchema.parse({
        ...sharedFilePreviewFixture,
        textContent: "line\n".repeat(2_000),
        truncated: true,
      }),
    ).toThrow();
  });
});
