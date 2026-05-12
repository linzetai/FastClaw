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

  const btn = "flex items-center justify-center transition-colors duration-100";

  return (
    <div className="ml-1 flex h-full items-stretch">
      <div className="my-auto h-4 w-px" style={{ background: "var(--separator)" }} />
      <button
        onClick={minimize}
        className={`${btn} w-[46px] hover:bg-[var(--bg-hover)]`}
        style={{ color: "var(--fill-secondary)" }}
        title="最小化"
      >
        <Minus size={12} strokeWidth={1.5} />
      </button>
      <button
        onClick={toggleMaximize}
        className={`${btn} w-[46px] hover:bg-[var(--bg-hover)]`}
        style={{ color: "var(--fill-secondary)" }}
        title={isMaximized ? "还原" : "最大化"}
      >
        {isMaximized ? <Maximize2 size={10} strokeWidth={1.5} /> : <Square size={10} strokeWidth={1.5} />}
      </button>
      <button
        onClick={close}
        className={`${btn} w-[46px] hover:bg-[#E81123] hover:text-white`}
        style={{ color: "var(--fill-secondary)", transition: "background 0.15s, color 0.15s" }}
        title="关闭"
      >
        <X size={12} strokeWidth={1.5} />
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
      <span
        className="inline-block h-[7px] w-[7px] rounded-full transition-colors duration-300"
        style={{ background: connected ? "var(--green)" : "var(--red)" }}
      />
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
          borderBottom: `0.5px solid var(--separator)`,
        }}
      >
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
