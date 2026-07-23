import { z } from "zod";

export const desktopNotificationResultSchema = z
  .object({
    schemaVersion: z.literal(1),
    status: z.enum([
      "sent",
      "foreground",
      "duplicate",
      "ineligible",
      "unavailable",
    ]),
  })
  .strict();

export type DesktopNotificationResult = z.infer<
  typeof desktopNotificationResultSchema
>;
