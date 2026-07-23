import type { ProjectWorkspaceSnapshot } from "./lib/project";
import type { SessionLifecycleSnapshot } from "./lib/session";
import type { CodexUsageSnapshot } from "./lib/usage";
import { UsagePanel } from "./UsagePanel";

interface HomeDashboardProps {
  projects: ProjectWorkspaceSnapshot;
  sessions: SessionLifecycleSnapshot;
  usage: CodexUsageSnapshot;
  usageState: "checking" | "native" | "preview";
  usageBusy: boolean;
  onRefreshUsage: () => void;
  onNewTask: () => void;
  onAttachProject: () => void;
  onOpenProjects: () => void;
  onOpenSessions: () => void;
  onOpenIntegrations: () => void;
  onOpenTerminal: () => void;
}

function sessionUpdatedLabel(updatedAtMs: number): string {
  const elapsed = Math.max(0, Date.now() - updatedAtMs);
  const minutes = Math.floor(elapsed / 60_000);
  if (minutes < 1) return "Just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

export function HomeDashboard({
  projects,
  sessions,
  usage,
  usageState,
  usageBusy,
  onRefreshUsage,
  onNewTask,
  onAttachProject,
  onOpenProjects,
  onOpenSessions,
  onOpenIntegrations,
  onOpenTerminal,
}: HomeDashboardProps) {
  const visibleProjects = projects.projects
    .filter((project) => !project.archived)
    .slice(0, 3);
  const recentSessions = [...sessions.sessions]
    .sort((left, right) => right.updatedAtMs - left.updatedAtMs)
    .slice(0, 5);

  return (
    <section className="home-dashboard" id="home" aria-labelledby="home-title">
      <div className="home-dashboard__main">
        <div className="home-welcome">
          <p className="eyebrow">QuireForge home</p>
          <h1 id="home-title">What should we build today?</h1>
          <p>
            Start a focused Codex task inside a verified local project. Your
            files stay where they are, and every execution uses the project’s
            reviewed working directory.
          </p>
        </div>

        <button className="home-composer" type="button" onClick={onNewTask}>
          <span>Describe a change, investigation, or review…</span>
          <strong>New task</strong>
        </button>

        <div className="home-section-heading">
          <h2>Projects</h2>
          <button type="button" onClick={onOpenProjects}>
            View all
          </button>
        </div>

        <div className="home-projects">
          {visibleProjects.map((project) => (
            <button type="button" onClick={onOpenProjects} key={project.id}>
              <span aria-hidden="true">⌑</span>
              <strong>{project.displayName}</strong>
              <small>
                {project.directory?.state === "connected-accessible"
                  ? "Ready for local work"
                  : "Needs attention"}
              </small>
              <em>
                {project.directory?.git.isRepository
                  ? "Git project"
                  : "Local project"}
              </em>
            </button>
          ))}
          {visibleProjects.length === 0 && (
            <button type="button" onClick={onAttachProject}>
              <span aria-hidden="true">+</span>
              <strong>Attach your first project</strong>
              <small>Choose an existing local directory</small>
              <em>Native picker</em>
            </button>
          )}
        </div>

        <div className="home-section-heading">
          <h2>Quick actions</h2>
        </div>
        <div className="home-actions">
          <button type="button" onClick={onAttachProject}>
            <span aria-hidden="true">＋</span>
            <strong>Attach project</strong>
            <small>Work in an existing folder</small>
          </button>
          <button type="button" onClick={onOpenSessions}>
            <span aria-hidden="true">◌</span>
            <strong>Resume a thread</strong>
            <small>Continue recent local work</small>
          </button>
          <button type="button" onClick={onOpenIntegrations}>
            <span aria-hidden="true">⌘</span>
            <strong>Integrations</strong>
            <small>Review connected tools</small>
          </button>
          <button type="button" onClick={onOpenTerminal}>
            <span aria-hidden="true">›_</span>
            <strong>Open terminal</strong>
            <small>Start in a verified project</small>
          </button>
        </div>
      </div>

      <aside className="home-dashboard__rail" aria-label="Workspace overview">
        <section className="recent-card" aria-labelledby="recent-title">
          <div>
            <h2 id="recent-title">Recent threads</h2>
            {recentSessions.length > 0 && (
              <button type="button" onClick={onOpenSessions}>
                View all
              </button>
            )}
          </div>
          {recentSessions.length ? (
            <ul>
              {recentSessions.map((session) => (
                <li key={session.conversationId}>
                  <button type="button" onClick={onOpenSessions}>
                    <strong>{session.title ?? "Untitled Codex task"}</strong>
                    <span>{sessionUpdatedLabel(session.updatedAtMs)}</span>
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <p>Your recent Codex threads will appear here.</p>
          )}
        </section>

        <UsagePanel
          snapshot={usage}
          state={usageState}
          busy={usageBusy}
          onRefresh={onRefreshUsage}
        />
      </aside>
    </section>
  );
}
