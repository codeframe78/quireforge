import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const nativeResponses = {
  desktop_bootstrap: {
    schemaVersion: 1,
    product: {
      name: "QuireForge",
      tagline: "Build boldly. Work locally.",
      description: "An unofficial native Linux workspace for Codex",
      identifier: "io.github.codeframe78.QuireForge",
      executable: "quireforge",
      version: "0.0.0",
    },
    capabilities: [
      {
        id: "desktop-foundation",
        label: "Desktop foundation",
        state: "ready",
        milestone: 3,
      },
      {
        id: "codex-runtime",
        label: "Codex runtime adapter",
        state: "ready",
        milestone: 4,
      },
      {
        id: "codex-auth",
        label: "Codex authentication",
        state: "ready",
        milestone: 5,
      },
      {
        id: "project-attachments",
        label: "Local project attachments",
        state: "ready",
        milestone: 6,
      },
      {
        id: "conversation-runtime",
        label: "Native conversation runtime",
        state: "ready",
        milestone: 7,
      },
    ],
  },
  codex_runtime_probe: {
    schemaVersion: 1,
    adapterVersion: "codex-app-server-v2",
    availability: "ready",
    backend: "app-server-stdio",
    cliVersion: "0.144.6",
    capabilities: [
      { id: "runtime-probe", state: "ready", route: "cli" },
      { id: "app-server-stdio", state: "ready", route: "app-server" },
      { id: "model-discovery", state: "ready", route: "app-server" },
      { id: "normalized-events", state: "ready", route: "native" },
      { id: "conversation-runtime", state: "ready", route: "app-server" },
    ],
    models: [
      {
        id: "gpt-5.6-sol",
        displayName: "GPT-5.6-Sol",
        isDefault: true,
        defaultReasoningEffort: "low",
        supportedReasoningEfforts: ["low", "medium", "high", "xhigh", "max"],
      },
    ],
    diagnosticCode: null,
  },
  codex_auth_status: {
    schemaVersion: 1,
    state: "unauthenticated",
    accountKind: null,
    pendingMethod: null,
    handoff: null,
    diagnosticCode: null,
  },
  project_workspace_status: {
    schemaVersion: 1,
    state: "ready",
    projects: [
      {
        id: "018f0000-0000-7000-8000-000000000001",
        displayName: "QuireForge",
        archived: false,
        directory: {
          associationId: "018f0000-0000-7000-8000-000000000002",
          displayPath: "~/work/quireforge",
          resolvedDisplayPath: "/mnt/work/quireforge",
          state: "connected-accessible",
          expectedAccess: "read-write",
          isPrimary: true,
          git: { isRepository: true, isLinkedWorktree: false },
          hasAgentsGuidance: true,
          hasCodexConfig: false,
        },
      },
    ],
    pendingAttachment: null,
    diagnosticCode: null,
  },
  conversation_status: {
    schemaVersion: 2,
    state: "empty",
    conversationId: null,
    projectId: null,
    modelId: null,
    reasoningEffort: null,
    sandboxMode: null,
    approvalPolicy: null,
    pendingApproval: null,
    events: [],
    diagnosticCode: null,
  },
  conversation_sessions: {
    schemaVersion: 2,
    state: "ready",
    sessions: [
      {
        conversationId: "018f0000-0000-7000-8000-000000000010",
        projectId: "018f0000-0000-7000-8000-000000000001",
        parentConversationId: null,
        title: "Review lifecycle boundaries",
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
        sandboxMode: "workspace-write",
        approvalPolicy: "on-request",
        state: "completed",
        createdAtMs: 1_700_000_000_000,
        updatedAtMs: 1_700_000_001_000,
      },
      {
        conversationId: "018f0000-0000-7000-8000-000000000011",
        projectId: "018f0000-0000-7000-8000-000000000001",
        parentConversationId: "018f0000-0000-7000-8000-000000000010",
        title: "Try the smaller adapter",
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
        sandboxMode: "workspace-write",
        approvalPolicy: "on-request",
        state: "interrupted",
        createdAtMs: 1_700_000_002_000,
        updatedAtMs: 1_700_000_003_000,
      },
    ],
    diagnosticCode: null,
  },
} as const;

async function installNativeFixture(
  page: import("@playwright/test").Page,
  responses: Record<string, unknown> = nativeResponses,
) {
  await page.addInitScript((responses) => {
    const target = window as unknown as {
      __TAURI_INTERNALS__: {
        invoke: (command: string) => Promise<unknown>;
      };
    };
    target.__TAURI_INTERNALS__ = {
      invoke: (command) => {
        if (!(command in responses))
          throw new Error(`Unexpected command: ${command}`);
        return Promise.resolve(structuredClone(responses[command]));
      },
    };
  }, responses);
}

