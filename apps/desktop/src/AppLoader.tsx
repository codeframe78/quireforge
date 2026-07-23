import { lazy, Suspense, useCallback, useEffect, useState } from "react";

const App = lazy(() => import("./App"));

function AppLoadingView() {
  return (
    <div
      className="crash-recovery app-loading-overlay"
      role="status"
      aria-labelledby="app-loading-title"
      aria-busy="true"
      aria-live="polite"
    >
      <div className="crash-recovery__card">
        <p className="eyebrow">Native workspace</p>
        <h1 id="app-loading-title">Preparing QuireForge.</h1>
        <p>Loading the local workspace interface.</p>
      </div>
    </div>
  );
}

function ReadySignal({ onReady }: { onReady: () => void }) {
  useEffect(() => {
    let secondFrame = 0;
    const firstFrame = window.requestAnimationFrame(() => {
      secondFrame = window.requestAnimationFrame(onReady);
    });
    return () => {
      window.cancelAnimationFrame(firstFrame);
      window.cancelAnimationFrame(secondFrame);
    };
  }, [onReady]);

  return null;
}

export function AppLoader() {
  const [ready, setReady] = useState(false);
  const markReady = useCallback(() => setReady(true), []);

  return (
    <Suspense fallback={<AppLoadingView />}>
      <App />
      <ReadySignal onReady={markReady} />
      {!ready && <AppLoadingView />}
    </Suspense>
  );
}
