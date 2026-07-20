import { useEffect, useRef, useState, type ReactNode } from "react";

import brandMark from "../../../assets/brand/quireforge-app-icon.svg";
import { ConversationWorkspace } from "./ConversationWorkspace";
import { GitWorkspace } from "./GitWorkspace";
import { ProjectWorkspace } from "./ProjectWorkspace";
import { SessionWorkspace } from "./SessionWorkspace";
import {
  archiveConversation,
  archiveProject,
  cancelCodexAuth,
  cancelProjectAttachment,
  confirmProjectAttachment,
  decideConversationApproval,
  detachProject,
  interruptConversation,
  loadCodexAuth,
  loadConversationStatus,
  loadConversationSessions,
  loadCodexRuntime,
  loadDesktopBootstrap,
  loadGitDiff,
  loadGitStatus,
  loadProjectWorkspace,
  logoutCodexAuth,
  openGitFile,
  openCodexAuthBrowser,
  pickProjectDirectory,
  pickProjectRelink,
  preflightProject,
  pollConversation,
  refreshCodexAuth,
  restoreConversation,
  resumeConversation,
  forkConversation,
  startConversation,
  startCodexAuth,
} from "./lib/bridge";
import {
  scaffoldCodexAuth,
  type AuthLoginMethod,
  type CodexAuthSnapshot,
} from "./lib/auth";
import { scaffoldCodexRuntime, type CodexRuntimeSnapshot } from "./lib/codex";
import { scaffoldBootstrap, type DesktopBootstrap } from "./lib/contract";
import {
  scaffoldConversation,
  type ConversationApprovalDecisionRequest,
  type ConversationEvent,
  type ConversationSnapshot,
  type ConversationStartRequest,
} from "./lib/conversation";
import { mergeConversationEvents } from "./lib/conversationView";
import {
  scaffoldGitWorkspace,
  type GitDiffRequest,
  type GitDiffSnapshot,
  type GitOpenFileRequest,
  type GitWorkspaceSnapshot,
} from "./lib/git";
import {
  scaffoldProjectWorkspace,
  type ProjectPreflightSnapshot,
  type ProjectWorkspaceSnapshot,
} from "./lib/project";
import {
  scaffoldSessionLifecycle,
  type ConversationContinueRequest,
  type SessionLifecycleSnapshot,
  type SessionListRequest,
} from "./lib/session";

import "./styles.css";

type BridgeState = "connecting" | "native" | "preview";
type RuntimeState =
  "checking" | "ready" | "degraded" | "unavailable" | "preview";
