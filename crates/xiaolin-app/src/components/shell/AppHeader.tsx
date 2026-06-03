import { useState, useEffect, useCallback, type CSSProperties, type ReactNode, type MouseEvent as RME } from "react";
import {
  PanelLeft,
  PanelBottom,
  ChevronLeft,
  ChevronRight,
  ChevronDown,
  Sun,
  Moon,
  Diamond,
  Square,
  Minus,
  Maximize2,
  X,
} from "lucide-react";
import { useUIStore } from "../../lib/stores";
import { useThemeStore } from "../../lib/theme";
import { useWorkspaceTabs } from "./workspace-tabs";

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

const iconBtnBase: CSSProperties = {
  width: 28,
  height: 28,
  borderRadius: 6,
  border: "none",
  background: "transparent",
  color: "var(--fill-quaternary)",
  cursor: "pointer",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  transition: "background 0.12s, color 0.12s",
};

const ICON_SIZE = 15;

function IconButton({
  children,
  title,
  onClick,
  style,
  active,
}: {
  children: ReactNode;
  title?: string;
  onClick?: ((e: RME<HTMLButtonElement>) => void) | (() => void);
  style?: CSSProperties;
  active?: boolean;
}) {
  return (
    <button
      type="button"
      style={{
        ...iconBtnBase,
        ...(active ? { background: "var(--bg-hover)", color: "var(--fill-secondary)" } : {}),
        ...style,
      }}
      title={title}
      onClick={onClick}
      onMouseEnter={(e) => { e.currentTarget.style.background = "var(--bg-hover)"; }}
      onMouseLeave={(e) => {
        e.currentTarget.style.background = active ? "var(--bg-hover)" : "transparent";
      }}
    >
      {children}
    </button>
  );
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
      if (!cancelled) setIsMaximized(await win.isMaximized());
      unlistenFn = await win.onResized(async () => {
        if (!cancelled) setIsMaximized(await win.isMaximized());
      });
    })();
    return () => { cancelled = true; unlistenFn?.(); };
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

  const wc: CSSProperties = {
    ...iconBtnBase,
    width: 36,
    borderRadius: 0,
  };

  return (
    <div style={{ display: "flex", alignItems: "stretch", height: "100%", marginLeft: 4 }}>
      <div style={{ width: 1, alignSelf: "center", height: 14, background: "var(--separator)" }} />
      <button type="button" style={wc} onClick={minimize} title="最小化"
        onMouseEnter={(e) => { e.currentTarget.style.background = "var(--bg-hover)"; }}
        onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}>
        <Minus size={14} strokeWidth={1.2} />
      </button>
      <button type="button" style={wc} onClick={toggleMaximize}
        title={isMaximized ? "还原" : "最大化"}
        onMouseEnter={(e) => { e.currentTarget.style.background = "var(--bg-hover)"; }}
        onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}>
        {isMaximized ? <Maximize2 size={14} strokeWidth={1.2} /> : <Square size={14} strokeWidth={1.2} />}
      </button>
      <button type="button" style={{ ...wc, borderRadius: "0 0 0 0" }} onClick={close} title="关闭"
        onMouseEnter={(e) => { e.currentTarget.style.background = "#E81123"; e.currentTarget.style.color = "#fff"; }}
        onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; e.currentTarget.style.color = "var(--fill-quaternary)"; }}>
        <X size={14} strokeWidth={1.2} />
      </button>
    </div>
  );
}

