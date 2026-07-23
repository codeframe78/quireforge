import { Component, type ReactNode } from "react";

interface AppCrashBoundaryProps {
  children: ReactNode;
  onReload?: () => void;
}

interface AppCrashBoundaryState {
  failed: boolean;
}

export class AppCrashBoundary extends Component<
  AppCrashBoundaryProps,
  AppCrashBoundaryState
> {
  override state: AppCrashBoundaryState = { failed: false };

  static getDerivedStateFromError(): AppCrashBoundaryState {
    return { failed: true };
  }

  override render(): ReactNode {
    if (!this.state.failed) return this.props.children;

    return (
      <main
        className="crash-recovery"
        role="alert"
        aria-labelledby="crash-recovery-title"
      >
        <div className="crash-recovery__card">
          <p className="eyebrow">Safe recovery</p>
          <h1 id="crash-recovery-title">QuireForge needs a fresh view.</h1>
          <p>
            The interface stopped unexpectedly. Raw diagnostics were not shown
            or retained here. Reloading reconciles app-owned task and terminal
            metadata without deleting project files or Codex history.
          </p>
          <button
            className="auth-button auth-button--primary"
            type="button"
            onClick={this.props.onReload ?? (() => window.location.reload())}
          >
            Reload workspace
          </button>
        </div>
      </main>
    );
  }
}
