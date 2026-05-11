import { useState, useEffect, useCallback, useMemo, lazy, Suspense } from "react";
import { useGatewayStore } from "../../lib/store";
import { useAgentStore } from "../../lib/agent-store";
import { AgentList } from "../agent-list/AgentList";
import { MessageStream } from "../message-stream/MessageStream";
import { TitleBar } from "./TitleBar";
import { ClawIcon } from "./ClawIcon";
import { UpdateBanner } from "./UpdateBanner";
import * as api from "../../lib/api";

const isTauri =
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

const OnboardingWizard = lazy(() =>
  import("../onboarding/OnboardingWizard").then((m) => ({ default: m.OnboardingWizard })),
);
const AgentDetail = lazy(() =>
  import("../agent-detail/AgentDetail").then((m) => ({ default: m.AgentDetail })),
);

function SkeletonPulse({ className = "", style = {} }: { className?: string; style?: React.CSSProperties }) {
  return (
    <div
      className={`rounded-md ${className}`}
      style={{
        background: "var(--bg-tertiary)",
        animation: "pulse-subtle 1.5s ease-in-out infinite",
        ...style,
      }}
    />
  );
}

function Loading({ error }: { error: string | null }) {
  if (error) {
    return (
      <div className="flex h-full flex-col items-center justify-center" style={{ background: "var(--bg-primary)" }}>
        <div style={{ animation: "scale-in var(--duration-slow) var(--ease-out)" }} className="text-center">
          <div className="mx-auto mb-5"><ClawIcon size={64} /></div>
          <p className="text-[15px] font-semibold tracking-[-0.01em]" style={{ color: "var(--fill-primary)" }}>FastClaw</p>
          <p className="mt-1.5 text-[13px]" style={{ color: "var(--red)" }}>连接失败: {error}</p>
          <button
            onClick={() => window.location.reload()}
            className="mt-4 cursor-pointer rounded-[var(--radius-xs)] px-4 py-1.5 text-[12px] font-medium transition-colors duration-150 hover:opacity-80 active:scale-[0.97]"
            style={{ background: "var(--tint)", color: "#fff" }}
          >
            重试连接
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col" style={{ background: "var(--bg-primary)", animation: "fade-in var(--duration-slow) var(--ease-out)" }}>
      <div className="flex h-[var(--titlebar-h)] shrink-0 items-center gap-2 px-4" style={{ background: "var(--bg-sidebar)", borderBottom: "0.5px solid var(--separator)" }}>
        <SkeletonPulse className="h-5 w-5" style={{ borderRadius: "50%" }} />
        <SkeletonPulse className="h-3 w-16" />
      </div>
      <div className="flex min-h-0 flex-1">
        <div className="flex w-[260px] shrink-0 flex-col gap-3 p-4" style={{ borderRight: "0.5px solid var(--separator)" }}>
          <SkeletonPulse className="h-9 w-full" />
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="flex items-center gap-3" style={{ animationDelay: `${i * 80}ms` }}>
              <SkeletonPulse className="h-9 w-9 shrink-0" style={{ borderRadius: "50%" }} />
              <div className="flex flex-1 flex-col gap-1.5">
                <SkeletonPulse className="h-3 w-24" />
                <SkeletonPulse className="h-2.5 w-36" />
              </div>
            </div>
          ))}
        </div>
        <div className="flex flex-1 flex-col items-center justify-center gap-3">
          <div style={{ animation: "pulse-subtle 2s ease-in-out infinite" }}><ClawIcon size={48} /></div>
          <SkeletonPulse className="h-3 w-20" />
        </div>
      </div>
    </div>
  );
}