export function AppHeader() {
  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const resolved = useThemeStore((s) => s.resolved);
  const setMode = useThemeStore((s) => s.setMode);

  const togglePanel = useWorkspaceTabs((s) => s.togglePanel);
  const panelOpen = useWorkspaceTabs((s) => s.panelOpen);

  const handleThemeToggle = useCallback(() => {
    setMode(resolved === "light" ? "dark" : "light");
  }, [resolved, setMode]);

  const comingSoon = useCallback((e: RME<HTMLButtonElement>) => {
    const el = e.currentTarget;
    el.style.background = "var(--tint)";
    el.style.color = "#fff";
    setTimeout(() => { el.style.background = "transparent"; el.style.color = "var(--fill-quaternary)"; }, 180);
  }, []);

  return (
    <header
      className="app-header"
      style={{
        height: "var(--header-h)",
        minHeight: "var(--header-h)",
        display: "flex",
        alignItems: "center",
        flexShrink: 0,
        background: "var(--bg-shell)",
        padding: "0 12px",
        position: "relative",
        zIndex: 10,
      } as CSSProperties}
    >
      {/* Left: nav tools */}
      <div style={{ display: "flex", alignItems: "center", gap: 2 }}>
        <IconButton title="切换侧边栏" onClick={toggleSidebar} active={!sidebarCollapsed}>
          <PanelLeft size={16} strokeWidth={1.7} />
        </IconButton>
        <IconButton title="切换面板" onClick={togglePanel} active={panelOpen}>
          <PanelBottom size={16} strokeWidth={1.7} />
        </IconButton>
        <IconButton title="后退" onClick={comingSoon}>
          <ChevronLeft size={16} strokeWidth={1.7} />
        </IconButton>
        <IconButton title="前进" onClick={comingSoon}>
          <ChevronRight size={16} strokeWidth={1.7} />
        </IconButton>
      </div>

      {/* Center: title — drag region */}
      <div
        data-tauri-drag-region=""
        onMouseDown={onDragMouseDown}
        onDoubleClick={onDragDoubleClick}
        style={{
          flex: 1,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          gap: 6,
          minWidth: 0,
          pointerEvents: "auto",
        }}
      >
        <span
          style={{
            fontSize: 13,
            fontWeight: 600,
            color: "var(--fill-primary)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            letterSpacing: "-0.01em",
          }}
        >
          New chat
        </span>
        <span
          style={{
            fontSize: 12,
            color: "var(--fill-quaternary)",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}
        >
          XiaoLin
        </span>
        <span
          style={{
            color: "var(--fill-quaternary)",
            cursor: "pointer",
            fontSize: 18,
            letterSpacing: 1,
            padding: "0 4px",
            lineHeight: 1,
            pointerEvents: "auto",
          }}
        >
          ···
        </span>
      </div>

      {/* Right: actions + window controls */}
      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
        <IconButton title={resolved === "light" ? "深色模式" : "浅色模式"} onClick={handleThemeToggle}>
          {resolved === "light" ? <Sun size={ICON_SIZE} strokeWidth={1.7} /> : <Moon size={ICON_SIZE} strokeWidth={1.7} />}
        </IconButton>
        <IconButton title="选项" onClick={comingSoon}>
          <ChevronDown size={ICON_SIZE} strokeWidth={1.7} />
        </IconButton>
        <div style={{ fontSize: 11, fontFamily: "var(--font-mono)", display: "flex", gap: 4, padding: "0 2px" }}>
          <span style={{ color: "var(--green-text)" }}>+0</span>
          <span style={{ color: "var(--red-text)" }}>-0</span>
        </div>
        <button
          type="button"
          disabled
          style={{
            display: "flex",
            alignItems: "center",
            gap: 5,
            padding: "4px 10px",
            borderRadius: 8,
            fontSize: 12,
            fontWeight: 500,
            border: "1.5px solid var(--green-text)",
            color: "var(--green-text)",
            background: "transparent",
            cursor: "not-allowed",
            opacity: 0.6,
            transition: "background 0.12s",
          }}
          title="Git 集成即将推出"
        >
          <Diamond size={12} strokeWidth={2} />
          Commit
          <ChevronDown size={10} strokeWidth={2} />
        </button>
        <div style={{ display: "flex", gap: 1 }}>
          <IconButton title="单栏" onClick={() => { if (panelOpen) togglePanel(); }} active={!panelOpen}>
            <svg viewBox="0 0 24 24" width={14} height={14} fill="none" stroke="currentColor" strokeWidth={1.7}>
              <rect x="3" y="3" width="18" height="18" rx="3" />
            </svg>
          </IconButton>
          <IconButton title="分栏" onClick={() => { if (!panelOpen) togglePanel(); }} active={panelOpen}>
            <svg viewBox="0 0 24 24" width={14} height={14} fill="none" stroke="currentColor" strokeWidth={1.7}>
              <rect x="3" y="3" width="18" height="18" rx="3" />
              <line x1="12" y1="3" x2="12" y2="21" />
            </svg>
          </IconButton>
        </div>
        <WindowControls />
      </div>
    </header>
  );
}
