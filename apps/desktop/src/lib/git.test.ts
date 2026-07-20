import { describe, expect, it } from "vitest";

import {
  gitDiffSchema,
  gitPathSchema,
  gitWorkspaceSchema,
  scaffoldGitDiff,
  scaffoldGitWorkspace,
} from "./git";

describe("Git review contract", () => {
  it("parses the shared normalized fixtures", () => {
    expect(gitWorkspaceSchema.parse(scaffoldGitWorkspace)).toEqual(
      scaffoldGitWorkspace,
    );
    expect(gitDiffSchema.parse(scaffoldGitDiff)).toEqual(scaffoldGitDiff);
  });

  it("rejects absolute, escaping, and control-bearing paths", () => {
    for (const path of [
      "/etc/passwd",
      "../outside",
      "src/../outside",
      "bad\nname",
    ]) {
      expect(() => gitPathSchema.parse(path)).toThrow();
    }
  });

  it("rejects raw patch fields from the native boundary", () => {
    expect(() =>
      gitDiffSchema.parse({ ...scaffoldGitDiff, patch: "private raw patch" }),
    ).toThrow();
  });
});
