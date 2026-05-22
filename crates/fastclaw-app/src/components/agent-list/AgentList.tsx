import { useState, useCallback, useRef, useEffect, useMemo, memo } from "react";
import { createPortal } from "react-dom";
import { useAgentStore } from "../../lib/agent-store";
import { useGatewayStore } from "../../lib/store";
import {
  Search, Plus, ChevronDown, ChevronRight, Check, Camera, X,
  PanelLeftClose, PanelLeftOpen, MoreHorizontal, Pin, User, Trash2, MessageCircle,
} from "lucide-react";
import { ICON, BTN_ICON } from "../../lib/ui-tokens";
import * as api from "../../lib/api";
import * as transport from "../../lib/transport";
import { useAvatarUrl, loadAvatarBlobUrl } from "../../lib/use-avatar-url";
import { fuzzyMatch } from "../../lib/fuzzy";
import type { Agent, Chat } from "../../lib/stores/types";

const AgentAvatar = memo(function AgentAvatar({ agent, size = 36 }: { agent: Agent; size?: number }) {
  const avatarUrl = useAvatarUrl(agent.avatar);
  return (
    <div
      className="flex shrink-0 items-center justify-center overflow-hidden rounded-full font-semibold"
      style={{
        width: size,
        height: size,
        fontSize: Math.round(size * 0.38),
        background: agent.color || "var(--bg-tertiary)",
        color: agent.color ? "#fff" : "var(--fill-secondary)",
      }}
    >
      {avatarUrl ? (
        <img src={avatarUrl} alt="" className="h-full w-full object-cover" />
      ) : (
        agent.initial
      )}
    </div>
  );
});

function AgentContextMenu({
  x, y, onClose, onPin, onDetail, onDelete,
}: {
  x: number; y: number;
  onClose: () => void;
  onPin: () => void;
  onDetail: () => void;
  onDelete: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) onClose();
    };
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("mousedown", handleClick);
    document.addEventListener("keydown", handleKey);
    return () => {
      document.removeEventListener("mousedown", handleClick);
      document.removeEventListener("keydown", handleKey);
    };
  }, [onClose]);

  const items = [
    { icon: Pin, label: "置顶", action: onPin },
    { icon: User, label: "Agent 详情", action: onDetail },
    { icon: Trash2, label: "删除", action: onDelete, danger: true },
  ];

  return createPortal(
    <div
      ref={ref}
      className="fixed z-[60] min-w-[140px] overflow-hidden rounded-lg py-1"
      style={{
        left: x,
        top: y,
        background: "var(--bg-elevated)",
        border: "0.5px solid var(--separator)",
        boxShadow: "var(--shadow-lg)",
        animation: "scale-in var(--duration-fast) var(--ease-out)",
        transformOrigin: "top left",
      }}
    >
      {items.map((item) => {
        const Icon = item.icon;
        return (
          <button
            key={item.label}
            onClick={() => { item.action(); onClose(); }}
            className="flex w-full items-center gap-2.5 px-3 py-2 text-left text-[12px] font-medium transition-colors duration-100 hover:bg-[var(--bg-hover)]"
            style={{ color: item.danger ? "var(--red)" : "var(--fill-secondary)" }}
          >
            <Icon {...ICON.md} />
            {item.label}
          </button>
        );
      })}
    </div>,
    document.body,
  );
}

function ChatItem({
  chat, active, onClick,
}: {
  chat: Chat; active: boolean; onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`group/chat flex w-full items-center gap-2 rounded-md px-3 py-1.5 text-left transition-colors duration-100 ${
        active ? "" : "hover:bg-[var(--bg-hover)]"
      }`}
      style={active ? { background: "var(--tint-bg)" } : undefined}
    >
      <MessageCircle {...ICON.sm} style={{ color: active ? "var(--tint)" : "var(--fill-quaternary)", flexShrink: 0 }} />
      <span
        className="min-w-0 truncate text-[12px]"
        style={{ color: active ? "var(--tint)" : "var(--fill-tertiary)" }}
      >
        {chat.title || "新会话"}
      </span>
    </button>
  );
}

