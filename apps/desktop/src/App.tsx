import { useEffect, useRef, useState, type ReactNode } from "react";

import brandMark from "../../../assets/brand/quireforge-app-icon.svg";
import { ConversationWorkspace } from "./ConversationWorkspace";
import { FilePreviewWorkspace } from "./FilePreviewWorkspace";
import { GitWorkspace } from "./GitWorkspace";
import { IntegrationCenter } from "./IntegrationCenter";
import { ProjectWorkspace } from "./ProjectWorkspace";
import { SessionWorkspace } from "./SessionWorkspace";
import { TerminalWorkspace } from "./TerminalWorkspace";
import {
  WorktreeWorkspace,
  type WorktreeExecutionView,
} from "./WorktreeWorkspace";
import {
  archiveConversation,
  archiveProject,
  cancelFilePreview,
  cancelConversationAttachments,
  cancelCodexAuth,
  cancelProjectAttachment,
  confirmProjectAttachment,
  confirmGitMutation,
  confirmIntegrationMutation,
  confirmIntegrationControl,
  decideConversationApproval,
  detachProject,
  interruptConversation,
  loadActiveConversations,
  loadCodexAuth,
  loadConversationStatus,
  loadConversationSessions,
  loadCodexRuntime,
  loadDesktopBootstrap,
  loadGitDiff,
  loadGitStatus,
  loadIntegrationCatalog,
  notifyConversation,
  openFilePreview,
  openIntegrationControlBrowser,
  loadProjectWorkspace,
  logoutCodexAuth,
  openGitFile,
  pickConversationAttachments,
  pickFilePreview,
  openCodexAuthBrowser,
  pickProjectDirectory,
  pickProjectRelink,
  preflightProject,
  previewGitMutation,
  previewIntegrationMutation,
  previewIntegrationControl,
  pollIntegrationControl,
  refreshIntegrationCatalog as refreshIntegrationCatalogNative,
  pollConversation,
  refreshCodexAuth,
  recoverGitMutation,
  restoreConversation,
  resumeConversation,
  forkConversation,
  startConversation,
  startCodexAuth,
  stageDroppedConversationAttachments,
  cancelWorktree,
  closeTerminal,
  confirmWorktree,
  loadTerminalStatus,
  loadWorktreeStatus,
  pickWorktreeAttach,
  pollTerminal,
  previewWorktreeCreate,
  previewWorktreeRecover,
  previewWorktreeRemove,
  resizeTerminal,
  startTerminal,
  writeTerminal,
} from "./lib/bridge";
import {
  scaffoldConversationAttachments,
  type ConversationAttachmentCancelRequest,
  type ConversationAttachmentDropRequest,
  type ConversationAttachmentSnapshot,
} from "./lib/attachment";
import {
  scaffoldCodexAuth,
  type AuthLoginMethod,
  type CodexAuthSnapshot,
} from "./lib/auth";
import { scaffoldCodexRuntime, type CodexRuntimeSnapshot } from "./lib/codex";
import { scaffoldBootstrap, type DesktopBootstrap } from "./lib/contract";
import {
  scaffoldFilePreview,
  type FilePreviewHandoffRequest,
  type FilePreviewSnapshot,
} from "./lib/filePreview";
import type { DesktopNotificationResult } from "./lib/desktopIntegration";
import {
  scaffoldConversation,
  type ConversationApprovalDecisionRequest,
  type ConversationEvent,
  type ConversationRegistrySnapshot,
  type ConversationSnapshot,
  type ConversationStartRequest,
} from "./lib/conversation";
import { mergeConversationEvents } from "./lib/conversationView";
import {
  scaffoldGitWorkspace,
  type GitDiffRequest,
  type GitDiffSnapshot,
  type GitMutationConfirmRequest,
  type GitMutationPreviewRequest,
  type GitMutationPreviewSnapshot,
  type GitMutationResultSnapshot,
  type GitOpenFileRequest,
  type GitRecoveryRequest,
  type GitWorkspaceSnapshot,
} from "./lib/git";
import {
  scaffoldProjectWorkspace,
  type ProjectPreflightSnapshot,
  type ProjectWorkspaceSnapshot,
} from "./lib/project";
import {
  scaffoldIntegrationCatalog,
  type IntegrationCatalogSnapshot,
  type IntegrationControlActionRequest,
  type IntegrationControlConfirmationRequest,
  type IntegrationControlPreviewRequest,
  type IntegrationControlPreviewSnapshot,
  type IntegrationControlResultSnapshot,
  type IntegrationMutationConfirmRequest,
  type IntegrationMutationPreviewRequest,
  type IntegrationMutationPreviewSnapshot,
  type IntegrationMutationResultSnapshot,
} from "./lib/integration";
import {
  scaffoldSessionLifecycle,
  type ConversationContinueRequest,
  type SessionLifecycleSnapshot,
  type SessionListRequest,
} from "./lib/session";
import {
  scaffoldTerminalRegistry,
  type TerminalCloseRequest,
  type TerminalPollRequest,
  type TerminalRegistrySnapshot,
  type TerminalResizeRequest,
  type TerminalSnapshot,
  type TerminalStartRequest,
  type TerminalWriteRequest,
} from "./lib/terminal";
import {
  scaffoldWorktreeWorkspace,
  type WorktreeConfirmationRequest,
  type WorktreeCreatePreviewRequest,
  type WorktreePreviewSnapshot,
  type WorktreeRecoverPreviewRequest,
  type WorktreeRemovePreviewRequest,
  type WorktreeResultSnapshot,
  type WorktreeWorkspaceSnapshot,
} from "./lib/worktree";

import "./styles.css";

type BridgeState = "connecting" | "native" | "preview";
type RuntimeState =
  "checking" | "ready" | "degraded" | "unavailable" | "preview";
type AuthViewState = CodexAuthSnapshot["state"] | "checking" | "preview";
type ProjectViewState = "checking" | "native" | "preview";
type GitViewState = "checking" | "native" | "preview";
type WorktreeViewState = "checking" | "native" | "preview";
type ConversationViewState = "checking" | "native" | "preview";
type SessionViewState = "checking" | "native" | "preview";
type TerminalViewState = "checking" | "native" | "preview";
type IntegrationViewState = "checking" | "native" | "preview";
type Theme = "light" | "dark";

interface AppProps {
  loadBootstrap?: () => Promise<DesktopBootstrap>;
  loadRuntime?: () => Promise<CodexRuntimeSnapshot>;
  loadAuth?: () => Promise<CodexAuthSnapshot>;
  refreshAuth?: () => Promise<CodexAuthSnapshot>;
  startAuth?: (method: AuthLoginMethod) => Promise<CodexAuthSnapshot>;
  cancelAuth?: () => Promise<CodexAuthSnapshot>;
  logoutAuth?: () => Promise<CodexAuthSnapshot>;
  openAuthBrowser?: () => Promise<void>;
  loadProjects?: () => Promise<ProjectWorkspaceSnapshot>;
  pickProject?: () => Promise<ProjectWorkspaceSnapshot>;
  pickRelink?: (projectId: string) => Promise<ProjectWorkspaceSnapshot>;
  confirmProject?: () => Promise<ProjectWorkspaceSnapshot>;
  cancelProject?: () => Promise<ProjectWorkspaceSnapshot>;
  detachProjectDirectory?: (
    projectId: string,
  ) => Promise<ProjectWorkspaceSnapshot>;
  archiveProjectMetadata?: (
    projectId: string,
  ) => Promise<ProjectWorkspaceSnapshot>;
  preflightProjectDirectory?: (
    projectId: string,
  ) => Promise<ProjectPreflightSnapshot>;
  pickFilePreviewTask?: (projectId: string) => Promise<FilePreviewSnapshot>;
  openFilePreviewTask?: (request: FilePreviewHandoffRequest) => Promise<void>;
  cancelFilePreviewTask?: (
    request: FilePreviewHandoffRequest,
  ) => Promise<boolean>;
  pickConversationAttachmentsTask?: (
    projectId: string,
  ) => Promise<ConversationAttachmentSnapshot>;
  stageDroppedConversationAttachmentsTask?: (
    request: ConversationAttachmentDropRequest,
  ) => Promise<ConversationAttachmentSnapshot>;
  cancelConversationAttachmentsTask?: (
    request: ConversationAttachmentCancelRequest,
  ) => Promise<ConversationAttachmentSnapshot>;
  loadWorktreesTask?: (projectId: string) => Promise<WorktreeWorkspaceSnapshot>;
  previewWorktreeCreateTask?: (
    request: WorktreeCreatePreviewRequest,
  ) => Promise<WorktreePreviewSnapshot>;
  previewWorktreeRecoverTask?: (
    request: WorktreeRecoverPreviewRequest,
  ) => Promise<WorktreePreviewSnapshot>;
  previewWorktreeRemoveTask?: (
    request: WorktreeRemovePreviewRequest,
  ) => Promise<WorktreePreviewSnapshot>;
  pickWorktreeAttachTask?: (
    projectId: string,
  ) => Promise<WorktreePreviewSnapshot>;
  confirmWorktreeTask?: (
    request: WorktreeConfirmationRequest,
  ) => Promise<WorktreeResultSnapshot>;
  cancelWorktreeTask?: (
    request: WorktreeConfirmationRequest,
  ) => Promise<boolean>;
  loadGitStatusTask?: (projectId: string) => Promise<GitWorkspaceSnapshot>;
  loadGitDiffTask?: (request: GitDiffRequest) => Promise<GitDiffSnapshot>;
  openGitFileTask?: (request: GitOpenFileRequest) => Promise<void>;
  previewGitMutationTask?: (
    request: GitMutationPreviewRequest,
  ) => Promise<GitMutationPreviewSnapshot>;
  confirmGitMutationTask?: (
    request: GitMutationConfirmRequest,
  ) => Promise<GitMutationResultSnapshot>;
  recoverGitMutationTask?: (
    request: GitRecoveryRequest,
  ) => Promise<GitMutationResultSnapshot>;
  loadConversation?: () => Promise<ConversationSnapshot>;
  loadActiveConversationTasks?: () => Promise<ConversationRegistrySnapshot>;
  startConversationTask?: (
    request: ConversationStartRequest,
  ) => Promise<ConversationSnapshot>;
  pollConversationTask?: (
    conversationId: string,
  ) => Promise<ConversationSnapshot>;
  notifyConversationTask?: (
    conversationId: string,
  ) => Promise<DesktopNotificationResult>;
  interruptConversationTask?: (
    conversationId: string,
  ) => Promise<ConversationSnapshot>;
  decideConversationApprovalTask?: (
    request: ConversationApprovalDecisionRequest,
  ) => Promise<ConversationSnapshot>;
  loadSessions?: (
    request?: SessionListRequest,
  ) => Promise<SessionLifecycleSnapshot>;
  resumeConversationTask?: (
    request: ConversationContinueRequest,
  ) => Promise<ConversationSnapshot>;
  forkConversationTask?: (
    request: ConversationContinueRequest,
  ) => Promise<ConversationSnapshot>;
  archiveConversationTask?: (
    conversationId: string,
  ) => Promise<SessionLifecycleSnapshot>;
  restoreConversationTask?: (
    conversationId: string,
  ) => Promise<SessionLifecycleSnapshot>;
  loadTerminalsTask?: () => Promise<TerminalRegistrySnapshot>;
  startTerminalTask?: (
    request: TerminalStartRequest,
  ) => Promise<TerminalSnapshot>;
  pollTerminalTask?: (
    request: TerminalPollRequest,
  ) => Promise<TerminalSnapshot>;
  writeTerminalTask?: (
    request: TerminalWriteRequest,
  ) => Promise<TerminalSnapshot>;
  resizeTerminalTask?: (
    request: TerminalResizeRequest,
  ) => Promise<TerminalSnapshot>;
  closeTerminalTask?: (
    request: TerminalCloseRequest,
  ) => Promise<TerminalRegistrySnapshot>;
  loadIntegrationCatalogTask?: () => Promise<IntegrationCatalogSnapshot>;
  refreshIntegrationCatalogTask?: () => Promise<IntegrationCatalogSnapshot>;
  previewIntegrationMutationTask?: (
    request: IntegrationMutationPreviewRequest,
  ) => Promise<IntegrationMutationPreviewSnapshot>;
  confirmIntegrationMutationTask?: (
    request: IntegrationMutationConfirmRequest,
  ) => Promise<IntegrationMutationResultSnapshot>;
  previewIntegrationControlTask?: (
    request: IntegrationControlPreviewRequest,
  ) => Promise<IntegrationControlPreviewSnapshot>;
  confirmIntegrationControlTask?: (
    request: IntegrationControlConfirmationRequest,
  ) => Promise<IntegrationControlResultSnapshot>;
  openIntegrationControlTask?: (
    request: IntegrationControlActionRequest,
  ) => Promise<IntegrationControlResultSnapshot>;
  pollIntegrationControlTask?: (
    request: IntegrationControlActionRequest,
  ) => Promise<IntegrationControlResultSnapshot>;
}

