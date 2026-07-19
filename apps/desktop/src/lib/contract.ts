import scaffoldFixture from "../../fixtures/desktop-bootstrap.json";
import { z } from "zod";

const capabilitySchema = z.object({
  id: z.string().min(1),
  label: z.string().min(1),
  state: z.enum(["ready", "planned"]),
  milestone: z.number().int().positive(),
});

export const desktopBootstrapSchema = z.object({
  schemaVersion: z.literal(1),
  product: z.object({
    name: z.literal("QuireForge"),
    tagline: z.literal("Build boldly. Work locally."),
    description: z.literal("An unofficial native Linux workspace for Codex"),
    identifier: z.literal("io.github.codeframe78.QuireForge"),
    executable: z.literal("quireforge"),
    version: z.string().regex(/^\d+\.\d+\.\d+$/u),
  }),
  capabilities: z.array(capabilitySchema).min(1),
});

export type DesktopBootstrap = z.infer<typeof desktopBootstrapSchema>;

export const scaffoldBootstrap = desktopBootstrapSchema.parse(scaffoldFixture);