interface AgentListProps {
  collapsed?: boolean;
  onToggleCollapse?: () => void;
}

export function AgentList({ collapsed = false, onToggleCollapse }: AgentListProps) {
  const agents = useAgentStore((s) => s.agents);
  const activeAgentId = useAgentStore((s) => s.activeAgentId);
  const agentChats = useAgentStore((s) => s.agentChats);
  const setActiveAgent = useAgentStore((s) => s.setActiveAgent);
  const setActiveChat = useAgentStore((s) => s.setActiveChat);
  const syncAgents = useAgentStore((s) => s.syncAgentsFromBackend);
  const newChat = useAgentStore((s) => s.newChat);
  const toggleDetail = useAgentStore((s) => s.toggleDetail);
  const removeAgent = useAgentStore((s) => s.removeAgent);
  const gatewayReady = useGatewayStore((s) => s.connected);

  const [query, setQuery] = useState("");
  const [creating, setCreating] = useState(false);
  const [showNewForm, setShowNewForm] = useState(false);
  const [newName, setNewName] = useState("");
  const [newAgentId, setNewAgentId] = useState("");
  const [agentIdTouched, setAgentIdTouched] = useState(false);
  const [newModel, setNewModel] = useState("");
  const [newAvatarPath, setNewAvatarPath] = useState<string | null>(null);
  const [newAvatarPreview, setNewAvatarPreview] = useState<string | null>(null);
  const [models, setModels] = useState<api.ModelInfo[]>([]);
  const [modelsLoading, setModelsLoading] = useState(false);
  const [onboardingError, setOnboardingError] = useState<string | null>(null);
  const newInputRef = useRef<HTMLInputElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);

  const [expandedAgents, setExpandedAgents] = useState<Set<string>>(() => new Set([activeAgentId]));
  const [contextMenu, setContextMenu] = useState<{ agentId: string; x: number; y: number } | null>(null);
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);

  useEffect(() => {
    if (activeAgentId) {
      setExpandedAgents((prev) => {
        if (prev.has(activeAgentId)) return prev;
        const next = new Set(prev);
        next.add(activeAgentId);
        return next;
      });
    }
  }, [activeAgentId]);

  const toggleExpand = useCallback((agentId: string) => {
    setExpandedAgents((prev) => {
      const next = new Set(prev);
      if (next.has(agentId)) next.delete(agentId);
      else next.add(agentId);
      return next;
    });
  }, []);

  const refreshModels = useCallback(async () => {
    if (!gatewayReady) return;
    try {
      const m = await api.listModels();
      setModels(m);
      setNewModel((prev) => {
        if (m.length === 0) return "";
        if (!prev || !m.some((item) => item.model === prev)) return m[0].model;
        return prev;
      });
    } catch {
      setModels([]);
      setNewModel("");
    }
  }, [gatewayReady]);

  useEffect(() => {
    if (showNewForm) {
      newInputRef.current?.focus();
      if (gatewayReady) {
        setModelsLoading(true);
        refreshModels().finally(() => setModelsLoading(false));
      }
    }
  }, [showNewForm, gatewayReady, refreshModels]);

  useEffect(() => {
    const onModelsUpdated = () => void refreshModels();
    window.addEventListener("fastclaw:models-updated", onModelsUpdated);
    return () => window.removeEventListener("fastclaw:models-updated", onModelsUpdated);
  }, [refreshModels]);

  const pickAvatar = useCallback(async () => {
    if (!transport.isTauri) return;
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const result = await open({
        title: "选择头像",
        filters: [{ name: "图片", extensions: ["png", "jpg", "jpeg", "webp", "gif"] }],
        multiple: false,
      });
      if (result) {
        const path = typeof result === "string" ? result : result;
        setNewAvatarPath(path as string);
        const url = await loadAvatarBlobUrl(path as string);
        setNewAvatarPreview(url);
      }
    } catch { /* user cancelled */ }
  }, []);

  const handleNewAgent = useCallback(async () => {
    const trimmed = newName.trim();
    if (!trimmed) return;
    setCreating(true);
    const selectedModelMeta = models.find((m) => m.model === newModel);
    const explicitId = newAgentId.trim() || undefined;
    const agent = await api.createAgent({
      name: trimmed,
      agentId: explicitId,
      model: newModel || undefined,
      provider: selectedModelMeta?.provider,
    });
    if (agent) {
      let avatarPath: string | undefined;
      if (newAvatarPath) {
        avatarPath = (await api.uploadAgentAvatar(agent.agentId, newAvatarPath)) ?? undefined;
      }
      const newModelStr = typeof agent.model === "string" ? agent.model : agent.model?.model ?? "";
      syncAgents([
        ...agents.map((a) => ({ agentId: a.id, name: a.name, model: a.model, avatar: a.avatar })),
        { agentId: agent.agentId, name: agent.name ?? "", model: newModelStr, avatar: avatarPath },
      ]);
      setActiveAgent(agent.agentId);
    }
    setCreating(false);
    setShowNewForm(false);
    setNewName("");
    setNewModel("");
    setNewAvatarPath(null);
    setNewAvatarPreview(null);
  }, [agents, syncAgents, newName, newModel, newAvatarPath, setActiveAgent, models, newAgentId]);

  const handleQuickCreateDefault = useCallback(async () => {
    setOnboardingError(null);
    let availableModels = models;
    if (availableModels.length === 0) {
      try {
        availableModels = await api.listModels();
        setModels(availableModels);
      } catch {
        availableModels = [];
      }
    }
    if (availableModels.length === 0) {
      setOnboardingError("未检测到可用模型，请先在设置里录入模型。");
      toggleDetail();
      return;
    }
    const defaultName = "Main Agent";
    const created = await api.createAgent({
      name: defaultName,
      model: availableModels[0].model,
      provider: availableModels[0].provider,
    });
    if (!created) {
      setOnboardingError("创建默认 Agent 失败，请稍后重试。");
      return;
    }
    const newModelStr = typeof created.model === "string" ? created.model : created.model?.model ?? "";
    syncAgents([
      ...agents.map((a) => ({ agentId: a.id, name: a.name, model: a.model, avatar: a.avatar })),
      { agentId: created.agentId, name: created.name ?? defaultName, model: newModelStr },
    ]);
    setActiveAgent(created.agentId);
  }, [agents, models, setActiveAgent, syncAgents, toggleDetail]);

  const cancelNew = useCallback(() => {
    setShowNewForm(false);
    setNewName("");
    setNewAgentId("");
    setAgentIdTouched(false);
    setNewModel("");
    setNewAvatarPath(null);
    setNewAvatarPreview(null);
  }, []);

  const handleDelete = useCallback(async (agentId: string) => {
    try { await api.deleteAgent(agentId); } catch { /* best-effort */ }
    removeAgent(agentId);
    setDeleteConfirm(null);
  }, [removeAgent]);

  const [searchFocused, setSearchFocused] = useState(false);
  const [searchSelectedIdx, setSearchSelectedIdx] = useState(0);
  const searchDropdownRef = useRef<HTMLDivElement>(null);
  const searchContainerRef = useRef<HTMLDivElement>(null);

  const searchResults = useMemo(() => {
    if (!query.trim()) return [];
    return agents
      .map((a) => {
        const nameResult = fuzzyMatch(query, a.name);
        const taglineResult = fuzzyMatch(query, a.tagline || "");
        const best = nameResult && taglineResult
          ? (nameResult.score >= taglineResult.score ? nameResult : taglineResult)
          : nameResult ?? taglineResult;
        return best ? { agent: a, score: best.score } : null;
      })
      .filter((r): r is { agent: Agent; score: number } => r !== null)
      .sort((a, b) => b.score - a.score);
  }, [agents, query]);

  const showSearchDropdown = searchFocused && query.trim().length > 0;

  useEffect(() => {
    setSearchSelectedIdx(0);
  }, [query]);

  useEffect(() => {
    if (!showSearchDropdown) return;
    const handleClick = (e: MouseEvent) => {
      if (searchContainerRef.current?.contains(e.target as Node)) return;
      if (searchDropdownRef.current?.contains(e.target as Node)) return;
      setSearchFocused(false);
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [showSearchDropdown]);

  const handleSearchSelect = useCallback((agent: Agent) => {
    setActiveAgent(agent.id);
    newChat(agent.id);
    setQuery("");
    setSearchFocused(false);
  }, [setActiveAgent, newChat]);

  const handleSearchKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (!showSearchDropdown || searchResults.length === 0) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSearchSelectedIdx((i) => (i + 1) % searchResults.length);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSearchSelectedIdx((i) => (i - 1 + searchResults.length) % searchResults.length);
    } else if (e.key === "Enter") {
      e.preventDefault();
      handleSearchSelect(searchResults[searchSelectedIdx].agent);
    } else if (e.key === "Escape") {
      setSearchFocused(false);
    }
  }, [showSearchDropdown, searchResults, searchSelectedIdx, handleSearchSelect]);


  return (
    <aside
      className="flex shrink-0 flex-col"
      style={{
        width: collapsed ? "52px" : "240px",
        background: "var(--bg-sidebar)",
        borderRight: "0.5px solid var(--separator)",
        transition: "width var(--duration-slow) var(--ease-in-out)",
        overflow: "hidden",
      }}
      tabIndex={0}
    >
      {/* Header: Search + New Agent */}
      <div className={`flex flex-col gap-2 pb-2 pt-2 ${collapsed ? "items-center px-2" : "px-3"}`}>
        {collapsed ? (
          <button
            onClick={onToggleCollapse}
            className={BTN_ICON.lg}
            style={{ color: "var(--fill-tertiary)" }}
            title="展开侧边栏"
          >
            <PanelLeftOpen size={20} strokeWidth={1.2} />
          </button>
        ) : (
          <>
            <div className="relative flex items-center gap-1.5" ref={searchContainerRef}>
              <div
                className="flex h-9 min-w-0 flex-1 items-center gap-2 rounded-lg px-3 transition-all duration-200"
                style={{
                  background: searchFocused ? "var(--bg-elevated)" : "var(--bg-hover)",
                  border: searchFocused ? "0.5px solid var(--tint)" : "0.5px solid transparent",
                  boxShadow: searchFocused ? "var(--glow-tint-sm)" : "none",
                }}
              >
                <Search {...ICON.sm} style={{ color: searchFocused ? "var(--tint)" : "var(--fill-tertiary)", flexShrink: 0, transition: "color 0.15s" }} />
                <input
                  ref={searchInputRef}
                  type="text"
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  onFocus={() => setSearchFocused(true)}
                  onKeyDown={handleSearchKeyDown}
                  placeholder="搜索"
                  className="min-w-0 flex-1 bg-transparent text-[13px] outline-none placeholder:text-[var(--fill-quaternary)]"
                  style={{ color: "var(--fill-primary)" }}
                />
                {query && (
                  <button
                    onClick={() => { setQuery(""); setSearchFocused(false); }}
                    className="flex h-4 w-4 shrink-0 cursor-pointer items-center justify-center rounded-full transition-colors duration-150 hover:bg-[var(--bg-active)]"
                    style={{ color: "var(--fill-tertiary)" }}
                  >
                    <X {...ICON.sm} />
                  </button>
                )}
              </div>
              <button
                onClick={onToggleCollapse}
                className={`${BTN_ICON.lg} shrink-0`}
                style={{ color: "var(--fill-tertiary)" }}
                title="折叠侧边栏"
              >
                <PanelLeftClose size={20} strokeWidth={1.2} />
              </button>

              {showSearchDropdown && (
                <div
                  ref={searchDropdownRef}
                  className="absolute left-0 right-9 top-[calc(100%+4px)] z-50 overflow-hidden rounded-lg py-1"
                  style={{
                    background: "var(--bg-elevated)",
                    border: "0.5px solid var(--separator)",
                    boxShadow: "var(--shadow-lg)",
                    animation: "scale-in var(--duration-fast) var(--ease-out)",
                    transformOrigin: "top left",
                    maxHeight: 280,
                    overflowY: "auto",
                  }}
                >
                  {searchResults.length === 0 ? (
                    <div className="flex items-center gap-2 px-3 py-3">
                      <Search {...ICON.sm} style={{ color: "var(--fill-quaternary)" }} />
                      <span className="text-[12px]" style={{ color: "var(--fill-tertiary)" }}>
                        未找到「{query}」相关 Agent
                      </span>
                    </div>
                  ) : (
                    searchResults.map((r, i) => (
                      <button
                        key={r.agent.id}
                        onClick={() => handleSearchSelect(r.agent)}
                        className="flex w-full items-center gap-2.5 px-3 py-2 text-left transition-colors duration-75 hover:bg-[var(--bg-hover)]"
                        style={{
                          background: i === searchSelectedIdx ? "var(--tint-bg)" : "transparent",
                        }}
                      >
                        <AgentAvatar agent={r.agent} size={24} />
                        <div className="min-w-0 flex-1">
                          <span className="block truncate text-[13px] font-medium" style={{ color: "var(--fill-primary)" }}>
                            {r.agent.name}
                          </span>
                          {r.agent.tagline && (
                            <span className="block truncate text-[11px]" style={{ color: "var(--fill-quaternary)" }}>
                              {r.agent.tagline}
                            </span>
                          )}
                        </div>
                      </button>
                    ))
                  )}
                </div>
              )}
            </div>
            <button
              onClick={() => setShowNewForm(true)}
              disabled={creating}
              className="flex w-full cursor-pointer items-center justify-center gap-1.5 rounded-lg py-2 text-[12px] font-medium transition-colors duration-150 hover:bg-[var(--bg-hover)] disabled:opacity-50"
              style={{
                color: "var(--fill-tertiary)",
                border: "0.5px dashed var(--separator-opaque)",
              }}
            >
              <Plus {...ICON.sm} />
              新建 Agent
            </button>
          </>
        )}
      </div>

      {/* Agent Tree List */}
      <div className={`flex-1 overflow-x-hidden overflow-y-auto py-1.5 ${collapsed ? "flex flex-col items-center px-2" : "px-3"}`}>
        {!collapsed && agents.length === 0 && (
          <div
            className="mx-2 mt-3 rounded-[var(--radius-sm)] p-4"
            style={{ background: "var(--bg-hover)", border: "0.5px solid var(--separator)" }}
          >
            <div className="text-[13px] font-semibold" style={{ color: "var(--fill-primary)" }}>
              欢迎使用 FastClaw
            </div>
            <p className="mt-1 text-[12px] leading-5" style={{ color: "var(--fill-tertiary)" }}>
              先录入模型，再创建第一个 Agent。
            </p>
            <div className="mt-3 flex gap-2">
              <button
                onClick={handleQuickCreateDefault}
                className="cursor-pointer rounded-[var(--radius-xs)] px-3 py-1.5 text-[12px] font-medium"
                style={{ background: "var(--fill-primary)", color: "var(--fill-inverse)" }}
              >
                一键创建默认 Agent
              </button>
              <button
                onClick={toggleDetail}
                className="cursor-pointer rounded-[var(--radius-xs)] px-3 py-1.5 text-[12px] font-medium"
                style={{ background: "var(--bg-base)", color: "var(--fill-secondary)", border: "0.5px solid var(--separator-opaque)" }}
              >
                去配置模型
              </button>
            </div>
            {onboardingError && (
              <div className="mt-2 text-[11px]" style={{ color: "#ff453a" }}>{onboardingError}</div>
            )}
          </div>
        )}
        {agents.map((agent, i) => {
          const active = activeAgentId === agent.id;
          const expanded = expandedAgents.has(agent.id);
          const ac = agentChats[agent.id];
          const chatList = ac?.chatList ?? [];

          if (collapsed) {
            return (
              <button
                key={agent.id}
                onClick={() => { setActiveAgent(agent.id); newChat(agent.id); }}
                className="group relative mx-auto mb-1 flex h-9 w-9 items-center justify-center rounded-[var(--radius-xs)] hover:bg-[var(--bg-hover)]"
                style={{
                  background: active ? "var(--bg-active)" : "transparent",
                  animation: `slide-up var(--duration-slow) var(--ease-out) ${i * 0.04}s backwards`,
                }}
                title={agent.name}
              >
                <AgentAvatar agent={agent} size={28} />
              </button>
            );
          }

          return (
            <div
              key={agent.id}
              className="mb-0.5"
              style={{ animation: `fade-slide-up var(--duration-slow) var(--ease-out) ${i * 30}ms backwards` }}
            >
              {/* Agent Row */}
              <div
                className="group/item relative flex w-full items-center gap-2 rounded-lg px-2 py-1.5 transition-all duration-150 hover:bg-[var(--bg-hover)]"
                style={{
                  background: active ? "var(--tint-bg)" : "transparent",
                  borderRadius: "8px",
                  cursor: "pointer",
                }}
              >
                <div
                  className="flex min-w-0 flex-1 items-center gap-2"
                  onClick={() => {
                    setActiveAgent(agent.id);
                    newChat(agent.id);
                  }}
                >
                  <AgentAvatar agent={agent} size={34} />
                  <div className="min-w-0 flex-1">
                    <span className="block truncate text-[13px] font-semibold tracking-[-0.01em]" style={{ color: "var(--fill-primary)" }}>
                      {agent.name}
                    </span>
                    <span className="block truncate text-[11px]" style={{ color: "var(--fill-quaternary)" }}>
                      {agent.tagline}
                    </span>
                  </div>
                </div>

                <div className="flex shrink-0 items-center gap-0.5 opacity-0 transition-all duration-100 group-hover/item:opacity-100">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      const rect = (e.target as HTMLElement).getBoundingClientRect();
                      setContextMenu({ agentId: agent.id, x: rect.right, y: rect.bottom });
                    }}
                    className={BTN_ICON.sm}
                    style={{ color: "var(--fill-tertiary)" }}
                  >
                    <MoreHorizontal {...ICON.md} />
                  </button>
                  <button
                    onClick={(e) => { e.stopPropagation(); toggleExpand(agent.id); }}
                    className="flex h-5 w-5 items-center justify-center rounded transition-colors duration-100 hover:bg-[var(--bg-active)]"
                    style={{ color: "var(--fill-quaternary)" }}
                    title={expanded ? "收起会话" : "展开会话"}
                  >
                    {expanded ? <ChevronDown {...ICON.sm} /> : <ChevronRight {...ICON.sm} />}
                  </button>
                </div>
              </div>

              {/* Chat Sub-items */}
              {expanded && chatList.length > 0 && (
                <div className="mt-0.5 overflow-y-auto" style={{ maxHeight: 180 }}>
                  {chatList.map((chat) => {
                    const chatActive = active && ac?.activeChatId === chat.id;
                    return (
                      <ChatItem
                        key={chat.id}
                        chat={chat}
                        active={chatActive}
                        onClick={() => {
                          if (activeAgentId !== agent.id) setActiveAgent(agent.id);
                          setActiveChat(agent.id, chat.id);
                        }}
                      />
                    );
                  })}
                </div>
              )}
            </div>
          );
        })}
      </div>

      {/* Collapse button moved to search bar area */}

      {/* Context Menu */}
      {contextMenu && (
        <AgentContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          onClose={() => setContextMenu(null)}
          onPin={() => { /* placeholder */ }}
          onDetail={() => { setActiveAgent(contextMenu.agentId); toggleDetail(); }}
          onDelete={() => setDeleteConfirm(contextMenu.agentId)}
        />
      )}

      {/* Delete Confirmation */}
      {deleteConfirm && createPortal(
        <div className="fixed inset-0 z-50 flex items-center justify-center" style={{ animation: "fade-in var(--duration-fast) var(--ease-out)" }}>
          <div className="absolute inset-0" style={{ background: "rgba(0, 0, 0, 0.3)" }} onClick={() => setDeleteConfirm(null)} />
          <div
            className="relative w-full max-w-[340px] overflow-hidden rounded-[var(--radius-md)] px-6 py-5"
            style={{
              background: "var(--bg-elevated)",
              boxShadow: "var(--shadow-lg)",
              animation: "scale-in var(--duration-normal) var(--ease-out)",
              border: "0.5px solid var(--separator)",
            }}
          >
            <h3 className="mb-2 text-[14px] font-semibold" style={{ color: "var(--fill-primary)" }}>确认删除</h3>
            <p className="mb-4 text-[13px]" style={{ color: "var(--fill-tertiary)" }}>
              删除后所有会话数据将丢失，此操作不可撤销。
            </p>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setDeleteConfirm(null)}
                className="cursor-pointer rounded-[var(--radius-xs)] px-4 py-1.5 text-[12px] font-medium"
                style={{ color: "var(--fill-secondary)" }}
              >
                取消
              </button>
              <button
                onClick={() => handleDelete(deleteConfirm)}
                className="cursor-pointer rounded-[var(--radius-xs)] px-4 py-1.5 text-[12px] font-medium"
                style={{ background: "var(--red)", color: "#fff" }}
              >
                删除
              </button>
            </div>
          </div>
        </div>,
        document.body,
      )}

      {/* New Agent Modal */}
      {showNewForm && createPortal(
        <div className="fixed inset-0 z-50 flex items-center justify-center" style={{ animation: "fade-in var(--duration-fast) var(--ease-out)" }}>
          <div className="absolute inset-0" style={{ background: "rgba(0, 0, 0, 0.3)" }} onClick={cancelNew} />
          <div
            className="relative w-full max-w-[380px] overflow-hidden rounded-[var(--radius-md)]"
            style={{
              background: "var(--bg-elevated)",
              boxShadow: "var(--shadow-lg)",
              animation: "scale-in var(--duration-normal) var(--ease-out)",
              border: "0.5px solid var(--separator)",
            }}
          >
            <div className="flex items-center justify-between px-5 py-3.5" style={{ borderBottom: "0.5px solid var(--separator)" }}>
              <h3 className="text-[14px] font-semibold" style={{ color: "var(--fill-primary)" }}>新建 Agent</h3>
              <button onClick={cancelNew} className="flex h-6 w-6 cursor-pointer items-center justify-center rounded-full transition-colors duration-100 hover:bg-[var(--bg-hover)]" style={{ color: "var(--fill-tertiary)" }}>
                <X {...ICON.sm} />
              </button>
            </div>
            <div className="space-y-4 px-5 py-4">
              <div className="flex items-center gap-3">
                <button
                  type="button" onClick={pickAvatar}
                  className="group relative flex h-12 w-12 shrink-0 cursor-pointer items-center justify-center overflow-hidden rounded-full"
                  style={{ background: "var(--bg-tertiary)" }} title="选择头像"
                >
                  {newAvatarPreview ? (
                    <img src={newAvatarPreview} alt="" className="h-full w-full object-cover" />
                  ) : (
                    <span className="text-[14px] font-semibold" style={{ color: "var(--fill-tertiary)" }}>
                      {newName.trim() ? newName.trim().charAt(0).toUpperCase() : "?"}
                    </span>
                  )}
                  <div className="absolute inset-0 flex items-center justify-center rounded-full opacity-0 transition-opacity duration-100 group-hover:opacity-100" style={{ background: "rgba(0,0,0,0.3)" }}>
                    <Camera {...ICON.sm} color="white" />
                  </div>
                </button>
                <div className="min-w-0 flex-1">
                  <label className="mb-1 block text-[11px] font-medium" style={{ color: "var(--fill-tertiary)" }}>名称</label>
                  <input
                    ref={newInputRef} type="text" value={newName}
                    onChange={(e) => setNewName(e.target.value)}
                    onKeyDown={(e) => { if (e.key === "Enter" && newName.trim()) handleNewAgent(); if (e.key === "Escape") cancelNew(); }}
                    placeholder="输入 Agent 名称"
                    className="input-bordered w-full rounded-[var(--radius-xs)] px-3 py-2 text-[13px] outline-none transition-colors focus:outline-none"
                    style={{ background: "var(--bg-base)", color: "var(--fill-primary)", border: "0.5px solid var(--separator-opaque)" }}
                    disabled={creating}
                  />
                </div>
              </div>
              <div>
                <label className="mb-1 block text-[11px] font-medium" style={{ color: "var(--fill-tertiary)" }}>Agent ID</label>
                <input
                  type="text"
                  value={agentIdTouched ? newAgentId : (newName.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, ""))}
                  onChange={(e) => { setAgentIdTouched(true); setNewAgentId(e.target.value.toLowerCase().replace(/[^a-z0-9-_]/g, "")); }}
                  onFocus={() => { if (!agentIdTouched) { setNewAgentId(newName.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "")); setAgentIdTouched(true); } }}
                  placeholder="自动生成，或手动输入"
                  className="input-bordered w-full rounded-[var(--radius-xs)] px-3 py-2 text-[13px] outline-none transition-colors focus:outline-none"
                  style={{ background: "var(--bg-base)", color: "var(--fill-secondary)", border: "0.5px solid var(--separator-opaque)", fontFamily: "var(--font-mono, monospace)" }}
                  disabled={creating}
                />
                <span className="mt-0.5 block text-[10px]" style={{ color: "var(--fill-quaternary)" }}>
                  用于工作目录和文件标识，仅限小写字母、数字、连字符
                </span>
              </div>
              <div>
                <label className="mb-1 block text-[11px] font-medium" style={{ color: "var(--fill-tertiary)" }}>模型</label>
                <div className="relative">
                  <select value={newModel} onChange={(e) => setNewModel(e.target.value)} className="select-premium select-mono" disabled={creating || modelsLoading || models.length === 0}>
                    {modelsLoading && <option value="">加载中...</option>}
                    {!modelsLoading && models.length === 0 && <option value="">暂无可用模型</option>}
                    {models.map((m) => <option key={`${m.provider}/${m.model}`} value={m.model}>{m.model}</option>)}
                  </select>
                  <ChevronDown {...ICON.sm} className="pointer-events-none absolute top-1/2 right-3 -translate-y-1/2" style={{ color: "var(--fill-tertiary)" }} />
                </div>
              </div>
            </div>
            <div className="flex items-center justify-end gap-2 px-5 py-3.5" style={{ borderTop: "0.5px solid var(--separator)" }}>
              <button onClick={cancelNew} className="cursor-pointer rounded-[var(--radius-xs)] px-4 py-1.5 text-[12px] font-medium transition-colors duration-100" style={{ color: "var(--fill-secondary)" }}>取消</button>
              <button
                onClick={handleNewAgent} disabled={creating || !newName.trim()}
                className="flex cursor-pointer items-center gap-1 rounded-[var(--radius-xs)] px-4 py-1.5 text-[12px] font-medium transition-opacity duration-100 hover:opacity-90 disabled:opacity-40"
                style={{ background: "var(--fill-primary)", color: "var(--fill-inverse)" }}
              >
                <Check {...ICON.sm} />
                {creating ? "创建中..." : "创建"}
              </button>
            </div>
          </div>
        </div>,
        document.body,
      )}
    </aside>
  );
}