const approvalConversation = {
  schemaVersion: 2,
  state: "waiting-for-approval",
  conversationId: "018f0000-0000-7000-8000-000000000020",
  projectId: "018f0000-0000-7000-8000-000000000001",
  modelId: "gpt-5.6-sol",
  reasoningEffort: "high",
  sandboxMode: "workspace-write",
  approvalPolicy: "on-request",
  pendingApproval: {
    approvalId: "018f0000-0000-7000-8000-000000000021",
    activityId: "018f0000-0000-7000-8000-000000000022",
    kind: "command-execution",
    title: "Run this command?",
    reason: "The project check needs permission.",
    details: [{ label: "Command", value: "pnpm check" }],
    decisions: ["approve", "decline", "cancel"],
  },
  events: [
    {
      type: "activity",
      sequence: 1,
      activityId: "018f0000-0000-7000-8000-000000000022",
      kind: "command-execution",
      status: "started",
      title: "Run command",
      detail: "pnpm check",
      exitCode: null,
    },
    {
      type: "activity-output-delta",
      sequence: 2,
      activityId: "018f0000-0000-7000-8000-000000000022",
      delta: "Checking the desktop contract…",
    },
    {
      type: "approval-requested",
      sequence: 3,
      approvalId: "018f0000-0000-7000-8000-000000000021",
      activityId: "018f0000-0000-7000-8000-000000000022",
      kind: "command-execution",
    },
  ],
  diagnosticCode: null,
} as const;

const approvedConversation = {
  ...approvalConversation,
  state: "completed",
  pendingApproval: null,
  events: [
    ...approvalConversation.events,
    {
      type: "approval-resolved",
      sequence: 4,
      approvalId: "018f0000-0000-7000-8000-000000000021",
      resolution: "approved",
    },
    {
      type: "activity",
      sequence: 5,
      activityId: "018f0000-0000-7000-8000-000000000022",
      kind: "command-execution",
      status: "completed",
      title: "Run command",
      detail: "pnpm check",
      exitCode: 0,
    },
    { type: "lifecycle", sequence: 6, phase: "completed" },
  ],
} as const;

test("desktop preview renders the honest semantic shell", async ({ page }) => {
  const response = await page.goto("/");

  expect(response?.ok()).toBe(true);
  await expect(
    page.getByRole("heading", { name: "A quiet place for ambitious work." }),
  ).toBeVisible();
  await expect(page.getByText("No project attached").first()).toBeAttached();
  await expect(
    page.getByRole("heading", { name: "Work where your files already live." }),
  ).toBeVisible();
  await expect(
    page.getByText(/cannot open a native folder picker/u),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Attach local project" }),
  ).toBeDisabled();
  await expect(
    page.getByText("Browser preview", { exact: true }),
  ).toBeAttached();
  await expect(
    page.getByText("Native probe unavailable").first(),
  ).toBeAttached();
  await expect(
    page.getByRole("heading", { name: "Authentication stays with Codex." }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Start a focused Codex task." }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", {
      name: "Return to work without copying its history.",
    }),
  ).toBeVisible();
  await expect(
    page.getByText(
      "Browser preview cannot inspect or simulate native session history.",
    ),
  ).toBeVisible();
  await expect(
    page.getByText("Browser preview cannot start or simulate a Codex task."),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Start task" })).toBeDisabled();
  await expect(
    page.getByText("Native authentication unavailable"),
  ).toBeVisible();
  await expect(page.locator("main h1")).toHaveCount(1);

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native session fixture renders grouping, tabs, and bounded controls", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await expect(page.getByText("2 app-owned sessions.")).toBeVisible();
  await expect(
    page.getByText(/Fork of Review lifecycle boundaries/u),
  ).toBeVisible();
  await page.getByText("Review lifecycle boundaries", { exact: true }).click();
  await expect(
    page.getByRole("tab", { name: "Review lifecycle boundaries" }),
  ).toHaveAttribute("aria-selected", "true");
  await expect(page.getByLabel("Next task")).toBeVisible();
  await expect(page.getByRole("button", { name: "Resume" })).toBeDisabled();

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native activity fixture renders bounded real-time approval detail", async ({
  page,
}) => {
  await installNativeFixture(page, {
    ...nativeResponses,
    conversation_status: approvalConversation,
    conversation_poll: approvalConversation,
    conversation_approval_decide: approvedConversation,
  });
  await page.goto("/");

  await expect(page.getByText("Codex is waiting for approval")).toBeVisible();
  await expect(
    page.getByText("The project check needs permission."),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Approve once" }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Decline" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Cancel task" })).toBeVisible();
  const activity = page.getByRole("button", { name: /Run command/u });
  await expect(activity).toHaveAttribute("aria-expanded", "false");
  await activity.click();
  await expect(activity).toHaveAttribute("aria-expanded", "true");
  await expect(page.getByText("Checking the desktop contract…")).toBeVisible();
  await expect(
    page.getByText("Approval requested for command execution."),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop task" })).toBeEnabled();

  await page.getByRole("button", { name: "Approve once" }).click();
  await expect(page.getByText("Task completed")).toBeVisible();
  await expect(page.getByText("Approval approved.")).toBeVisible();
  await expect(page.getByText("Run this command?")).toHaveCount(0);

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("desktop preview has no automatically detectable accessibility violations", async ({
  page,
}) => {
  await page.goto("/");
  const results = await new AxeBuilder({ page }).analyze();

  expect(results.violations).toEqual([]);
});

test("theme control changes and persists the selected theme", async ({
  page,
}) => {
  await page.goto("/");
  const toggle = page.getByRole("button", { name: /use (dark|light) theme/iu });
  const before = await page.locator("html").getAttribute("data-theme");

  await toggle.click();
  const after = await page.locator("html").getAttribute("data-theme");
  expect(after).not.toBe(before);
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute(
    "data-theme",
    after ?? "dark",
  );
});
