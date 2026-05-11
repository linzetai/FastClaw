import { useState, useCallback, lazy, Suspense } from "react";
import { useAgentStore } from "../../lib/agent-store";
import { X, Camera } from "lucide-react";
import * as api from "../../lib/api";
import * as transport from "../../lib/transport";
import { useAvatarUrl, loadAvatarBlobUrl } from "../../lib/use-avatar-url";
import type { ConfigSection } from "./AgentConfigForm";

const ChatsTab = lazy(() => import("./AgentChatsTab").then((m) => ({ default: m.ChatsTab })));
const CronTab = lazy(() => import("./AgentCronTab").then((m) => ({ default: m.CronTab })));
const AgentConfigForm = lazy(() => import("./AgentConfigForm").then((m) => ({ default: m.AgentConfigForm })));

export interface AgentDetailProps {
  open: boolean;
  onClose: () => void;
  agentName: string;
  agentInitial: string;
  agentColor: string;
}

type MainTab = ConfigSection | "chats" | "cron";

const CONFIG_TABS: MainTab[] = ["basic", "tools", "skills", "identity", "chats", "cron"];

function tabLabel(t: MainTab) {
  switch (t) {
    case "basic": return "基础";
    case "tools": return "工具";
    case "skills": return "技能";
    case "identity": return "身份";
    case "chats": return "会话";
    case "cron": return "定时";
  }
}

export function AgentDetail({ open, onClose, agentName, agentInitial, agentColor: _agentColor }: AgentDetailProps) {
  const [tab, setTab] = useState<MainTab>("basic");
  const activeAgentId = useAgentStore((s) => s.activeAgentId);
  const agents = useAgentStore((s) => s.agents);
  const updateAgentProps = useAgentStore((s) => s.updateAgentProps);
  const agent = agents.find((a) => a.id === activeAgentId);
  const [uploadPreview, setUploadPreview] = useState<string | null>(null);

  const storedAvatarUrl = useAvatarUrl(agent?.avatar);
  const avatarSrc = uploadPreview ?? storedAvatarUrl;

  const handleAvatarClick = useCallback(async () => {
    if (!transport.isTauri) return;
    try {
      const { open: openDialog } = await import("@tauri-apps/plugin-dialog");
      const selected = await openDialog({
        title: "选择头像图片",
        filters: [{ name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] }],
        multiple: false,
      });
      if (!selected) return;
      const path = typeof selected === "string" ? selected : (selected as { path?: string }).path;
      if (!path) return;
      const resp = await api.uploadAgentAvatar(activeAgentId, path);
      if (resp) {
        updateAgentProps(activeAgentId, { avatar: resp });
        const url = await loadAvatarBlobUrl(resp);
        if (url) setUploadPreview(url);
      }
    } catch (e) {
      console.warn("[AgentDetail] avatar upload failed:", e);
    }
  }, [activeAgentId, updateAgentProps]);

  return (
    <aside
      className="flex shrink-0 flex-col overflow-hidden"
      data-testid="agent-detail-panel"
      style={{
        width: open ? 320 : 0,
        opacity: open ? 1 : 0,
        borderLeft: open ? "0.5px solid var(--separator)" : "none",
        background: "var(--bg-secondary)",
        transition: `width var(--duration-slow) var(--ease-out), opacity var(--duration-slow) var(--ease-out)`,
      }}
    >
      <div className="flex shrink-0 items-center justify-between gap-2 px-4 py-3.5" style={{ borderBottom: "0.5px solid var(--separator)" }}>
        <div className="flex min-w-0 flex-1 items-center gap-2.5">
          <button
            className="group relative flex h-8 w-8 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-full text-[12px] font-semibold"
            style={{ background: "var(--bg-tertiary)", color: "var(--fill-secondary)" }}
            onClick={handleAvatarClick}
            title="修改头像"
          >
            {avatarSrc ? (
              <img src={avatarSrc} alt="" className="h-full w-full object-cover" />
            ) : (
              agentInitial
            )}
            <div className="absolute inset-0 flex items-center justify-center rounded-full opacity-0 group-hover:opacity-100" style={{ background: "rgba(0,0,0,0.3)", transition: `opacity var(--duration-instant) var(--ease-in-out)` }}>
              <Camera size={12} strokeWidth={1.5} color="white" />
            </div>
          </button>
          <span className="min-w-0 truncate text-[14px] font-semibold" style={{ color: "var(--fill-primary)" }} title={agentName}>{agentName}</span>
        </div>
        <button onClick={onClose} className="flex h-7 w-7 shrink-0 cursor-pointer items-center justify-center rounded-full hover:bg-[var(--bg-hover)]" style={{ color: "var(--fill-tertiary)", transition: `background var(--duration-instant) var(--ease-in-out)` }} title="关闭面板">
          <X size={14} strokeWidth={1.5} />
        </button>
      </div>

      <div className="flex shrink-0 gap-0 px-3" style={{ borderBottom: "0.5px solid var(--separator)" }}>
        {CONFIG_TABS.map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className="relative cursor-pointer px-2.5 py-2 text-[12px] font-medium transition-colors duration-150"
            style={{
              color: tab === t ? "var(--tint)" : "var(--fill-tertiary)",
            }}
          >
            {tabLabel(t)}
            {tab === t && (
              <span
                className="absolute bottom-0 left-1/2 h-[2px] w-4/5 -translate-x-1/2 rounded-full"
                style={{ background: "var(--tint)", animation: "scale-in var(--duration-fast) var(--ease-out)" }}
              />
            )}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-y-auto">
        <Suspense fallback={<div className="h-full" style={{ background: "var(--bg-secondary)" }} />}>
          <div key={tab} style={{ animation: "tab-crossfade var(--duration-normal) var(--ease-out)" }}>
            {tab === "chats" ? <ChatsTab /> : tab === "cron" ? <CronTab key={`cron-${activeAgentId}`} /> : (
              <AgentConfigForm key={activeAgentId} section={tab} />
            )}
          </div>
        </Suspense>
      </div>
    </aside>
  );
}