export function AppLayout() {
  const mode = useGatewayStore((s) => s.mode);
  const error = useGatewayStore((s) => s.error);
  const connected = useGatewayStore((s) => s.connected);

  const activeAgentId = useAgentStore((s) => s.activeAgentId);
  const agents = useAgentStore((s) => s.agents);
  const detailOpen = useAgentStore((s) => s.detailOpen);
  const toggleDetail = useAgentStore((s) => s.toggleDetail);
  const closeDetail = useAgentStore((s) => s.closeDetail);
  const sidebarCollapsed = useAgentStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useAgentStore((s) => s.toggleSidebar);

  const activeAgent = useMemo(
    () => agents.find((a) => a.id === activeAgentId) ?? agents[0],
    [agents, activeAgentId],
  );

  const [showOnboarding, setShowOnboarding] = useState(false);
  const [onboardingChecked, setOnboardingChecked] = useState(false);
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    if (!isTauri) return;
    let cancelled = false;
    let unlistenFn: (() => void) | undefined;
    (async () => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      if (!cancelled) setIsMaximized(await win.isMaximized());
      unlistenFn = await win.onResized(async () => {
        if (!cancelled) setIsMaximized(await win.isMaximized());
      });
    })();
    return () => {
      cancelled = true;
      unlistenFn?.();
    };
  }, []);

  useEffect(() => {
    if (mode === "connecting" || (!connected && mode !== "embedded")) return;
    let cancelled = false;
    (async () => {
      try {
        const [cfg, models] = await Promise.all([
          api.getConfig("onboarding") as Promise<{ value?: { completed?: boolean }; completed?: boolean } | null>,
          api.listModels(),
        ]);
        if (cancelled) return;
        const val = cfg?.value ?? cfg;
        if (val && typeof val === "object" && (val as Record<string, unknown>).completed) {
          setShowOnboarding(false);
          setOnboardingChecked(true);
          return;
        }
        setShowOnboarding(models.length === 0);
        setOnboardingChecked(true);
      } catch {
        if (!cancelled) { setShowOnboarding(false); setOnboardingChecked(true); }
      }
    })();
    return () => { cancelled = true; };
  }, [mode, connected]);

  const handleOnboardingComplete = useCallback(async () => {
    try { await api.setConfig("onboarding", { completed: true }); } catch { /* best-effort */ }
    setShowOnboarding(false);
  }, []);

  let content: React.ReactNode;

  if (mode === "connecting" || !activeAgent || !onboardingChecked) {
    content = <Loading error={error} />;
  } else if (showOnboarding) {
    content = (
      <>
        <TitleBar />
        <Suspense fallback={<div className="flex-1" style={{ background: "var(--bg-primary)" }} />}>
          <OnboardingWizard onComplete={handleOnboardingComplete} />
        </Suspense>
      </>
    );
  } else {
    content = (
      <>
        <TitleBar />
        <UpdateBanner />
        <div className="flex min-h-0 flex-1">
          <AgentList collapsed={sidebarCollapsed} onToggleCollapse={toggleSidebar} />
          <main className="relative flex min-w-0 flex-1 flex-col">
            <MessageStream onToggleDetail={toggleDetail} detailOpen={detailOpen} />
            {!connected && mode !== "browser" && (
              <div
                className="absolute inset-x-0 top-0 z-20 flex items-center justify-center py-1.5"
                style={{
                  background: "rgba(var(--bg-primary-rgb, 0, 0, 0), 0.85)",
                  backdropFilter: "blur(8px)",
                  animation: "fade-in var(--duration-slow)",
                }}
              >
                <span className="text-[12px]" style={{ color: "var(--fill-tertiary)" }}>
                  连接已断开，正在重连...
                </span>
              </div>
            )}
          </main>
          <Suspense fallback={null}>
            <AgentDetail
              open={detailOpen}
              onClose={closeDetail}
              agentName={activeAgent.name}
              agentInitial={activeAgent.initial}
              agentColor={activeAgent.color}
            />
          </Suspense>
        </div>
      </>
    );
  }

  return (
    <div className={`app-shell flex h-full flex-col${isMaximized ? " maximized" : ""}`}>
      {content}
    </div>
  );
}
