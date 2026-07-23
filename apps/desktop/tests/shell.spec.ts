import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

import integrationCatalogFixture from "../fixtures/integration-catalog.json" with { type: "json" };
import integrationControlFixture from "../fixtures/integration-control.json" with { type: "json" };
import integrationMutationFixture from "../fixtures/integration-mutation.json" with { type: "json" };
import filePreviewFixture from "../fixtures/file-preview.json" with { type: "json" };
import conversationAttachmentFixture from "../fixtures/conversation-attachments.json" with { type: "json" };
import usageFixture from "../fixtures/codex-usage.json" with { type: "json" };

const nativeIntegrationCatalog = {
  ...integrationCatalogFixture,
  capabilities: integrationCatalogFixture.capabilities.map((capability) =>
    [
      "connector.authorize",
      "plugin.install",
      "plugin.remove",
      "marketplace.configure",
      "skill.configure",
      "mcp.authorize",
      "scheduled-task.catalog",
    ].includes(capability.id)
      ? {
          ...capability,
          availability: "ready",
          implementation: "ready",
          diagnosticCode: null,
        }
      : capability,
  ),
};

const modelSelectionFixture = {
  schemaVersion: 1,
  availability: "ready",
  effective: {
    modelId: "gpt-5.6-sol",
    reasoningEffort: "high",
  },
  pending: null,
  policy: {
    ownership: "manual",
    userLocked: false,
    allowedModelIds: [],
    reasoningCeiling: null,
  },
  diagnosticCode: null,
} as const;