type AuthViewState = CodexAuthSnapshot["state"] | "checking" | "preview";
type ProjectViewState = "checking" | "native" | "preview";
type GitViewState = "checking" | "native" | "preview";
type ConversationViewState = "checking" | "native" | "preview";
type SessionViewState = "checking" | "native" | "preview";
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
  loadGitStatusTask?: (projectId: string) => Promise<GitWorkspaceSnapshot>;
  loadGitDiffTask?: (request: GitDiffRequest) => Promise<GitDiffSnapshot>;
  openGitFileTask?: (request: GitOpenFileRequest) => Promise<void>;
  loadConversation?: () => Promise<ConversationSnapshot>;
  startConversationTask?: (
    request: ConversationStartRequest,
  ) => Promise<ConversationSnapshot>;
  pollConversationTask?: (
    conversationId: string,
  ) => Promise<ConversationSnapshot>;
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
    label: "Changes",
    milestone: 10,
    icon: "git",
    target: "changes",
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
    target: "",
    ready: false,
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
  loadGitStatusTask = loadGitStatus,
  loadGitDiffTask = loadGitDiff,
  openGitFileTask = openGitFile,
  loadConversation = loadConversationStatus,
  startConversationTask = startConversation,
  pollConversationTask = pollConversation,
  interruptConversationTask = interruptConversation,
  decideConversationApprovalTask = decideConversationApproval,
  loadSessions = loadConversationSessions,
  resumeConversationTask = resumeConversation,
  forkConversationTask = forkConversation,
  archiveConversationTask = archiveConversation,
  restoreConversationTask = restoreConversation,
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
  const [gitSnapshot, setGitSnapshot] =
    useState<GitWorkspaceSnapshot>(scaffoldGitWorkspace);
  const [gitDiff, setGitDiff] = useState<GitDiffSnapshot | null>(null);
  const [gitSelectedRequest, setGitSelectedRequest] =
    useState<GitDiffRequest | null>(null);
  const [gitState, setGitState] = useState<GitViewState>("checking");
  const [gitBusy, setGitBusy] = useState(false);
  const [gitActionError, setGitActionError] = useState(false);
  const [conversation, setConversation] =
    useState<ConversationSnapshot>(scaffoldConversation);
  const [conversationEvents, setConversationEvents] = useState<
    ConversationEvent[]
  >([]);
  const [conversationState, setConversationState] =
    useState<ConversationViewState>("checking");
  const [conversationBusy, setConversationBusy] = useState(false);
  const [conversationActionError, setConversationActionError] = useState(false);
  const conversationActionGeneration = useRef(0);
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

  useEffect(() => {
    if (
      conversationBusy ||
      !conversation.conversationId ||
      !["running", "waiting-for-approval", "stopping"].includes(
        conversation.state,
      )
    ) {
      return;
    }

    let active = true;
    let timer: number | undefined;
    const conversationId = conversation.conversationId;
    const actionGeneration = conversationActionGeneration.current;

    async function poll() {
      try {
        const result = await pollConversationTask(conversationId);
        if (
          !active ||
          actionGeneration !== conversationActionGeneration.current
        )
          return;
        setConversation(result);
        setConversationEvents((current) =>
          mergeConversationEvents(current, result.events),
        );
        if (
          ["running", "waiting-for-approval", "stopping"].includes(result.state)
        ) {
          timer = window.setTimeout(() => void poll(), 250);
        } else {
          void loadSessions({
            projectId: null,
            searchTerm: sessionSearchTerm,
          })
            .then((sessionResult) => setSessions(sessionResult))
            .catch(() => setSessionActionError(true));
        }
      } catch {
        if (active) setConversationActionError(true);
      }
    }

    timer = window.setTimeout(() => void poll(), 250);
    return () => {
      active = false;
      if (timer !== undefined) window.clearTimeout(timer);
    };
  }, [
    conversation.conversationId,
    conversation.state,
    conversationBusy,
    loadSessions,
    pollConversationTask,
    sessionSearchTerm,
  ]);

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

  async function applyProjectAction(
    action: () => Promise<ProjectWorkspaceSnapshot>,
  ) {
    setProjectBusy(true);
    setProjectActionError(false);
    try {
      const result = await action();
      setProjects(result);
      setProjectPreflights({});
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

  async function beginConversation(
    request: ConversationStartRequest,
  ): Promise<ConversationSnapshot> {
    setConversationBusy(true);
    setConversationActionError(false);
    try {
      const result = await startConversationTask(request);
      setConversation(result);
      setConversationEvents(result.events);
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
    setConversationBusy(true);
    setConversationActionError(false);
    try {
      const result = await interruptConversationTask(conversationId);
      setConversation(result);
      setConversationEvents((current) =>
        mergeConversationEvents(current, result.events),
      );
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
    conversationActionGeneration.current += 1;
    setConversationBusy(true);
    setConversationActionError(false);
    try {
      const result = await decideConversationApprovalTask(request);
      setConversation(result);
      setConversationEvents((current) =>
        mergeConversationEvents(current, result.events),
      );
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
      if (source) setSelectedProjectId(source.projectId);
      const result = await action(request);
      setConversation(result);
      setConversationEvents(result.events);
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
              Milestone 10 read-only source review
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

          <GitWorkspace
            availability={gitState}
            projectName={currentProject?.displayName ?? null}
            snapshot={gitSnapshot}
            diff={gitDiff}
            selectedRequest={gitSelectedRequest}
            busy={gitBusy}
            actionError={gitActionError}
            onRefresh={refreshGitReview}
            onReview={reviewGitDiff}
            onOpen={openReviewedGitFile}
          />

          <SessionWorkspace
            availability={sessionState}
            snapshot={sessions}
            projects={projects.projects}
            activeConversationId={conversation.conversationId}
            busy={sessionBusy || conversationBusy || conversationActive}
            actionError={sessionActionError}
            searchTerm={sessionSearchTerm}
            onSearch={refreshSessions}
            onRefresh={() => refreshSessions()}
            onSelect={(session) => setSelectedProjectId(session.projectId)}
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
          />

          <ConversationWorkspace
            availability={conversationState}
            snapshot={conversation}
            events={conversationEvents}
            runtime={runtime}
            project={currentProject}
            busy={conversationBusy}
            actionError={conversationActionError}
            onStart={beginConversation}
            onInterrupt={stopConversation}
            onDecideApproval={applyConversationApproval}
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
