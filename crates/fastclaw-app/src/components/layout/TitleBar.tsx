import { useState, useEffect, useCallback, type MouseEvent as RME } from "react";
import { useGatewayStore } from "../../lib/store";
import { NotificationCenter } from "../notification/NotificationCenter";
import { NotificationDetailPanel } from "../notification/NotificationDetailPanel";
import { Minus, Square, Maximize2, X } from "lucide-react";
import type { AppNotification } from "../../lib/transport";

const isTauri =
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

async function onDragMouseDown(e: RME) {
  if (!isTauri || e.button !== 0) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  getCurrentWindow().startDragging();
}

async function onDragDoubleClick() {
  if (!isTauri) return;
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  getCurrentWindow().toggleMaximize();
}

function WindowControls() {
  const [isMaximized, setIsMaximized] = useState(false);

  useEffect(() => {
    if (!isTauri) return;
    let cancelled = false;
    let unlistenFn: (() => void) | undefined;
    (async () => {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      const win = getCurrentWindow();
      setIsMaximized(await win.isMaximized());
      unlistenFn = await win.onResized(async () => {
        if (!cancelled) setIsMaximized(await win.isMaximized());
      });
    })();
    return () => {
      cancelled = true;
      unlistenFn?.();
    };
  }, []);

  const minimize = useCallback(async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    getCurrentWindow().minimize();
  }, []);

  const toggleMaximize = useCallback(async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    getCurrentWindow().toggleMaximize();
  }, []);

  const close = useCallback(async () => {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    getCurrentWindow().close();
  }, []);

  if (!isTauri) return null;

  const btn = "flex items-center justify-center rounded-[4px] transition-all duration-150";

  return (
    <div className="ml-1 flex h-full items-stretch gap-px">
      <div className="my-auto h-4 w-px" style={{ background: "var(--separator)" }} />
      <button
        onClick={minimize}
        className={`${btn} w-[42px] hover:bg-[var(--bg-hover)] hover:scale-[1.02] active:scale-95`}
        style={{ color: "var(--fill-primary)" }}
        title="最小化"
      >
        <Minus size={16} strokeWidth={1.5} />
      </button>
      <button
        onClick={toggleMaximize}
        className={`${btn} w-[42px] hover:bg-[var(--bg-hover)] hover:scale-[1.02] active:scale-95`}
        style={{ color: "var(--fill-primary)" }}
        title={isMaximized ? "还原" : "最大化"}
      >
        {isMaximized ? <Maximize2 size={16} strokeWidth={1.5} /> : <Square size={16} strokeWidth={1.5} />}
      </button>
      <button
        onClick={close}
        className={`${btn} w-[42px] hover:bg-[#E81123] hover:text-white active:scale-95`}
        style={{ color: "var(--fill-primary)", transition: "background 0.15s, color 0.15s, transform 0.1s, box-shadow 0.15s" }}
        onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.boxShadow = "0 0 8px rgba(232,17,35,0.3)"; }}
        onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.boxShadow = "none"; }}
      >
        <X size={16} strokeWidth={1.5} />
      </button>
    </div>
  );
}

function ConnectionDot() {
  const connected = useGatewayStore((s) => s.connected);
  return (
    <div
      className="flex h-7 w-7 items-center justify-center"
      title={connected ? "已连接" : "未连接"}
    >
      <span className="relative inline-flex items-center justify-center">
        <span
          className="inline-block h-[7px] w-[7px] rounded-full transition-colors duration-300"
          style={{ background: connected ? "var(--green)" : "var(--red)" }}
        />
        {connected && (
          <span
            className="absolute inline-block h-[7px] w-[7px] rounded-full"
            style={{
              background: "var(--green)",
              animation: "pulse-ring 2s ease-out infinite",
            }}
          />
        )}
        {!connected && (
          <span
            className="absolute inline-block h-[7px] w-[7px] rounded-full"
            style={{
              background: "var(--red)",
              animation: "shake 0.5s ease-in-out",
            }}
          />
        )}
      </span>
    </div>
  );
}

export function TitleBar() {
  const [detailNotification, setDetailNotification] = useState<AppNotification | null>(null);

  return (
    <>
      {detailNotification && (
        <NotificationDetailPanel
          notification={detailNotification}
          onClose={() => setDetailNotification(null)}
        />
      )}
      <header
        className="relative z-30 flex shrink-0 select-none items-stretch"
        style={{
          height: "var(--titlebar-h)",
          background: "var(--bg-sidebar)",
        }}
      >
        <div
          className="absolute inset-x-0 bottom-0 h-px pointer-events-none"
          style={{ background: "linear-gradient(90deg, transparent 5%, var(--separator) 50%, transparent 95%)" }}
        />
        <div
          className="h-full flex-1"
          data-tauri-drag-region=""
          onMouseDown={onDragMouseDown}
          onDoubleClick={onDragDoubleClick}
          style={{ WebkitAppRegion: "drag" } as React.CSSProperties}
        />

        <div className="flex h-full items-center gap-0.5">
          <ConnectionDot />
          <NotificationCenter onDetailOpen={setDetailNotification} />
          <WindowControls />
        </div>
      </header>
    </>
  );
}
