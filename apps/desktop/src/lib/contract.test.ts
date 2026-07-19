import fixture from "../../fixtures/desktop-bootstrap.json";
import { describe, expect, it } from "vitest";

import { desktopBootstrapSchema, scaffoldBootstrap } from "./contract";

describe("desktop bootstrap contract", () => {
  it("accepts the shared Rust and TypeScript fixture", () => {
    expect(desktopBootstrapSchema.parse(fixture)).toEqual(scaffoldBootstrap);
  });

  it("rejects identity drift at the IPC boundary", () => {
    expect(() =>
      desktopBootstrapSchema.parse({
        ...fixture,
        product: { ...fixture.product, executable: "codex" },
      }),
    ).toThrow();
  });
});