const recommendationSelectionFixture = {
  ...modelSelectionFixture,
  pending: {
    choice: {
      modelId: "gpt-5.6-terra",
      reasoningEffort: "high",
    },
    provenance: "codex",
    application: "recommendation",
    rationale: "Use the bounded lower-latency option for the next turn.",
    requestedAtMs: 1_700_000_001_500,
  },
  policy: {
    ...modelSelectionFixture.policy,
    ownership: "recommend",
  },
} as const;

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
      {
        id: "integrated-terminal",
        label: "Integrated terminal",
        state: "ready",
        milestone: 12,
      },
      {
        id: "integration-center",
        label: "Integration Center",
        state: "ready",
        milestone: 14,
      },
      {
        id: "safe-file-previews",
        label: "Safe file previews",
        state: "ready",
        milestone: 15,
      },
      {
        id: "conversation-attachments",
        label: "Conversation image attachments",
        state: "ready",
        milestone: 15,
      },
      {
        id: "desktop-integration",
        label: "Reviewed desktop integration",
        state: "ready",
        milestone: 15,
      },
      {
        id: "scheduled-task-catalog",
        label: "Read-only scheduled task catalog",
        state: "ready",
        milestone: 17,
      },
      {
        id: "agent-model-selection",
        label: "Policy-bounded next-turn selection",
        state: "ready",
        milestone: 18,
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
      {
        id: "gpt-5.6-terra",
        displayName: "GPT-5.6 Terra",
        isDefault: false,
        defaultReasoningEffort: "medium",
        supportedReasoningEfforts: ["medium", "high"],
      },
    ],
    diagnosticCode: null,
  },
  codex_auth_status: {
    schemaVersion: 1,
    state: "authenticated",
    accountKind: "chatgpt",
    pendingMethod: null,
    handoff: null,
    diagnosticCode: null,
  },
  codex_usage_status: usageFixture,
  codex_usage_refresh: usageFixture,
  file_preview_pick: {
    ...filePreviewFixture,
    projectId: "018f0000-0000-7000-8000-000000000001",
  },
  file_preview_open: null,
  file_preview_cancel: true,
  conversation_notify: {
    schemaVersion: 1,
    status: "foreground",
  },
  conversation_attachment_status: {
    schemaVersion: 1,
    state: "empty",
    projectId: "018f0000-0000-7000-8000-000000000001",
    attachments: [],
    diagnosticCode: null,
  },
  conversation_attachment_pick: {
    ...conversationAttachmentFixture,
    projectId: "018f0000-0000-7000-8000-000000000001",
  },
  conversation_attachment_cancel: {
    schemaVersion: 1,
    state: "empty",
    projectId: "018f0000-0000-7000-8000-000000000001",
    attachments: [],
    diagnosticCode: null,
  },
  integration_catalog_read: nativeIntegrationCatalog,
  integration_catalog_refresh: nativeIntegrationCatalog,
  integration_control_preview: integrationControlFixture.preview,
  integration_control_confirm: integrationControlFixture.result,
  integration_control_open_browser: {
    ...integrationControlFixture.result,
    state: "pending",
    browserHandoffAvailable: false,
  },
  integration_control_status: {
    ...integrationControlFixture.result,
    state: "completed",
    actionId: null,
    browserHandoffAvailable: false,
    catalogRefreshRequired: true,
  },
  integration_mutation_preview: integrationMutationFixture.preview,
  integration_mutation_confirm: integrationMutationFixture.result,
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
  worktree_status: {
    schemaVersion: 2,
    state: "ready",
    sourceProjectId: "018f0000-0000-7000-8000-000000000001",
    worktrees: [
      {
        projectId: "018f0000-0000-7000-8000-000000000001",
        recoveryId: null,
        displayName: "QuireForge",
        displayPath: "~/work/quireforge",
        branchName: "feature/review",
        ownership: "source",
        state: "ready",
        current: true,
      },
      {
        projectId: null,
        recoveryId: "018f0000-0000-7000-8000-000000000041",
        displayName: "feature/recoverable",
        displayPath:
          "~/.local/share/io.github.codeframe78.QuireForge/worktrees/recoverable",
        branchName: "feature/recoverable",
        ownership: "external",
        state: "ready",
        current: false,
      },
      {
        projectId: "018f0000-0000-7000-8000-000000000003",
        recoveryId: null,
        displayName: "feature/managed-cleanup",
        displayPath:
          "~/.local/share/io.github.codeframe78.QuireForge/worktrees/managed-cleanup",
        branchName: "feature/managed-cleanup",
        ownership: "managed",
        state: "ready",
        current: false,
      },
    ],
    truncated: false,
    diagnosticCode: null,
  },
  worktree_create_preview: {
    schemaVersion: 2,
    state: "ready",
    sourceProjectId: "018f0000-0000-7000-8000-000000000001",
    operation: "create",
    branchName: "feature/isolated",
    displayPath: "~/.local/share/quireforge/worktrees/isolated",
    ownership: "managed",
    destructive: false,
    confirmationId: "018f0000-0000-7000-8000-000000000040",
    diagnosticCode: null,
  },
  worktree_recover_preview: {
    schemaVersion: 2,
    state: "ready",
    sourceProjectId: "018f0000-0000-7000-8000-000000000001",
    operation: "recover",
    branchName: "feature/recoverable",
    displayPath:
      "~/.local/share/io.github.codeframe78.QuireForge/worktrees/recoverable",
    ownership: "managed",
    destructive: false,
    confirmationId: "018f0000-0000-7000-8000-000000000042",
    diagnosticCode: null,
  },
  worktree_remove_preview: {
    schemaVersion: 2,
    state: "ready",
    sourceProjectId: "018f0000-0000-7000-8000-000000000001",
    operation: "remove",
    branchName: "feature/managed-cleanup",
    displayPath:
      "~/.local/share/io.github.codeframe78.QuireForge/worktrees/managed-cleanup",
    ownership: "managed",
    destructive: true,
    confirmationId: "018f0000-0000-7000-8000-000000000043",
    diagnosticCode: null,
  },
  worktree_cancel: true,
  git_status: {
    schemaVersion: 1,
    state: "ready",
    projectId: "018f0000-0000-7000-8000-000000000001",
    branch: {
      head: "feature/review",
      upstream: "origin/feature/review",
      ahead: 1,
      behind: 0,
      detached: false,
    },
    changes: [
      {
        path: "src/App.tsx",
        previousPath: null,
        staged: null,
        worktree: "modified",
        conflict: false,
        submodule: false,
        reviewable: true,
      },
    ],
    truncated: false,
    diagnosticCode: null,
  },
  git_diff: {
    schemaVersion: 1,
    state: "ready",
    projectId: "018f0000-0000-7000-8000-000000000001",
    path: "src/App.tsx",
    area: "worktree",
    kind: "text",
    lines: [
      {
        kind: "hunk",
        oldLine: null,
        newLine: null,
        text: "@@ -1 +1 @@",
      },
      { kind: "deletion", oldLine: 1, newLine: null, text: "old line" },
      { kind: "addition", oldLine: null, newLine: 1, text: "new line" },
    ],
    truncated: false,
    diagnosticCode: null,
  },
  git_mutation_preview: {
    schemaVersion: 1,
    state: "ready",
    projectId: "018f0000-0000-7000-8000-000000000001",
    operation: "stage",
    path: "src/App.tsx",
    targets: [
      {
        path: "src/App.tsx",
        staged: null,
        worktree: "modified",
      },
    ],
    destructive: false,
    confirmationId: "018f0000-0000-7000-8000-000000000030",
    secretFindings: [],
    diagnosticCode: null,
  },
  git_mutation_confirm: {
    schemaVersion: 1,
    state: "applied",
    projectId: "018f0000-0000-7000-8000-000000000001",
    operation: "stage",
    recoveryId: null,
    workspace: {
      schemaVersion: 1,
      state: "ready",
      projectId: "018f0000-0000-7000-8000-000000000001",
      branch: {
        head: "feature/review",
        upstream: "origin/feature/review",
        ahead: 1,
        behind: 0,
        detached: false,
      },
      changes: [
        {
          path: "src/App.tsx",
          previousPath: null,
          staged: "modified",
          worktree: null,
          conflict: false,
          submodule: false,
          reviewable: true,
        },
      ],
      truncated: false,
      diagnosticCode: null,
    },
    diagnosticCode: null,
  },
  conversation_status: {
    schemaVersion: 3,
    state: "empty",
    conversationId: null,
    projectId: null,
    modelId: null,
    reasoningEffort: null,
    modelSelection: null,
    sandboxMode: null,
    approvalPolicy: null,
    pendingApproval: null,
    events: [],
    diagnosticCode: null,
  },
  conversation_active: {
    schemaVersion: 1,
    capacity: 4,
    conversations: [],
  },
  terminal_status: {
    schemaVersion: 1,
    capacity: 8,
    terminals: [],
    diagnosticCode: null,
  },
  conversation_sessions: {
    schemaVersion: 3,
    state: "ready",
    sessions: [
      {
        conversationId: "018f0000-0000-7000-8000-000000000010",
        projectId: "018f0000-0000-7000-8000-000000000001",
        parentConversationId: null,
        title: "Review lifecycle boundaries",
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
        modelSelection: recommendationSelectionFixture,
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
        modelSelection: modelSelectionFixture,
        sandboxMode: "workspace-write",
        approvalPolicy: "on-request",
        state: "interrupted",
        createdAtMs: 1_700_000_002_000,
        updatedAtMs: 1_700_000_003_000,
      },
    ],
    diagnosticCode: null,
  },
  model_selection_update: modelSelectionFixture,
} as const;

