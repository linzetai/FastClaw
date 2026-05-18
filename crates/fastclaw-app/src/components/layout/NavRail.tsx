import { lazy, Suspense, useState } from "react";
import { MessageSquare, Users, Layout, ListTodo, FolderOpen, Link2, HelpCircle, Settings } from "lucide-react";
import { useAgentStore } from "../../lib/agent-store";
import { ClawIcon } from "./ClawIcon";
import type { NavItem } from "../../lib/stores/ui-store";

const SettingsPanel = lazy(() =>
  import("../settings/SettingsPanel").then((m) => ({ default: m.SettingsPanel })),
);

interface NavEntry {
  id: NavItem;
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  label: string;
}

const TOP_ITEMS: NavEntry[] = [
  { id: "chat", icon: MessageSquare, label: "对话" },
  { id: "experts", icon: Users, label: "专家" },
  { id: "workspace", icon: Layout, label: "工作室" },
  { id: "tasks", icon: ListTodo, label: "任务" },
  { id: "files", icon: FolderOpen, label: "文件" },
  { id: "connections", icon: Link2, label: "连接" },
];

export function NavRail() {
  const activeNav = useAgentStore((s) => s.activeNav);
  const setActiveNav = useAgentStore((s) => s.setActiveNav);
  const [settingsOpen, setSettingsOpen] = useState(false);

  return (
    <>
      {settingsOpen && (
        <Suspense fallback={null}>
          <SettingsPanel open={settingsOpen} onClose={() => setSettingsOpen(false)} />
        </Suspense>
      )}
      <nav
        className="flex shrink-0 flex-col items-center justify-between py-3"
        style={{
          width: "var(--nav-rail-w)",
          background: "var(--bg-secondary)",
          borderRight: "0.5px solid var(--separator)",
        }}
      >
        <div className="flex flex-col items-center gap-0.5">
          <div
            className="mb-3 flex h-9 w-9 items-center justify-center transition-all duration-300"
            style={{ color: "var(--fill-primary)", filter: "drop-shadow(0 0 0px transparent)" }}
            onMouseEnter={(e) => { e.currentTarget.style.filter = `drop-shadow(0 0 8px var(--tint))`; e.currentTarget.style.opacity = "1"; }}
            onMouseLeave={(e) => { e.currentTarget.style.filter = "drop-shadow(0 0 0px transparent)"; e.currentTarget.style.opacity = "0.85"; }}
          >
            <ClawIcon size={28} />
          </div>

          {TOP_ITEMS.map((item) => {
            const active = activeNav === item.id;
            const Icon = item.icon;
            return (
              <button
                key={item.id}
                onClick={() => setActiveNav(item.id)}
                className="group relative flex h-11 w-11 flex-col items-center justify-center rounded-xl transition-all duration-150"
                style={{
                  background: active ? "var(--tint-bg)" : "transparent",
                  color: active ? "var(--tint)" : "var(--fill-tertiary)",
                }}
                title={item.label}
              >
                {active && (
                  <span
                    className="absolute left-0 top-1/2 w-[3px] h-5 rounded-full -translate-y-1/2"
                    style={{
                      background: "var(--tint)",
                      animation: "scale-spring var(--duration-normal) var(--ease-spring)",
                    }}
                  />
                )}
                <span className="transition-transform duration-150 group-hover:scale-110">
                  <Icon size={18} strokeWidth={active ? 2 : 1.5} />
                </span>
                <span
                  className="mt-[2px] text-[9px] font-medium leading-none"
                  style={{ color: active ? "var(--tint)" : "var(--fill-quaternary)" }}
                >
                  {item.label}
                </span>
              </button>
            );
          })}
        </div>

        <div className="flex flex-col items-center gap-1">
          <div className="mb-1 w-6 h-px" style={{ background: "var(--separator)" }} />
          <button
            className="flex h-9 w-9 items-center justify-center rounded-lg transition-all duration-150 hover:bg-[var(--bg-hover)] hover:scale-105 active:scale-95"
            style={{ color: "var(--fill-tertiary)" }}
            title="帮助"
          >
            <HelpCircle size={17} strokeWidth={1.5} />
          </button>
          <button
            onClick={() => setSettingsOpen(true)}
            className="flex h-9 w-9 items-center justify-center rounded-lg transition-all duration-150 hover:bg-[var(--bg-hover)] hover:scale-105 active:scale-95"
            style={{ color: "var(--fill-tertiary)" }}
            title="设置"
          >
            <Settings size={17} strokeWidth={1.5} />
          </button>
        </div>
      </nav>
    </>
  );
}