interface TrackedConversation {
  snapshot: ConversationSnapshot;
  events: ConversationEvent[];
}

const navigation = [
  {
    label: "Workspace",
    milestone: 3,
    icon: "grid",
    target: "workspace-top",
    ready: true,
  },
  {
    label: "Files",
    milestone: 15,
    icon: "folder",
    target: "files",
    ready: true,
  },
  {
    label: "Changes",
    milestone: 10,
    icon: "git",
    target: "changes",
    ready: true,
  },
  {
    label: "Worktrees",
    milestone: 11,
    icon: "git",
    target: "worktrees",
    ready: true,
  },
  {
    label: "Terminal",
    milestone: 12,
    icon: "terminal",
    target: "terminal",
    ready: true,
  },
  {
    label: "Threads",
    milestone: 8,
    icon: "thread",
    target: "sessions",
    ready: true,
  },
  {
    label: "Integrations",
    milestone: 14,
    icon: "blocks",
    target: "integrations",
    ready: true,
  },
] as const;

function initialTheme(): Theme {
  const stored = window.localStorage.getItem("quireforge-theme");
  if (stored === "light" || stored === "dark") return stored;
  return window.matchMedia?.("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

function Glyph({ name }: { name: string }) {
  const paths: Record<string, ReactNode> = {
    grid: (
      <>
        <rect x="3" y="3" width="7" height="7" rx="2" />
        <rect x="14" y="3" width="7" height="7" rx="2" />
        <rect x="3" y="14" width="7" height="7" rx="2" />
        <rect x="14" y="14" width="7" height="7" rx="2" />
      </>
    ),
    thread: (
      <>
        <path d="M6 7.5h12M6 12h8M6 16.5h5" />
        <path d="M4 3.5h16a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2h-8l-5 3v-3H4a2 2 0 0 1-2-2v-11a2 2 0 0 1 2-2Z" />
      </>
    ),
    blocks: (
      <>
        <path d="m8 3 4 2.3v4.6L8 12.2 4 9.9V5.3L8 3ZM16 11.8l4 2.3v4.6L16 21l-4-2.3v-4.6l4-2.3Z" />
        <path d="m16 3 4 2.3v4.6l-4 2.3-4-2.3V5.3L16 3ZM8 11.8l4 2.3v4.6L8 21l-4-2.3v-4.6l4-2.3Z" />
      </>
    ),
    git: (
      <>
        <circle cx="6" cy="5" r="2.5" />
        <circle cx="18" cy="19" r="2.5" />
        <circle cx="18" cy="7" r="2.5" />
        <path d="M8.5 5h2.2A3.3 3.3 0 0 1 14 8.3v7.4a3.3 3.3 0 0 0 3.3 3.3h-1.8M8.5 5A3.5 3.5 0 0 1 12 8.5v2A3.5 3.5 0 0 0 15.5 14H18" />
      </>
    ),
    plus: <path d="M12 5v14M5 12h14" />,
    folder: (
      <path d="M3 6.5A2.5 2.5 0 0 1 5.5 4H10l2 2h6.5A2.5 2.5 0 0 1 21 8.5v8A2.5 2.5 0 0 1 18.5 19h-13A2.5 2.5 0 0 1 3 16.5v-10Z" />
    ),
    terminal: (
      <>
        <path d="m5 7 4 5-4 5M11 17h8" />
        <rect x="2.5" y="3.5" width="19" height="17" rx="3" />
      </>
    ),
    shield: (
      <>
        <path d="M12 2.8 20 6v5.7c0 4.5-3.1 8-8 9.5-4.9-1.5-8-5-8-9.5V6l8-3.2Z" />
        <path d="m8.5 12 2.2 2.2 4.8-5" />
      </>
    ),
    external: (
      <>
        <path d="M14 4h6v6M20 4l-9 9" />
        <path d="M18 13v5a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h5" />
      </>
    ),
    refresh: (
      <>
        <path d="M20 7v5h-5" />
        <path d="M4 17v-5h5" />
        <path d="M6.1 9A7 7 0 0 1 18.5 7L20 12M4 12l1.5 5A7 7 0 0 0 17.9 15" />
      </>
    ),
    check: <path d="m5 12 4.2 4.2L19 6.5" />,
    chevron: <path d="m9 18 6-6-6-6" />,
  };

  return (
    <svg
      aria-hidden="true"
      className="glyph"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth="1.7"
    >
      {paths[name]}
    </svg>
  );
}

function StatusDot({ state }: { state: BridgeState }) {
  return (
    <span className={`status-dot status-dot--${state}`} aria-hidden="true" />
  );
}

export default function App({
  loadBootstrap = loadDesktopBootstrap,
  loadRuntime = loadCodexRuntime,
  loadAuth = loadCodexAuth,
  refreshAuth = refreshCodexAuth,
  startAuth = startCodexAuth,
  cancelAuth = cancelCodexAuth,
  logoutAuth = logoutCodexAuth,
  openAuthBrowser = openCodexAuthBrowser,
  loadProjects = loadProjectWorkspace,
  pickProject = pickProjectDirectory,
  pickRelink = pickProjectRelink,
  confirmProject = confirmProjectAttachment,
  cancelProject = cancelProjectAttachment,
  detachProjectDirectory = detachProject,
  archiveProjectMetadata = archiveProject,
  preflightProjectDirectory = preflightProject,
  pickFilePreviewTask = pickFilePreview,
  openFilePreviewTask = openFilePreview,
  cancelFilePreviewTask = cancelFilePreview,
  pickConversationAttachmentsTask = pickConversationAttachments,
  stageDroppedConversationAttachmentsTask = stageDroppedConversationAttachments,
  cancelConversationAttachmentsTask = cancelConversationAttachments,
  loadWorktreesTask = loadWorktreeStatus,
  previewWorktreeCreateTask = previewWorktreeCreate,
  previewWorktreeRecoverTask = previewWorktreeRecover,
  previewWorktreeRemoveTask = previewWorktreeRemove,
  pickWorktreeAttachTask = pickWorktreeAttach,
  confirmWorktreeTask = confirmWorktree,
  cancelWorktreeTask = cancelWorktree,
  loadGitStatusTask = loadGitStatus,
  loadGitDiffTask = loadGitDiff,
  openGitFileTask = openGitFile,
  previewGitMutationTask = previewGitMutation,
  confirmGitMutationTask = confirmGitMutation,
  recoverGitMutationTask = recoverGitMutation,
  loadConversation = loadConversationStatus,
  loadActiveConversationTasks = loadActiveConversations,
  startConversationTask = startConversation,
  pollConversationTask = pollConversation,
  notifyConversationTask = notifyConversation,
  interruptConversationTask = interruptConversation,
  decideConversationApprovalTask = decideConversationApproval,
  loadSessions = loadConversationSessions,
  resumeConversationTask = resumeConversation,
  forkConversationTask = forkConversation,
  archiveConversationTask = archiveConversation,
  restoreConversationTask = restoreConversation,
  loadTerminalsTask = loadTerminalStatus,
  startTerminalTask = startTerminal,
  pollTerminalTask = pollTerminal,
  writeTerminalTask = writeTerminal,
  resizeTerminalTask = resizeTerminal,
  closeTerminalTask = closeTerminal,
  loadIntegrationCatalogTask = loadIntegrationCatalog,
  refreshIntegrationCatalogTask = refreshIntegrationCatalogNative,
  previewIntegrationMutationTask = previewIntegrationMutation,
  confirmIntegrationMutationTask = confirmIntegrationMutation,
  previewIntegrationControlTask = previewIntegrationControl,
  confirmIntegrationControlTask = confirmIntegrationControl,
  openIntegrationControlTask = openIntegrationControlBrowser,
  pollIntegrationControlTask = pollIntegrationControl,
}: AppProps) {
  const [bootstrap, setBootstrap] =
    useState<DesktopBootstrap>(scaffoldBootstrap);
  const [bridgeState, setBridgeState] = useState<BridgeState>("connecting");
  const [runtime, setRuntime] =
    useState<CodexRuntimeSnapshot>(scaffoldCodexRuntime);
  const [runtimeState, setRuntimeState] = useState<RuntimeState>("checking");
  const [auth, setAuth] = useState<CodexAuthSnapshot>(scaffoldCodexAuth);
  const [authState, setAuthState] = useState<AuthViewState>("checking");
  const [authBusy, setAuthBusy] = useState(false);
  const [authActionError, setAuthActionError] = useState(false);
  const [confirmLogout, setConfirmLogout] = useState(false);
  const [projects, setProjects] = useState<ProjectWorkspaceSnapshot>(
    scaffoldProjectWorkspace,
  );
  const [projectState, setProjectState] =
    useState<ProjectViewState>("checking");
  const [projectBusy, setProjectBusy] = useState(false);
  const [projectActionError, setProjectActionError] = useState(false);
  const [projectPreflights, setProjectPreflights] = useState<
    Record<string, ProjectPreflightSnapshot>
  >({});
  const [filePreview, setFilePreview] =
    useState<FilePreviewSnapshot>(scaffoldFilePreview);
  const [filePreviewBusy, setFilePreviewBusy] = useState(false);
  const [filePreviewActionError, setFilePreviewActionError] = useState(false);
  const [conversationAttachments, setConversationAttachments] =
    useState<ConversationAttachmentSnapshot>(scaffoldConversationAttachments);
  const [conversationAttachmentBusy, setConversationAttachmentBusy] =
    useState(false);
  const [
    conversationAttachmentActionError,
    setConversationAttachmentActionError,
  ] = useState(false);
  const [worktrees, setWorktrees] = useState<WorktreeWorkspaceSnapshot>(
    scaffoldWorktreeWorkspace,
  );
  const [worktreePreview, setWorktreePreview] =
    useState<WorktreePreviewSnapshot | null>(null);
  const [worktreeResult, setWorktreeResult] =
    useState<WorktreeResultSnapshot | null>(null);
  const [worktreeState, setWorktreeState] =
    useState<WorktreeViewState>("checking");
  const [worktreeBusy, setWorktreeBusy] = useState(false);
  const [worktreeActionError, setWorktreeActionError] = useState(false);
  const [gitSnapshot, setGitSnapshot] =
    useState<GitWorkspaceSnapshot>(scaffoldGitWorkspace);
  const [gitDiff, setGitDiff] = useState<GitDiffSnapshot | null>(null);
  const [gitSelectedRequest, setGitSelectedRequest] =
    useState<GitDiffRequest | null>(null);
  const [gitState, setGitState] = useState<GitViewState>("checking");
  const [gitBusy, setGitBusy] = useState(false);
  const [gitActionError, setGitActionError] = useState(false);
  const [gitMutationPreview, setGitMutationPreview] =
    useState<GitMutationPreviewSnapshot | null>(null);
  const [gitMutationResult, setGitMutationResult] =
    useState<GitMutationResultSnapshot | null>(null);
  const [taskGitSnapshots, setTaskGitSnapshots] = useState<
    Record<string, GitWorkspaceSnapshot>
  >({});
  const [conversation, setConversation] =
    useState<ConversationSnapshot>(scaffoldConversation);
  const [conversationEvents, setConversationEvents] = useState<
    ConversationEvent[]
  >([]);
  const [trackedConversations, setTrackedConversations] = useState<
    Record<string, TrackedConversation>
  >({});
  const [conversationState, setConversationState] =
    useState<ConversationViewState>("checking");
  const [conversationBusy, setConversationBusy] = useState(false);
  const [conversationActionError, setConversationActionError] = useState(false);
  const conversationActionGenerations = useRef<Record<string, number>>({});
  const observedConversationStates = useRef<
    Record<string, ConversationSnapshot["state"]>
  >({});
  const [sessions, setSessions] = useState<SessionLifecycleSnapshot>(
    scaffoldSessionLifecycle,
  );
  const [sessionState, setSessionState] =
    useState<SessionViewState>("checking");
  const [sessionBusy, setSessionBusy] = useState(false);
  const [sessionActionError, setSessionActionError] = useState(false);
  const [sessionSearchTerm, setSessionSearchTerm] = useState<string | null>(
    null,
  );
  const [terminals, setTerminals] = useState<TerminalRegistrySnapshot>(
    scaffoldTerminalRegistry,
  );
  const [terminalState, setTerminalState] =
    useState<TerminalViewState>("checking");
  const [terminalBusy, setTerminalBusy] = useState(false);
  const [terminalActionError, setTerminalActionError] = useState(false);
  const [integrationCatalog, setIntegrationCatalog] =
    useState<IntegrationCatalogSnapshot>(scaffoldIntegrationCatalog);
  const [integrationPreview, setIntegrationPreview] =
    useState<IntegrationMutationPreviewSnapshot | null>(null);
  const [integrationResult, setIntegrationResult] =
    useState<IntegrationMutationResultSnapshot | null>(null);
  const [integrationControlPreview, setIntegrationControlPreview] =
    useState<IntegrationControlPreviewSnapshot | null>(null);
  const [integrationControlResult, setIntegrationControlResult] =
    useState<IntegrationControlResultSnapshot | null>(null);
  const [integrationState, setIntegrationState] =
    useState<IntegrationViewState>("checking");
  const [integrationBusy, setIntegrationBusy] = useState(false);
  const [integrationActionError, setIntegrationActionError] = useState(false);
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(
    null,
  );
  const [theme, setTheme] = useState<Theme>(initialTheme);

  useEffect(() => {
    let active = true;
    void loadBootstrap()
      .then((result) => {
        if (!active) return;
        setBootstrap(result);
        setBridgeState("native");
      })
      .catch(() => {
        if (active) setBridgeState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadBootstrap]);

  useEffect(() => {
    let active = true;
    void loadRuntime()
      .then((result) => {
        if (!active) return;
        setRuntime(result);
        setRuntimeState(result.availability);
      })
      .catch(() => {
        if (active) setRuntimeState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadRuntime]);

  useEffect(() => {
    let active = true;
    void loadAuth()
      .then((result) => {
        if (!active) return;
        setAuth(result);
        setAuthState(result.state);
      })
      .catch(() => {
        if (active) setAuthState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadAuth]);

  useEffect(() => {
    let active = true;
    void loadProjects()
      .then((result) => {
        if (!active) return;
        setProjects(result);
        setProjectState("native");
      })
      .catch(() => {
        if (active) setProjectState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadProjects]);

  useEffect(() => {
    let active = true;
    void loadConversation()
      .then((result) => {
        if (!active) return;
        setConversation(result);
        setConversationEvents(result.events);
        if (result.projectId && result.conversationId) {
          setTrackedConversations((current) => ({
            ...current,
            [result.projectId!]: { snapshot: result, events: result.events },
          }));
        }
        setConversationState("native");
      })
      .catch(() => {
        if (active) setConversationState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadConversation]);

  useEffect(() => {
    let active = true;
    void loadActiveConversationTasks()
      .then((registry) => {
        if (!active) return;
        setTrackedConversations(
          Object.fromEntries(
            registry.conversations.flatMap((snapshot) =>
              snapshot.projectId
                ? [[snapshot.projectId, { snapshot, events: snapshot.events }]]
                : [],
            ),
          ),
        );
      })
      .catch(() => {
        // Older/preview bridges have no active native process registry.
      });

    return () => {
      active = false;
    };
  }, [loadActiveConversationTasks]);

  useEffect(() => {
    if (projectState === "checking") return;
    let active = true;
    const resetReview = (state: GitViewState) => {
      void Promise.resolve().then(() => {
        if (!active) return;
        setGitState(state);
        setGitSnapshot(scaffoldGitWorkspace);
        setGitDiff(null);
        setGitSelectedRequest(null);
        setGitActionError(false);
        setGitMutationPreview(null);
        setGitMutationResult(null);
      });
    };
    if (projectState === "preview") {
      resetReview("preview");
      return () => {
        active = false;
      };
    }
    const project =
      projects.projects.find(
        (candidate) =>
          candidate.id === selectedProjectId && !candidate.archived,
      ) ??
      projects.projects.find((candidate) => !candidate.archived) ??
      projects.projects[0];
    if (!project) {
      resetReview("native");
      return () => {
        active = false;
      };
    }

    void Promise.resolve().then(() => {
      if (!active) return;
      setGitState("checking");
      setGitDiff(null);
      setGitSelectedRequest(null);
      setGitActionError(false);
      setGitMutationPreview(null);
      setGitMutationResult(null);
    });
    void loadGitStatusTask(project.id)
      .then((result) => {
        if (!active) return;
        setGitSnapshot(result);
        setGitState("native");
      })
      .catch(() => {
        if (!active) return;
        setGitSnapshot(scaffoldGitWorkspace);
        setGitState("preview");
      });
    return () => {
      active = false;
    };
  }, [loadGitStatusTask, projectState, projects, selectedProjectId]);

  useEffect(() => {
    if (projectState === "checking") return;
    let active = true;
    const resetWorktrees = (state: WorktreeViewState) => {
      void Promise.resolve().then(() => {
        if (!active) return;
        setWorktreeState(state);
        setWorktrees(scaffoldWorktreeWorkspace);
        setWorktreePreview(null);
        setWorktreeResult(null);
        setWorktreeActionError(false);
      });
    };
    if (projectState === "preview") {
      resetWorktrees("preview");
      return () => {
        active = false;
      };
    }
    const project =
      projects.projects.find(
        (candidate) =>
          candidate.id === selectedProjectId && !candidate.archived,
      ) ??
      projects.projects.find((candidate) => !candidate.archived) ??
      projects.projects[0];
    if (!project) {
      resetWorktrees("native");
      return () => {
        active = false;
      };
    }
    void Promise.resolve().then(() => {
      if (!active) return;
      setWorktreeState("checking");
      setWorktreePreview(null);
      setWorktreeResult(null);
      setWorktreeActionError(false);
    });
    void loadWorktreesTask(project.id)
      .then((result) => {
        if (!active) return;
        setWorktrees(result);
        setWorktreeState("native");
      })
      .catch(() => {
        if (!active) return;
        setWorktrees(scaffoldWorktreeWorkspace);
        setWorktreeState("preview");
      });
    return () => {
      active = false;
    };
  }, [loadWorktreesTask, projectState, projects, selectedProjectId]);

  useEffect(() => {
    let active = true;
    void loadSessions({ projectId: null, searchTerm: null })
      .then((result) => {
        if (!active) return;
        setSessions(result);
        setSessionState("native");
      })
      .catch(() => {
        if (active) setSessionState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadSessions]);

  useEffect(() => {
    let active = true;
    void loadTerminalsTask()
      .then((result) => {
        if (!active) return;
        setTerminals(result);
        setTerminalState("native");
      })
      .catch(() => {
        if (active) setTerminalState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadTerminalsTask]);

  useEffect(() => {
    let active = true;
    void loadIntegrationCatalogTask()
      .then((result) => {
        if (!active) return;
        setIntegrationCatalog(result);
        setIntegrationState("native");
      })
      .catch(() => {
        if (active) setIntegrationState("preview");
      });

    return () => {
      active = false;
    };
  }, [loadIntegrationCatalogTask]);

  useEffect(() => {
    if (authState !== "login-pending") return;
    let active = true;
    const poll = window.setInterval(() => {
      void loadAuth()
        .then((result) => {
          if (!active) return;
          setAuth(result);
          setAuthState(result.state);
        })
        .catch(() => {
          if (active) setAuthState("unavailable");
        });
    }, 750);

    return () => {
      active = false;
      window.clearInterval(poll);
    };
  }, [authState, loadAuth]);

  const activeConversationIds = Object.values(trackedConversations)
    .map((tracked) => tracked.snapshot)
    .filter((snapshot) =>
      ["running", "waiting-for-approval", "stopping"].includes(snapshot.state),
    )
    .flatMap((snapshot) =>
      snapshot.conversationId ? [snapshot.conversationId] : [],
    )
    .sort();
  const activeConversationKey = activeConversationIds.join(",");
  const activeTaskProjectIds = Object.values(trackedConversations)
    .map((tracked) => tracked.snapshot)
    .filter((snapshot) =>
      ["running", "waiting-for-approval", "stopping"].includes(snapshot.state),
    )
    .flatMap((snapshot) => (snapshot.projectId ? [snapshot.projectId] : []))
    .sort();
  const activeTaskProjectKey = activeTaskProjectIds.join(",");

  useEffect(() => {
    if (!activeConversationKey) return;

    let active = true;
    let timer: number | undefined;
    const ids = activeConversationKey.split(",");
    const observed = observedConversationStates.current;
    observedConversationStates.current = Object.fromEntries(
      ids.flatMap((conversationId) =>
        observed[conversationId]
          ? [[conversationId, observed[conversationId]]]
          : [],
      ),
    );

    async function poll() {
      const pollGenerations = Object.fromEntries(
        ids.map((conversationId) => [
          conversationId,
          conversationActionGenerations.current[conversationId] ?? 0,
        ]),
      );
      const settled = await Promise.allSettled(
        ids.map((conversationId) => pollConversationTask(conversationId)),
      );
      if (!active) return;
      const results = settled.flatMap((result, index) =>
        result.status === "fulfilled" &&
        pollGenerations[ids[index]!] ===
          (conversationActionGenerations.current[ids[index]!] ?? 0)
          ? [result.value]
          : [],
      );
      if (settled.some((result) => result.status === "rejected"))
        setConversationActionError(true);

      for (const result of results) {
        if (!result.conversationId) continue;
        const previous =
          observedConversationStates.current[result.conversationId];
        observedConversationStates.current[result.conversationId] =
          result.state;
        if (
          previous !== result.state &&
          ["waiting-for-approval", "completed", "blocked", "failed"].includes(
            result.state,
          )
        ) {
          void notifyConversationTask(result.conversationId).catch(() => {
            // Notification delivery is best-effort and never changes task state.
          });
        }
      }

      setTrackedConversations((current) => {
        const next = { ...current };
        for (const result of results) {
          if (!result.projectId || !result.conversationId) continue;
          const previous = current[result.projectId];
          if (
            previous &&
            previous.snapshot.conversationId !== result.conversationId
          )
            continue;
          next[result.projectId] = {
            snapshot: result,
            events: mergeConversationEvents(
              previous?.events ?? [],
              result.events,
            ),
          };
        }
        return next;
      });

      const displayed = results.find(
        (result) => result.conversationId === conversation.conversationId,
      );
      if (displayed) {
        setConversation(displayed);
        setConversationEvents((current) =>
          mergeConversationEvents(current, displayed.events),
        );
      }
      if (
        results.some(
          (result) =>
            !["running", "waiting-for-approval", "stopping"].includes(
              result.state,
            ),
        )
      ) {
        void loadSessions({ projectId: null, searchTerm: sessionSearchTerm })
          .then((sessionResult) => setSessions(sessionResult))
          .catch(() => setSessionActionError(true));
        for (const result of results) {
          if (
            result.projectId &&
            !["running", "waiting-for-approval", "stopping"].includes(
              result.state,
            )
          ) {
            void loadGitStatusTask(result.projectId)
              .then((gitResult) => {
                if (gitResult.projectId) {
                  setTaskGitSnapshots((current) => ({
                    ...current,
                    [gitResult.projectId!]: gitResult,
                  }));
                }
              })
              .catch(() => setGitActionError(true));
          }
        }
      }
      timer = window.setTimeout(() => void poll(), 250);
    }

    timer = window.setTimeout(() => void poll(), 250);
    return () => {
      active = false;
      if (timer !== undefined) window.clearTimeout(timer);
    };
  }, [
    activeConversationKey,
    conversation.conversationId,
    loadGitStatusTask,
    loadSessions,
    notifyConversationTask,
    pollConversationTask,
    sessionSearchTerm,
  ]);

  useEffect(() => {
    if (!activeTaskProjectKey) return;
    let active = true;
    const projectIds = activeTaskProjectKey.split(",");

    async function refreshTaskGit() {
      const settled = await Promise.allSettled(
        projectIds.map((projectId) => loadGitStatusTask(projectId)),
      );
      if (!active) return;
      setTaskGitSnapshots((current) => {
        const next = { ...current };
        for (const [index, result] of settled.entries()) {
          if (result.status === "fulfilled" && result.value.projectId) {
            next[result.value.projectId] = result.value;
          } else if (result.status === "rejected") {
            delete next[projectIds[index]!];
          }
        }
        return next;
      });
    }

    void refreshTaskGit();
    const interval = window.setInterval(() => void refreshTaskGit(), 2000);
    return () => {
      active = false;
      window.clearInterval(interval);
    };
  }, [activeTaskProjectKey, loadGitStatusTask]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem("quireforge-theme", theme);
  }, [theme]);

  const bridgeLabel = {
    connecting: "Checking native bridge",
    native: "Native IPC verified",
    preview: "Browser preview",
  }[bridgeState];

  const runtimeLabel = {
    checking: "Checking Codex adapter",
    ready: "Codex adapter ready",
    degraded: "CLI fallback only",
    unavailable: "Codex unavailable",
    preview: "Native probe unavailable",
  }[runtimeState];

  const authLabel = {
    checking: "Checking Codex account",
    authenticated: "Codex account connected",
    unauthenticated: "Codex sign-in available",
    "login-pending": "Codex sign-in pending",
    "not-required": "Provider sign-in not required",
    unavailable: "Codex authentication unavailable",
    preview: "Native authentication unavailable",
  }[authState];

  async function applyAuthAction(
    action: () => Promise<CodexAuthSnapshot>,
    openBrowser = false,
  ) {
    setAuthBusy(true);
    setAuthActionError(false);
    try {
      const result = await action();
      setAuth(result);
      setAuthState(result.state);
      if (openBrowser && result.state === "login-pending") {
        await openAuthBrowser();
      }
    } catch {
      setAuthActionError(true);
    } finally {
      setAuthBusy(false);
    }
  }

  function beginLogin(method: AuthLoginMethod) {
    void applyAuthAction(() => startAuth(method), true);
  }

  function discardConversationAttachmentDraft() {
    if (
      conversationAttachments.state === "ready" &&
      conversationAttachments.projectId
    ) {
      void cancelConversationAttachmentsTask({
        projectId: conversationAttachments.projectId,
        attachmentIds: conversationAttachments.attachments.map(
          (attachment) => attachment.attachmentId,
        ),
      }).catch(() => {
        // Native expiry/startup cleanup remains the fail-closed fallback.
      });
    }
    setConversationAttachments(scaffoldConversationAttachments);
    setConversationAttachmentActionError(false);
  }

  function discardFilePreview() {
    if (filePreview.state === "ready" && filePreview.openActionId) {
      void cancelFilePreviewTask({
        openActionId: filePreview.openActionId,
      }).catch(() => {
        // Native expiry and bounded eviction remain the fail-closed fallback.
      });
    }
    setFilePreview(scaffoldFilePreview);
    setFilePreviewActionError(false);
  }

  async function applyProjectAction(
    action: () => Promise<ProjectWorkspaceSnapshot>,
  ) {
    setProjectBusy(true);
    setProjectActionError(false);
    try {
      const result = await action();
      setProjects(result);
      setProjectPreflights({});
      if (
        conversationAttachments.state === "ready" &&
        !result.projects.some(
          (project) =>
            project.id === conversationAttachments.projectId &&
            !project.archived &&
            project.directory?.state === "connected-accessible",
        )
      ) {
        discardConversationAttachmentDraft();
      }
      if (
        filePreview.state === "ready" &&
        filePreview.projectId &&
        !result.projects.some(
          (project) =>
            project.id === filePreview.projectId &&
            !project.archived &&
            ["connected-accessible", "connected-read-only"].includes(
              project.directory?.state ?? "",
            ),
        )
      ) {
        discardFilePreview();
      }
    } catch {
      setProjectActionError(true);
    } finally {
      setProjectBusy(false);
    }
  }

  async function verifyProject(projectId: string) {
    setProjectBusy(true);
    setProjectActionError(false);
    try {
      const result = await preflightProjectDirectory(projectId);
      setProjectPreflights((current) => ({ ...current, [projectId]: result }));
    } catch {
      setProjectActionError(true);
    } finally {
      setProjectBusy(false);
    }
  }

  async function refreshWorktrees() {
    if (!currentProject) return;
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    try {
      const result = await loadWorktreesTask(currentProject.id);
      setWorktrees(result);
      setWorktreePreview(null);
      setWorktreeResult(null);
      setWorktreeState("native");
    } catch {
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function beginWorktreeCreate(branchName: string) {
    if (!currentProject) return;
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    setWorktreeResult(null);
    try {
      setWorktreePreview(
        await previewWorktreeCreateTask({
          projectId: currentProject.id,
          branchName,
        }),
      );
    } catch {
      setWorktreePreview(null);
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function beginWorktreeAttach() {
    if (!currentProject) return;
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    setWorktreeResult(null);
    try {
      const preview = await pickWorktreeAttachTask(currentProject.id);
      setWorktreePreview(preview.state === "cancelled" ? null : preview);
    } catch {
      setWorktreePreview(null);
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function beginWorktreeRecover(recoveryId: string) {
    if (!currentProject) return;
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    setWorktreeResult(null);
    try {
      setWorktreePreview(
        await previewWorktreeRecoverTask({
          projectId: currentProject.id,
          recoveryId,
        }),
      );
    } catch {
      setWorktreePreview(null);
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function beginWorktreeRemove(worktreeProjectId: string) {
    if (!currentProject) return;
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    setWorktreeResult(null);
    try {
      setWorktreePreview(
        await previewWorktreeRemoveTask({
          projectId: currentProject.id,
          worktreeProjectId,
        }),
      );
    } catch {
      setWorktreePreview(null);
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function applyWorktree(confirmationId: string) {
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    try {
      const result = await confirmWorktreeTask({ confirmationId });
      setWorktreePreview(null);
      setWorktreeResult(result);
      if (result.workspace) {
        setWorktrees(result.workspace);
        setWorktreeState("native");
      }
      if (result.state === "applied") {
        const projectResult = await loadProjects();
        setProjects(projectResult);
        if (result.projectId) selectProject(result.projectId);
      }
    } catch {
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function cancelWorktreePreview(confirmationId: string) {
    setWorktreeBusy(true);
    setWorktreeActionError(false);
    try {
      await cancelWorktreeTask({ confirmationId });
      setWorktreePreview(null);
    } catch {
      setWorktreeActionError(true);
    } finally {
      setWorktreeBusy(false);
    }
  }

  async function refreshGitReview() {
    if (!currentProject) return;
    setGitBusy(true);
    setGitActionError(false);
    try {
      const result = await loadGitStatusTask(currentProject.id);
      setGitSnapshot(result);
      setGitDiff(null);
      setGitSelectedRequest(null);
      setGitState("native");
      setGitMutationPreview(null);
      setGitMutationResult(null);
    } catch {
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  async function reviewGitDiff(request: GitDiffRequest) {
    setGitBusy(true);
    setGitActionError(false);
    setGitSelectedRequest(request);
    setGitDiff(null);
    try {
      setGitDiff(await loadGitDiffTask(request));
    } catch {
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  async function openReviewedGitFile(projectId: string, path: string) {
    setGitBusy(true);
    setGitActionError(false);
    try {
      await openGitFileTask({ projectId, path });
    } catch {
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  async function beginGitMutation(request: GitMutationPreviewRequest) {
    setGitBusy(true);
    setGitActionError(false);
    setGitMutationResult(null);
    try {
      setGitMutationPreview(await previewGitMutationTask(request));
    } catch {
      setGitMutationPreview(null);
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  async function applyGitMutation(confirmationId: string) {
    setGitBusy(true);
    setGitActionError(false);
    try {
      const result = await confirmGitMutationTask({ confirmationId });
      setGitMutationPreview(null);
      setGitMutationResult(result);
      if (result.workspace) {
        setGitSnapshot(result.workspace);
        setGitDiff(null);
        setGitSelectedRequest(null);
        setGitState("native");
      }
    } catch {
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  async function recoverGitRevert(recoveryId: string) {
    setGitBusy(true);
    setGitActionError(false);
    try {
      const result = await recoverGitMutationTask({ recoveryId });
      setGitMutationResult(result);
      if (result.workspace) {
        setGitSnapshot(result.workspace);
        setGitDiff(null);
        setGitSelectedRequest(null);
      }
    } catch {
      setGitActionError(true);
    } finally {
      setGitBusy(false);
    }
  }

  function trackConversation(
    snapshot: ConversationSnapshot,
    replaceEvents: boolean,
  ) {
    if (!snapshot.projectId || !snapshot.conversationId) return;
    const projectId = snapshot.projectId;
    setTrackedConversations((current) => {
      const previous = current[projectId];
      return {
        ...current,
        [projectId]: {
          snapshot,
          events: replaceEvents
            ? snapshot.events
            : mergeConversationEvents(previous?.events ?? [], snapshot.events),
        },
      };
    });
  }

  function selectProject(projectId: string) {
    if (
      conversationAttachments.state === "ready" &&
      conversationAttachments.projectId &&
      conversationAttachments.projectId !== projectId
    ) {
      discardConversationAttachmentDraft();
    }
    setSelectedProjectId(projectId);
    discardFilePreview();
    const tracked = trackedConversations[projectId];
    setConversation(tracked?.snapshot ?? scaffoldConversation);
    setConversationEvents(tracked?.events ?? []);
    setConversationActionError(false);
  }

  async function chooseFilePreview(projectId: string) {
    setFilePreviewBusy(true);
    setFilePreviewActionError(false);
    try {
      setFilePreview(await pickFilePreviewTask(projectId));
    } catch {
      setFilePreview(scaffoldFilePreview);
      setFilePreviewActionError(true);
    } finally {
      setFilePreviewBusy(false);
    }
  }

  async function openSelectedFilePreview(request: FilePreviewHandoffRequest) {
    setFilePreviewBusy(true);
    setFilePreviewActionError(false);
    try {
      await openFilePreviewTask(request);
    } catch (error) {
      try {
        await cancelFilePreviewTask(request);
      } catch {
        // Native expiry and bounded eviction remain the fail-closed fallback.
      }
      setFilePreview(scaffoldFilePreview);
      setFilePreviewActionError(true);
      throw error;
    } finally {
      setFilePreviewBusy(false);
    }
  }

  async function chooseConversationAttachments(projectId: string) {
    setConversationAttachmentBusy(true);
    setConversationAttachmentActionError(false);
    try {
      const result = await pickConversationAttachmentsTask(projectId);
      if (
        result.state === "unavailable" &&
        conversationAttachments.state === "ready" &&
        conversationAttachments.projectId === projectId
      ) {
        setConversationAttachmentActionError(true);
      } else {
        setConversationAttachments(result);
      }
    } catch (error) {
      setConversationAttachmentActionError(true);
      throw error;
    } finally {
      setConversationAttachmentBusy(false);
    }
  }

  async function stageConversationAttachmentDrop(
    request: ConversationAttachmentDropRequest,
  ) {
    setConversationAttachmentBusy(true);
    setConversationAttachmentActionError(false);
    try {
      const result = await stageDroppedConversationAttachmentsTask(request);
      if (
        result.state === "unavailable" &&
        conversationAttachments.state === "ready" &&
        conversationAttachments.projectId === request.projectId
      ) {
        setConversationAttachmentActionError(true);
      } else {
        setConversationAttachments(result);
      }
    } catch (error) {
      setConversationAttachmentActionError(true);
      throw error;
    } finally {
      setConversationAttachmentBusy(false);
    }
  }

  async function removeConversationAttachment(
    projectId: string,
    attachmentId: string,
  ) {
    setConversationAttachmentBusy(true);
    setConversationAttachmentActionError(false);
    try {
      const result = await cancelConversationAttachmentsTask({
        projectId,
        attachmentIds: [attachmentId],
      });
      if (
        result.state === "unavailable" &&
        conversationAttachments.state === "ready" &&
        conversationAttachments.projectId === projectId
      ) {
        setConversationAttachmentActionError(true);
      } else {
        setConversationAttachments(result);
      }
    } catch (error) {
      setConversationAttachmentActionError(true);
      throw error;
    } finally {
      setConversationAttachmentBusy(false);
    }
  }

  async function beginConversation(
    request: ConversationStartRequest,
  ): Promise<ConversationSnapshot> {
    setConversationBusy(true);
    setConversationActionError(false);
    try {
      const result = await startConversationTask(request);
      setConversationAttachments(scaffoldConversationAttachments);
      setConversationAttachmentActionError(false);
      setConversation(result);
      setConversationEvents(result.events);
      trackConversation(result, true);
      return result;
    } catch (error) {
      setConversationActionError(true);
      throw error;
    } finally {
      setConversationBusy(false);
    }
  }

  async function stopConversation(
    conversationId: string,
  ): Promise<ConversationSnapshot> {
    conversationActionGenerations.current[conversationId] =
      (conversationActionGenerations.current[conversationId] ?? 0) + 1;
    setConversationBusy(true);
    setConversationActionError(false);
    try {
      const result = await interruptConversationTask(conversationId);
      setConversation(result);
      setConversationEvents((current) =>
        mergeConversationEvents(current, result.events),
      );
      trackConversation(result, false);
      return result;
    } catch (error) {
      setConversationActionError(true);
      throw error;
    } finally {
      setConversationBusy(false);
    }
  }

  async function applyConversationApproval(
    request: ConversationApprovalDecisionRequest,
  ): Promise<ConversationSnapshot> {
    conversationActionGenerations.current[request.conversationId] =
      (conversationActionGenerations.current[request.conversationId] ?? 0) + 1;
    setConversationBusy(true);
    setConversationActionError(false);
    try {
      const result = await decideConversationApprovalTask(request);
      setConversation(result);
      setConversationEvents((current) =>
        mergeConversationEvents(current, result.events),
      );
      trackConversation(result, false);
      return result;
    } catch (error) {
      setConversationActionError(true);
      throw error;
    } finally {
      setConversationBusy(false);
    }
  }

  async function refreshSessions(
    request: SessionListRequest = {
      projectId: null,
      searchTerm: sessionSearchTerm,
    },
  ) {
    setSessionBusy(true);
    setSessionActionError(false);
    try {
      const result = await loadSessions(request);
      setSessions(result);
      setSessionSearchTerm(request.searchTerm);
      setSessionState("native");
    } catch (error) {
      setSessionActionError(true);
      throw error;
    } finally {
      setSessionBusy(false);
    }
  }

  async function continueHistoricalConversation(
    action: (
      request: ConversationContinueRequest,
    ) => Promise<ConversationSnapshot>,
    request: ConversationContinueRequest,
  ): Promise<ConversationSnapshot> {
    setConversationBusy(true);
    setSessionBusy(true);
    setConversationActionError(false);
    setSessionActionError(false);
    try {
      const source = sessions.sessions.find(
        (session) => session.conversationId === request.conversationId,
      );
      if (source) selectProject(source.projectId);
      const result = await action(request);
      setConversationAttachments(scaffoldConversationAttachments);
      setConversationAttachmentActionError(false);
      setConversation(result);
      setConversationEvents(result.events);
      trackConversation(result, true);
      if (result.state === "unavailable") setSessionActionError(true);
      return result;
    } catch (error) {
      setConversationActionError(true);
      setSessionActionError(true);
      throw error;
    } finally {
      setConversationBusy(false);
      setSessionBusy(false);
    }
  }

  async function mutateSession(
    action: () => Promise<SessionLifecycleSnapshot>,
  ) {
    setSessionBusy(true);
    setSessionActionError(false);
    try {
      const mutation = await action();
      if (mutation.state === "unavailable") {
        setSessionActionError(true);
        return;
      }
      const result = await loadSessions({
        projectId: null,
        searchTerm: sessionSearchTerm,
      });
      setSessions(result);
    } catch (error) {
      setSessionActionError(true);
      throw error;
    } finally {
      setSessionBusy(false);
    }
  }

  function trackTerminal(snapshot: TerminalSnapshot) {
    if (!snapshot.terminalId) return;
    setTerminals((current) => {
      const reviewed = { ...snapshot, output: [] };
      const index = current.terminals.findIndex(
        (terminal) => terminal.terminalId === snapshot.terminalId,
      );
      const next = [...current.terminals];
      if (index === -1) next.push(reviewed);
      else next[index] = reviewed;
      return { ...current, terminals: next, diagnosticCode: null };
    });
  }

  async function beginTerminal(
    request: TerminalStartRequest,
  ): Promise<TerminalSnapshot> {
    setTerminalBusy(true);
    setTerminalActionError(false);
    try {
      const result = await startTerminalTask(request);
      if (result.state === "unavailable") setTerminalActionError(true);
      else trackTerminal(result);
      return result;
    } catch (error) {
      setTerminalActionError(true);
      throw error;
    } finally {
      setTerminalBusy(false);
    }
  }

  async function pollActiveTerminal(
    request: TerminalPollRequest,
  ): Promise<TerminalSnapshot> {
    try {
      return await pollTerminalTask(request);
    } catch (error) {
      setTerminalActionError(true);
      throw error;
    }
  }

  async function writeActiveTerminal(
    request: TerminalWriteRequest,
  ): Promise<TerminalSnapshot> {
    try {
      return await writeTerminalTask(request);
    } catch (error) {
      setTerminalActionError(true);
      throw error;
    }
  }

  async function resizeActiveTerminal(
    request: TerminalResizeRequest,
  ): Promise<TerminalSnapshot> {
    try {
      return await resizeTerminalTask(request);
    } catch (error) {
      setTerminalActionError(true);
      throw error;
    }
  }

  async function endTerminal(
    request: TerminalCloseRequest,
  ): Promise<TerminalRegistrySnapshot> {
    setTerminalBusy(true);
    setTerminalActionError(false);
    try {
      const result = await closeTerminalTask(request);
      setTerminals(result);
      if (result.diagnosticCode) setTerminalActionError(true);
      return result;
    } catch (error) {
      setTerminalActionError(true);
      throw error;
    } finally {
      setTerminalBusy(false);
    }
  }

  async function refreshIntegrationCatalog() {
    if (
      integrationControlResult?.state === "handoff-ready" ||
      integrationControlResult?.state === "pending"
    )
      return;
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    setIntegrationPreview(null);
    setIntegrationControlPreview(null);
    try {
      const result = await refreshIntegrationCatalogTask();
      setIntegrationCatalog(result);
      setIntegrationResult(null);
      setIntegrationControlResult(null);
      setIntegrationState("native");
    } catch (error) {
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function beginIntegrationMutation(
    request: IntegrationMutationPreviewRequest,
  ) {
    if (
      integrationControlResult?.state === "handoff-ready" ||
      integrationControlResult?.state === "pending"
    )
      return;
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    setIntegrationResult(null);
    setIntegrationControlPreview(null);
    setIntegrationControlResult(null);
    try {
      setIntegrationPreview(await previewIntegrationMutationTask(request));
    } catch (error) {
      setIntegrationPreview(null);
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function applyIntegrationMutation(confirmationId: string) {
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    try {
      const result = await confirmIntegrationMutationTask({ confirmationId });
      setIntegrationPreview(null);
      if (result.state === "applied" && result.catalogRefreshRequired) {
        setIntegrationCatalog(await loadIntegrationCatalogTask());
        setIntegrationState("native");
      }
      setIntegrationResult(result);
    } catch (error) {
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function beginIntegrationControl(
    request: IntegrationControlPreviewRequest,
  ) {
    if (
      integrationControlResult?.state === "handoff-ready" ||
      integrationControlResult?.state === "pending"
    )
      return;
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    setIntegrationPreview(null);
    setIntegrationResult(null);
    setIntegrationControlResult(null);
    try {
      setIntegrationControlPreview(
        await previewIntegrationControlTask(request),
      );
    } catch (error) {
      setIntegrationControlPreview(null);
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function applyIntegrationControl(confirmationId: string) {
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    try {
      const result = await confirmIntegrationControlTask({ confirmationId });
      setIntegrationControlPreview(null);
      if (result.catalogRefreshRequired) {
        setIntegrationCatalog(await loadIntegrationCatalogTask());
        setIntegrationState("native");
      }
      setIntegrationControlResult(result);
    } catch (error) {
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function openIntegrationControl(actionId: string) {
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    try {
      setIntegrationControlResult(
        await openIntegrationControlTask({ actionId }),
      );
    } catch (error) {
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  async function checkIntegrationControl(actionId: string) {
    setIntegrationBusy(true);
    setIntegrationActionError(false);
    try {
      const result = await pollIntegrationControlTask({ actionId });
      if (result.catalogRefreshRequired) {
        setIntegrationCatalog(await loadIntegrationCatalogTask());
        setIntegrationState("native");
      }
      setIntegrationControlResult(result);
    } catch (error) {
      setIntegrationActionError(true);
      throw error;
    } finally {
      setIntegrationBusy(false);
    }
  }

  const currentProject =
    projects.projects.find(
      (project) => project.id === selectedProjectId && !project.archived,
    ) ??
    projects.projects.find((project) => !project.archived) ??
    projects.projects[0];

  const conversationActive = [
    "running",
    "waiting-for-approval",
    "stopping",
  ].includes(conversation.state);
  const visibleWorktreeProjects = new Set(
    worktrees.worktrees.flatMap((worktree) =>
      worktree.projectId ? [worktree.projectId] : [],
    ),
  );
  const worktreeExecutions = Object.values(trackedConversations)
    .flatMap(({ snapshot }) => {
      if (
        !snapshot.projectId ||
        !snapshot.conversationId ||
        !visibleWorktreeProjects.has(snapshot.projectId) ||
        snapshot.state === "empty" ||
        snapshot.state === "unavailable"
      )
        return [];
      const project = projects.projects.find(
        (candidate) => candidate.id === snapshot.projectId,
      );
      const git = taskGitSnapshots[snapshot.projectId];
      const gitReady = git && git.state !== "unavailable";
      return [
        {
          projectId: snapshot.projectId,
          projectName: project?.displayName ?? "Attached worktree",
          conversationId: snapshot.conversationId,
          state: snapshot.state,
          changeCount: gitReady ? git.changes.length : null,
          conflictCount: gitReady
            ? git.changes.filter((change) => change.conflict).length
            : null,
        } satisfies WorktreeExecutionView,
      ];
    })
    .sort((left, right) => left.projectName.localeCompare(right.projectName));

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand-lockup">
          <img src={brandMark} alt="" className="brand-mark" />
          <div>
            <strong>{bootstrap.product.name}</strong>
            <span>Linux workspace</span>
          </div>
        </div>

        <button
          className="primary-action"
          type="button"
          disabled={conversationState !== "native"}
          onClick={() =>
            document.getElementById("conversation")?.scrollIntoView({
              behavior: "smooth",
            })
          }
        >
          <Glyph name="plus" />
          New thread
        </button>

        <nav className="primary-nav" aria-label="Primary navigation">
          <p className="nav-label">Workbench</p>
          {navigation.map((item, index) => (
            <button
              className={`nav-item ${index === 0 ? "nav-item--active" : ""}`}
              type="button"
              aria-current={index === 0 ? "page" : undefined}
              disabled={!item.ready}
              onClick={() =>
                document.getElementById(item.target)?.scrollIntoView({
                  behavior: "smooth",
                })
              }
              key={item.label}
            >
              <Glyph name={item.icon} />
              <span>{item.label}</span>
              {index !== 0 && (
                <span className="nav-milestone">M{item.milestone}</span>
              )}
            </button>
          ))}
        </nav>

        <div className="project-panel">
          <div className="project-icon">
            <Glyph name="folder" />
          </div>
          <div>
            <strong>
              {currentProject?.displayName ?? "No project attached"}
            </strong>
            <span>
              {currentProject?.directory?.displayPath ??
                (projectState === "preview"
                  ? "Native project access unavailable in browser preview."
                  : "Attach an original local directory in place.")}
            </span>
          </div>
        </div>

        <div className="sidebar-footer">
          <div className="bridge-status" role="status" aria-live="polite">
            <StatusDot state={bridgeState} />
            <span>{bridgeLabel}</span>
          </div>
          <span className="version">v{bootstrap.product.version}</span>
        </div>
      </aside>

      <main className="workspace" id="workspace-top">
        <header className="topbar">
          <div className="breadcrumb" aria-label="Current location">
            <span>QuireForge</span>
            <Glyph name="chevron" />
            <strong>Workspace</strong>
          </div>
          <div className="topbar-actions">
            <span className="foundation-badge">
              <Glyph name="shield" />
              Milestone 12 integrated terminal
            </span>
            <button
              className="theme-toggle"
              type="button"
              aria-label={`Use ${theme === "dark" ? "light" : "dark"} theme`}
              onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            >
              <span className="theme-toggle__track" aria-hidden="true">
                <span className="theme-toggle__thumb" />
              </span>
            </button>
          </div>
        </header>

        <div className="workspace-scroll">
          <section className="hero" aria-labelledby="workspace-title">
            <div className="hero-copy">
              <p className="eyebrow">
                <span /> Native Linux foundation
              </p>
              <h1 id="workspace-title">A quiet place for ambitious work.</h1>
              <p className="hero-description">
                Attach an original directory without copying it into QuireForge.
                Selected and resolved paths are reviewed before app-owned
                metadata is saved.
              </p>
              <div className="hero-actions">
                <button
                  className="secondary-action"
                  type="button"
                  disabled={
                    projectState !== "native" ||
                    projects.state === "unavailable" ||
                    projectBusy
                  }
                  onClick={() => void applyProjectAction(pickProject)}
                >
                  <Glyph name="folder" />
                  Attach a local project
                  <span>Native picker</span>
                </button>
                <a className="text-link" href="#foundation">
                  Inspect foundation
                  <Glyph name="chevron" />
                </a>
              </div>
            </div>

            <div
              className="hero-visual"
              aria-label="QuireForge foundation status"
            >
              <div className="visual-glow" />
              <div className="terminal-card">
                <div className="terminal-card__bar">
                  <div className="window-dots" aria-hidden="true">
                    <span />
                    <span />
                    <span />
                  </div>
                  <span>quireforge / foundation</span>
                  <Glyph name="terminal" />
                </div>
                <div className="terminal-card__body">
                  <p>
                    <span className="prompt">›</span> verify desktop boundary
                  </p>
                  <div className="verification-line">
                    <Glyph name="check" />
                    <div>
                      <strong>Identity contract</strong>
                      <span>io.github.codeframe78.QuireForge</span>
                    </div>
                    <em>verified</em>
                  </div>
                  <div className="verification-line">
                    <Glyph name="check" />
                    <div>
                      <strong>Typed IPC fixture</strong>
                      <span>desktop_bootstrap · schema v1</span>
                    </div>
                    <em>verified</em>
                  </div>
                  <div
                    className={`verification-line ${runtimeState === "ready" ? "" : "verification-line--planned"}`}
                  >
                    {runtimeState === "ready" ? (
                      <Glyph name="check" />
                    ) : (
                      <span className="planned-ring" />
                    )}
                    <div>
                      <strong>Codex process adapter</strong>
                      <span>
                        {runtimeState === "ready"
                          ? `${runtime.adapterVersion} · ${runtime.models.length} models`
                          : "Supported native interfaces only"}
                      </span>
                    </div>
                    <em>{runtimeLabel}</em>
                  </div>
                </div>
              </div>
            </div>
          </section>

          <ProjectWorkspace
            availability={projectState}
            snapshot={projects}
            busy={projectBusy}
            actionError={projectActionError}
            preflights={projectPreflights}
            onPick={() => applyProjectAction(pickProject)}
            onPickRelink={(projectId) =>
              applyProjectAction(() => pickRelink(projectId))
            }
            onConfirm={() => applyProjectAction(confirmProject)}
            onCancel={() => applyProjectAction(cancelProject)}
            onDetach={(projectId) =>
              applyProjectAction(() => detachProjectDirectory(projectId))
            }
            onArchive={(projectId) =>
              applyProjectAction(() => archiveProjectMetadata(projectId))
            }
            onPreflight={verifyProject}
          />

          <FilePreviewWorkspace
            availability={projectState}
            project={currentProject}
            snapshot={filePreview}
            busy={filePreviewBusy}
            actionError={filePreviewActionError}
            onPick={chooseFilePreview}
            onOpen={openSelectedFilePreview}
            onClear={discardFilePreview}
          />

          <WorktreeWorkspace
            availability={worktreeState}
            projectName={currentProject?.displayName ?? null}
            snapshot={worktrees}
            preview={worktreePreview}
            result={worktreeResult}
            busy={worktreeBusy || conversationActive || gitBusy}
            selectionBusy={worktreeBusy}
            actionError={worktreeActionError}
            executions={worktreeExecutions}
            onRefresh={refreshWorktrees}
            onCreate={beginWorktreeCreate}
            onPickAttach={beginWorktreeAttach}
            onRecover={beginWorktreeRecover}
            onRemove={beginWorktreeRemove}
            onConfirm={applyWorktree}
            onCancel={cancelWorktreePreview}
            onSelectProject={selectProject}
            onOpenExecution={(projectId) => {
              selectProject(projectId);
              window.setTimeout(
                () =>
                  document.getElementById("conversation")?.scrollIntoView({
                    behavior: "smooth",
                  }),
                0,
              );
            }}
          />

          <TerminalWorkspace
            availability={terminalState}
            registry={terminals}
            projects={projects}
            busy={terminalBusy}
            actionError={terminalActionError}
            onStart={beginTerminal}
            onPoll={pollActiveTerminal}
            onWrite={writeActiveTerminal}
            onResize={resizeActiveTerminal}
            onClose={endTerminal}
            onSnapshot={trackTerminal}
          />

          <GitWorkspace
            availability={gitState}
            projectName={currentProject?.displayName ?? null}
            snapshot={gitSnapshot}
            diff={gitDiff}
            selectedRequest={gitSelectedRequest}
            mutationPreview={gitMutationPreview}
            mutationResult={gitMutationResult}
            busy={gitBusy || conversationActive}
            actionError={gitActionError}
            onRefresh={refreshGitReview}
            onReview={reviewGitDiff}
            onOpen={openReviewedGitFile}
            onPreviewMutation={beginGitMutation}
            onConfirmMutation={applyGitMutation}
            onCancelMutation={() => setGitMutationPreview(null)}
            onRecoverMutation={recoverGitRevert}
          />

          <SessionWorkspace
            availability={sessionState}
            snapshot={sessions}
            projects={projects.projects}
            activeConversationId={conversation.conversationId}
            attachments={conversationAttachments}
            busy={sessionBusy || conversationBusy || conversationActive}
            attachmentBusy={conversationAttachmentBusy}
            actionError={sessionActionError}
            attachmentActionError={conversationAttachmentActionError}
            searchTerm={sessionSearchTerm}
            onSearch={refreshSessions}
            onRefresh={() => refreshSessions()}
            onSelect={(session) => selectProject(session.projectId)}
            onResume={(request) =>
              continueHistoricalConversation(resumeConversationTask, request)
            }
            onFork={(request) =>
              continueHistoricalConversation(forkConversationTask, request)
            }
            onArchive={(conversationId) =>
              mutateSession(() => archiveConversationTask(conversationId))
            }
            onRestore={(conversationId) =>
              mutateSession(() => restoreConversationTask(conversationId))
            }
            onAttachmentPick={chooseConversationAttachments}
            onAttachmentDrop={stageConversationAttachmentDrop}
            onAttachmentCancel={removeConversationAttachment}
          />

          <IntegrationCenter
            availability={integrationState}
            snapshot={integrationCatalog}
            preview={integrationPreview}
            result={integrationResult}
            controlPreview={integrationControlPreview}
            controlResult={integrationControlResult}
            busy={integrationBusy}
            actionError={integrationActionError}
            onRefresh={refreshIntegrationCatalog}
            onPreview={beginIntegrationMutation}
            onConfirm={applyIntegrationMutation}
            onControlPreview={beginIntegrationControl}
            onControlConfirm={applyIntegrationControl}
            onControlOpen={openIntegrationControl}
            onControlPoll={checkIntegrationControl}
            onCancel={() => {
              setIntegrationPreview(null);
              setIntegrationControlPreview(null);
            }}
          />

          <ConversationWorkspace
            availability={conversationState}
            snapshot={conversation}
            events={conversationEvents}
            runtime={runtime}
            project={currentProject}
            integrations={integrationCatalog}
            attachments={conversationAttachments}
            busy={conversationBusy}
            attachmentBusy={conversationAttachmentBusy}
            actionError={conversationActionError}
            attachmentActionError={conversationAttachmentActionError}
            onStart={beginConversation}
            onInterrupt={stopConversation}
            onDecideApproval={applyConversationApproval}
            onAttachmentPick={chooseConversationAttachments}
            onAttachmentDrop={stageConversationAttachmentDrop}
            onAttachmentCancel={removeConversationAttachment}
          />

          <section className="auth-onboarding" aria-labelledby="auth-title">
            <div className="auth-onboarding__intro">
              <p className="eyebrow">Codex account</p>
              <h2 id="auth-title">Authentication stays with Codex.</h2>
              <p>
                QuireForge receives only a bounded connection state. Email,
                tokens, account identifiers, raw errors, and completed sign-in
                URLs do not enter application state or logs.
              </p>
            </div>

            <div className="auth-card" aria-live="polite">
              <div className="auth-card__heading">
                <span
                  className={"auth-state auth-state--" + authState}
                  aria-hidden="true"
                />
                <div>
                  <strong>{authLabel}</strong>
                  <span>Codex CLI {runtime.cliVersion ?? "not detected"}</span>
                </div>
              </div>

              {authState === "checking" && (
                <p className="auth-card__copy">
                  Reading a normalized account status from the local Codex
                  runtime.
                </p>
              )}

              {authState === "preview" && (
                <p className="auth-card__copy">
                  Browser preview cannot inspect or simulate a native Codex
                  account.
                </p>
              )}

              {authState === "authenticated" && (
                <>
                  <p className="auth-card__copy">
                    Connected using{" "}
                    {auth.accountKind === "chatgpt"
                      ? "Codex-managed ChatGPT authentication"
                      : "authentication already owned by Codex"}
                    . No account identity is displayed or retained.
                  </p>
                  <div className="auth-actions">
                    <button
                      className="auth-button auth-button--quiet"
                      type="button"
                      disabled={authBusy}
                      onClick={() => void applyAuthAction(refreshAuth)}
                    >
                      <Glyph name="refresh" />
                      Refresh status
                    </button>
                    {!confirmLogout ? (
                      <button
                        className="auth-button auth-button--danger"
                        type="button"
                        disabled={authBusy}
                        onClick={() => setConfirmLogout(true)}
                      >
                        Sign out of Codex
                      </button>
                    ) : (
                      <div
                        className="logout-confirmation"
                        role="group"
                        aria-label="Confirm Codex sign out"
                      >
                        <button
                          className="auth-button auth-button--danger"
                          type="button"
                          disabled={authBusy}
                          onClick={() => {
                            setConfirmLogout(false);
                            void applyAuthAction(logoutAuth);
                          }}
                        >
                          Confirm sign out
                        </button>
                        <button
                          className="auth-button auth-button--quiet"
                          type="button"
                          disabled={authBusy}
                          onClick={() => setConfirmLogout(false)}
                        >
                          Keep signed in
                        </button>
                      </div>
                    )}
                  </div>
                </>
              )}

              {authState === "unauthenticated" && (
                <>
                  <p className="auth-card__copy">
                    Continue in your browser or use an official device code.
                    Codex hosts the callback and owns the resulting session.
                  </p>
                  <div className="auth-actions">
                    <button
                      className="auth-button auth-button--primary"
                      type="button"
                      disabled={authBusy}
                      onClick={() => beginLogin("browser")}
                    >
                      <Glyph name="external" />
                      Continue in browser
                    </button>
                    <button
                      className="auth-button auth-button--quiet"
                      type="button"
                      disabled={authBusy}
                      onClick={() => beginLogin("device-code")}
                    >
                      Use a device code
                    </button>
                    <button
                      className="auth-button auth-button--quiet"
                      type="button"
                      disabled={authBusy}
                      onClick={() => void applyAuthAction(refreshAuth)}
                    >
                      <Glyph name="refresh" />
                      Refresh
                    </button>
                  </div>
                </>
              )}

              {authState === "login-pending" && auth.handoff && (
                <>
                  <p className="auth-card__copy">
                    Complete the official Codex sign-in page. This short-lived
                    handoff is cleared after completion or cancellation.
                  </p>
                  {auth.handoff.userCode && (
                    <div className="device-code">
                      <span>One-time device code</span>
                      <code>{auth.handoff.userCode}</code>
                    </div>
                  )}
                  <div className="auth-actions">
                    <button
                      className="auth-button auth-button--primary"
                      type="button"
                      disabled={authBusy}
                      onClick={() => {
                        setAuthActionError(false);
                        void openAuthBrowser().catch(() =>
                          setAuthActionError(true),
                        );
                      }}
                    >
                      <Glyph name="external" />
                      Open sign-in page
                    </button>
                    <button
                      className="auth-button auth-button--quiet"
                      type="button"
                      disabled={authBusy}
                      onClick={() => void applyAuthAction(cancelAuth)}
                    >
                      Cancel sign-in
                    </button>
                  </div>
                </>
              )}

              {authState === "not-required" && (
                <p className="auth-card__copy">
                  The selected Codex provider does not require OpenAI account
                  authentication. QuireForge will continue to defer credential
                  ownership to Codex.
                </p>
              )}

              {authState === "unavailable" && (
                <>
                  <p className="auth-card__copy">
                    Authentication could not be verified safely. No raw Codex
                    error or account metadata was retained.
                  </p>
                  <button
                    className="auth-button auth-button--quiet"
                    type="button"
                    disabled={authBusy}
                    onClick={() => void applyAuthAction(refreshAuth)}
                  >
                    <Glyph name="refresh" />
                    Try again
                  </button>
                </>
              )}

              {authActionError && (
                <p className="auth-error" role="alert">
                  The native authentication action did not complete. Your Codex
                  credentials were not changed by QuireForge.
                </p>
              )}
            </div>
          </section>

          <section
            className="foundation"
            id="foundation"
            aria-labelledby="foundation-title"
          >
            <div className="section-heading">
              <div>
                <p className="eyebrow">Implementation map</p>
                <h2 id="foundation-title">Foundation, with honest edges.</h2>
              </div>
              <p>
                Each surface reports what exists now and what remains planned.
                Nothing here fabricates a Codex session or integration.
              </p>
            </div>

            <div className="capability-grid">
              {bootstrap.capabilities.map((capability, index) => (
                <article className="capability-card" key={capability.id}>
                  <div className="capability-card__top">
                    <span
                      className={`capability-number capability-number--${index}`}
                    >
                      0{index + 1}
                    </span>
                    <span
                      className={`state-badge state-badge--${capability.state}`}
                    >
                      {capability.state}
                    </span>
                  </div>
                  <h3>{capability.label}</h3>
                  <p>
                    {capability.id === "codex-runtime"
                      ? "Version probing, supervised stdio, normalized models, and bounded failure states."
                      : capability.id === "codex-auth"
                        ? "Codex-owned browser and device login with bounded state, cancellation, and redaction."
                        : capability.id === "project-attachments"
                          ? "Native selection, explicit confirmation, durable identity, and fail-closed cwd preflight."
                          : capability.id === "conversation-runtime"
                            ? "Verified-cwd thread startup, normalized streaming, exact interruption, and reference-only persistence."
                            : capability.state === "ready"
                              ? "Tauri, React, strict TypeScript, and a validated native contract."
                              : "Explicit directory selection, identity verification, and in-place local work."}
                  </p>
                  <footer>
                    <span>Milestone {capability.milestone}</span>
                    <Glyph
                      name={capability.state === "ready" ? "check" : "chevron"}
                    />
                  </footer>
                </article>
              ))}
            </div>
          </section>

          <section className="boundary-note" aria-label="Security boundary">
            <div className="boundary-icon">
              <Glyph name="shield" />
            </div>
            <div>
              <strong>Local by design. Narrow by default.</strong>
              <p>
                The frontend cannot spawn arbitrary processes or read arbitrary
                files. QuireForge metadata stays separate from Codex
                credentials, configuration, sessions, and connector
                authorization.
              </p>
            </div>
            <span>{runtimeLabel}</span>
          </section>

          <footer className="product-footer">
            <span>{bootstrap.product.tagline}</span>
            <p>
              QuireForge is an unofficial community project. It is not made,
              endorsed, supported, or distributed by OpenAI.
            </p>
          </footer>
        </div>
      </main>
    </div>
  );
}