const liveTerminalSnapshot = {
  schemaVersion: 1,
  state: "running",
  terminalId: "018f0000-0000-7000-8000-000000000050",
  projectId: "018f0000-0000-7000-8000-000000000001",
  title: "Terminal 1",
  live: true,
  columns: 100,
  rows: 30,
  output: [],
  firstSequence: 0,
  lastSequence: 0,
  truncated: false,
  hasMore: false,
  exitCode: null,
  diagnosticCode: null,
} as const;

const nativeTerminalResponses = {
  ...nativeResponses,
  terminal_status: {
    schemaVersion: 1,
    capacity: 8,
    terminals: [liveTerminalSnapshot],
    diagnosticCode: null,
  },
  terminal_poll: liveTerminalSnapshot,
  terminal_resize: liveTerminalSnapshot,
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
  schemaVersion: 3,
  state: "waiting-for-approval",
  conversationId: "018f0000-0000-7000-8000-000000000020",
  projectId: "018f0000-0000-7000-8000-000000000001",
  modelId: "gpt-5.6-sol",
  reasoningEffort: "high",
  modelSelection: modelSelectionFixture,
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
    page.getByRole("heading", { name: "Welcome to QuireForge." }),
  ).toBeVisible();
  await expect(
    page.getByText(
      "Native Codex authentication is unavailable in this browser preview.",
    ),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Try again" })).toBeEnabled();
  await expect(page.getByRole("navigation")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Start task" })).toHaveCount(0);
  await expect(page.locator("main h1")).toHaveCount(1);

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("authenticated home shows real usage without milestone labels", async ({
  page,
  isMobile,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: "What should we build today?" }),
  ).toBeVisible();
  await expect(page.getByText("Codex account connected")).toBeVisible();
  await expect(page.getByText("73% remaining")).toBeVisible();
  await expect(page.getByText("44% remaining")).toBeVisible();
  await expect(page.getByText(/Milestone/u)).toHaveCount(0);
  if (isMobile) {
    await expect(
      page.getByRole("button", { name: /Open terminal/u }),
    ).toBeEnabled();
  } else {
    await expect(page.getByRole("link", { name: "Terminal" })).toHaveAttribute(
      "href",
      "#terminal",
    );
  }

  await page
    .getByLabel("Remaining usage")
    .getByRole("button", { name: "Refresh", exact: true })
    .click();
  await expect(page.getByText("73% remaining")).toBeVisible();
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
  await page
    .locator("#sessions")
    .getByRole("button", {
      name: /Review lifecycle boundaries.*Completed/u,
    })
    .click();
  await expect(
    page.getByRole("tab", { name: "Review lifecycle boundaries" }),
  ).toHaveAttribute("aria-selected", "true");
  await expect(page.getByLabel("Next task")).toBeVisible();
  await expect(page.getByText("Effective now")).toBeVisible();
  await expect(page.getByText("Pending next turn")).toBeVisible();
  await expect(page.getByText("Requested by Codex")).toBeVisible();
  await expect(
    page.getByText("Recommendation — never automatic"),
  ).toBeVisible();
  await page.getByRole("button", { name: "Dismiss" }).click();
  await expect(page.getByText("No change")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Resume", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("heading", { name: "A real shell, rooted where you work." }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "New terminal" }),
  ).toBeEnabled();
  await expect(page.getByText("No terminal open")).toBeVisible();
  await expect(
    page.getByRole("checkbox", { name: "Fixture calendar connector" }),
  ).toBeEnabled();

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native file preview uses the bounded shared contract", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await page.getByRole("button", { name: "Choose project file" }).click();
  await expect(
    page.getByRole("article", { name: "Preview of docs/preview.md" }),
  ).toBeVisible();
  await expect(page.getByText("48 B")).toBeVisible();
  await expect(page.locator(".file-preview-text code")).toContainText(
    "Paths remain native-only.",
  );
  await page.getByRole("button", { name: "Open with desktop app" }).click();
  await expect(
    page.getByText("Destination · System default application"),
  ).toBeVisible();
  await page.getByRole("button", { name: "Open with default app" }).click();
  await expect(
    page.getByRole("button", { name: "Opened with desktop app" }),
  ).toBeDisabled();

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native conversation attachments expose only bounded draft metadata", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await page.getByRole("button", { name: "Choose images" }).click();
  await expect(page.getByText("review.png")).toBeVisible();
  await expect(page.getByText(/67 B · 1 × 1 · drag drop/u)).toBeVisible();
  await expect(
    page.getByText(/sent only with Start, Resume, or Fork/u),
  ).toBeVisible();
  await page.getByRole("button", { name: "Remove review.png" }).click();
  await expect(page.getByText("review.png")).not.toBeVisible();

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native terminal fixture mounts the app-owned xterm tab", async ({
  page,
}) => {
  await installNativeFixture(page, nativeTerminalResponses);
  await page.goto("/");

  await expect(
    page.getByRole("heading", { name: "A real shell, rooted where you work." }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Terminal 1 Running/u }),
  ).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator(".terminal-pane__viewport .xterm")).toBeVisible();
  const closeButton = page.getByRole("button", { name: "Close Terminal 1" });
  await expect(closeButton).toBeVisible();
  await expect(page.getByText(/Linux account privileges/u)).toBeVisible();

  await closeButton.click();
  const closeReview = page.getByRole("alertdialog", {
    name: "Close Terminal 1?",
  });
  await expect(closeReview).toBeVisible();
  await expect(
    page.getByRole("button", { name: "End processes and close" }),
  ).toBeFocused();
  const accessibility = await new AxeBuilder({ page }).analyze();
  expect(accessibility.violations).toEqual([]);
  await page.keyboard.press("Escape");
  await expect(closeReview).toHaveCount(0);
  await expect(closeButton).toBeFocused();

  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native Integration Center reviews trust before a fixed mutation", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  const navigation = page.getByRole("button", { name: /Integrations/u });
  if (await navigation.isVisible()) {
    await expect(navigation).toBeEnabled();
    await navigation.click();
  } else {
    await page.locator("#integrations").scrollIntoViewIfNeeded();
  }
  await expect(
    page.getByRole("heading", { name: "Inspect trust before changing state." }),
  ).toBeVisible();
  await expect(page.getByText("5 of 5 integrations")).toBeVisible();
  await page.getByLabel("Category").selectOption("plugin");
  await expect(
    page.getByRole("heading", { name: "Fixture review plugin" }),
  ).toBeVisible();
  await expect(page.getByText(/requires separate hook trust/u)).toBeVisible();

  await page.getByRole("button", { name: "Install plugin" }).click();
  const confirmation = page.getByRole("dialog", { name: "Install plugin" });
  await expect(confirmation).toContainText(
    "Authentication, if needed, remains a separate action.",
  );
  await expect(confirmation).toContainText("Pinned plugin repository");
  await confirmation.getByRole("button", { name: "Confirm change" }).click();
  await expect(
    page.getByText(/Install plugin completed and the catalog was refreshed/u),
  ).toBeVisible();

  await page.getByLabel("Category").selectOption("mcp-server");
  await page.getByRole("button", { name: "Authorize MCP server" }).click();
  const authorization = page.getByRole("dialog", {
    name: "Authorize MCP server",
  });
  await expect(authorization).toContainText(
    "exact authorization URL returned by Codex",
  );
  await authorization.getByRole("button", { name: "Confirm action" }).click();
  await page
    .getByRole("button", { name: "Open authorization in browser" })
    .click();
  await page.getByRole("button", { name: "Check authorization" }).click();
  await expect(
    page.getByText(
      /Authorize MCP server completed and the catalog was refreshed/u,
    ),
  ).toBeVisible();
  await expect(page.getByText(/authorizationUrl/u)).toHaveCount(0);

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native Scheduled catalog presents inert plugin task templates", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  const navigation = page.getByRole("button", { name: /Scheduled/u });
  if (await navigation.isVisible()) {
    await expect(navigation).toBeEnabled();
    await navigation.click();
  } else {
    await page.locator("#scheduled").scrollIntoViewIfNeeded();
  }

  const scheduled = page.locator("#scheduled");
  await expect(
    scheduled.getByRole("heading", {
      name: "Review task templates without handing over control.",
    }),
  ).toBeVisible();
  await expect(scheduled.getByText("Weekly review")).toBeVisible();
  await expect(scheduled.getByText("Mon, Thu at 09:30")).toBeVisible();
  await expect(scheduled.getByText("Untrusted prompt preview")).toBeVisible();
  await expect(
    scheduled.getByText(
      /cannot create, edit, enable, run, pause, or delete scheduled tasks/u,
    ),
  ).toBeVisible();
  await expect(scheduled.getByRole("button")).toHaveCount(0);

  const results = await new AxeBuilder({ page })
    .include("#scheduled")
    .analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native Git fixture reviews a diff and confirms a fixed mutation", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await expect(
    page
      .getByLabel("Review each Git change before applying it.")
      .getByText("feature/review"),
  ).toBeVisible();
  await page.getByRole("button", { name: "Working · modified" }).click();
  await expect(
    page.getByRole("table", { name: "Diff for src/App.tsx" }),
  ).toBeVisible();
  await expect(page.getByText("new line")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Open in default editor" }),
  ).toBeEnabled();
  await page.getByRole("button", { name: "Stage" }).click();
  const confirmation = page.getByRole("dialog", {
    name: "Stage change",
  });
  await expect(confirmation).toContainText("src/App.tsx");
  await confirmation.getByRole("button", { name: "Confirm stage" }).click();
  await expect(
    page.getByText("The repository was updated and revalidated."),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Staged · modified" }),
  ).toBeVisible();
  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("native worktree fixture reviews creation, recovery, and managed cleanup", async ({
  page,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  await expect(
    page.getByRole("heading", {
      name: "Give each line of work its own checkout.",
    }),
  ).toBeVisible();
  await expect(page.getByText("external checkout")).toBeVisible();
  await page.getByLabel("New branch name").fill("feature/isolated");
  await page.getByRole("button", { name: "Preview managed worktree" }).click();
  await expect(page.getByText("Create feature/isolated")).toBeVisible();
  await expect(page.getByText("Non-destructive preview")).toBeVisible();
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(page.getByText("Create feature/isolated")).toHaveCount(0);

  await page.getByRole("button", { name: "Review recovery" }).click();
  await expect(page.getByText("Recover feature/recoverable")).toBeVisible();
  await expect(
    page.getByText(/registers this retained app-managed checkout/u),
  ).toBeVisible();
  await page.getByRole("button", { name: "Cancel" }).click();

  await page.getByRole("button", { name: "Review cleanup" }).click();
  await expect(page.getByText("Destructive cleanup preview")).toBeVisible();
  await expect(page.getByText(/Its branch is preserved/u)).toBeVisible();
  await expect(
    page.getByRole("button", { name: /force|prune|delete branch/u }),
  ).toHaveCount(0);

  const results = await new AxeBuilder({ page }).analyze();
  expect(results.violations).toEqual([]);
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("parallel worktree monitor opens live activity and reports conflicts", async ({
  page,
}) => {
  const conflictedGit = {
    ...nativeResponses.git_status,
    changes: [
      {
        path: "src/conflicted.ts",
        previousPath: null,
        staged: "unmerged",
        worktree: "unmerged",
        conflict: true,
        submodule: false,
        reviewable: false,
      },
    ],
  };
  await installNativeFixture(page, {
    ...nativeResponses,
    conversation_status: approvalConversation,
    conversation_active: {
      schemaVersion: 1,
      capacity: 4,
      conversations: [{ ...approvalConversation, events: [] }],
    },
    conversation_poll: approvalConversation,
    git_status: conflictedGit,
  });
  await page.goto("/");

  await expect(page.getByText("1 of 4 active")).toBeVisible();
  await expect(page.getByText("Approval needed")).toBeVisible();
  await expect(page.getByText("1 conflict")).toBeVisible();
  await page.getByRole("button", { name: "View live activity" }).click();
  await expect(page.getByText("Codex is waiting for approval")).toBeVisible();
  const activity = page.getByRole("button", { name: /Run command/u });
  await expect(activity).toBeVisible();
  await activity.click();
  await expect(page.getByText("Checking the desktop contract…")).toBeVisible();

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

test("keyboard users can bypass navigation and use semantic workspace links", async ({
  page,
  isMobile,
}) => {
  await installNativeFixture(page);
  await page.goto("/");

  const skipLink = page.getByRole("link", { name: "Skip to workspace" });
  await expect(skipLink).toBeAttached();
  await page.keyboard.press("Tab");
  await expect(skipLink).toBeFocused();
  await expect(skipLink).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("main")).toBeFocused();

  if (isMobile) return;

  const terminalLink = page.getByRole("link", { name: "Terminal" });
  await expect(terminalLink).toHaveAttribute("href", "#terminal");
  await terminalLink.focus();
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/#terminal$/u);
  await expect(
    page.getByRole("heading", {
      name: "A real shell, rooted where you work.",
    }),
  ).toBeVisible();
});

test("reduced-motion preference disables animation and scripted smooth scrolling", async ({
  page,
  isMobile,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.addInitScript(() => {
    const state = window as typeof window & {
      __quireforgeScrollBehaviors: ScrollBehavior[];
    };
    state.__quireforgeScrollBehaviors = [];
    Element.prototype.scrollIntoView = function (
      options?: boolean | ScrollIntoViewOptions,
    ) {
      if (typeof options === "object" && options.behavior) {
        state.__quireforgeScrollBehaviors.push(options.behavior);
      }
    };
  });
  await installNativeFixture(page, nativeResponses);
  await page.goto("/");

  await page.evaluate(() => {
    const probe = document.createElement("span");
    probe.className = "conversation-pulse";
    probe.dataset.testid = "motion-probe";
    document.body.append(probe);
  });
  const animationDuration = await page
    .locator('[data-testid="motion-probe"]')
    .evaluate((element) => getComputedStyle(element).animationDuration);
  const animationDurationMs = animationDuration.endsWith("ms")
    ? Number.parseFloat(animationDuration)
    : Number.parseFloat(animationDuration) * 1_000;
  expect(animationDurationMs).toBeLessThanOrEqual(0.01);

  if (isMobile) return;

  const newThread = page.getByRole("button", { name: /New task/u });
  await expect(newThread).toBeEnabled();
  await newThread.click();
  const behaviors = await page.evaluate(
    () =>
      (
        window as typeof window & {
          __quireforgeScrollBehaviors: ScrollBehavior[];
        }
      ).__quireforgeScrollBehaviors,
  );
  expect(behaviors).toEqual(["auto"]);
});

test("forced-colors mode retains visible controls without horizontal overflow", async ({
  page,
}) => {
  await page.emulateMedia({ forcedColors: "active" });
  await page.goto("/");

  const toggle = page.getByRole("button", { name: /use (dark|light) theme/iu });
  await expect(toggle).toBeVisible();
  await toggle.focus();
  const outlineStyle = await toggle.evaluate(
    (element) => getComputedStyle(element).outlineStyle,
  );
  expect(outlineStyle).not.toBe("none");
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - window.innerWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
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
