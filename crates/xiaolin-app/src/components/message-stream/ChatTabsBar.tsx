import { useState, useRef, useMemo, useCallback, useEffect } from "react";
import { createPortal } from "react-dom";
import { X, Plus, ChevronLeft, ChevronRight, PanelLeftOpen, Search, MessageSquare, Menu, Layout, ListTodo, FolderOpen, Link2 } from "lucide-react";
import { useChatMetaStore } from "../../lib/stores";
import { useUIStore } from "../../lib/stores";
import type { ChatMeta } from "../../lib/stores/types";
import type { NavItem } from "../../lib/stores/ui-store";
import { ICON, BTN_ICON } from "../../lib/ui-tokens";

export interface ChatTabsBarViewProps {
  chats: ChatMeta[];
  activeChatId: string;
  streamingChatIds: Set<string>;
  attentionChatIds?: Set<string>;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
  onNew: () => void;
  onRename: (id: string, title: string) => void;
  onReorder: (fromIdx: number, toIdx: number) => void;
}

function TabItem({
  chat, isActive, isStreaming, needsAttention,
  editingId, editValue, editRef,
  onSelect, onClose, onDoubleClick,
  onEditChange, onCommitRename, onCancelEdit,
  onDragStart, onDragOver, onDrop,
}: {
  chat: ChatMeta; isActive: boolean; isStreaming: boolean; needsAttention: boolean;
  editingId: string | null; editValue: string;
  editRef: React.RefObject<HTMLInputElement | null>;
  onSelect: (id: string) => void; onClose: (id: string) => void;
  onDoubleClick: (chat: ChatMeta) => void;
  onEditChange: (v: string) => void; onCommitRename: () => void; onCancelEdit: () => void;
  onDragStart: (e: React.DragEvent, chat: ChatMeta) => void;
  onDragOver: (e: React.DragEvent) => void;
  onDrop: (e: React.DragEvent, chat: ChatMeta) => void;
}) {
  const [hovered, setHovered] = useState(false);
  const isEditing = editingId === chat.id;

  return (
    <div
      className="group relative flex shrink-0 items-center gap-1.5 rounded-md px-2.5 text-[12px]"
      style={{
        height: 28,
        background: isActive ? "var(--tint-bg)" : hovered ? "var(--bg-hover)" : "transparent",
        cursor: isEditing ? "text" : "pointer",
        transition: "background var(--duration-fast) var(--ease-in-out)",
        borderRight: "0.5px solid var(--separator)",
        maxWidth: 180,
      }}
      draggable={!isEditing}
      onClick={() => { if (!isEditing) onSelect(chat.id); }}
      onDoubleClick={() => onDoubleClick(chat)}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onDragStart={(e) => onDragStart(e, chat)}
      onDragOver={onDragOver}
      onDrop={(e) => onDrop(e, chat)}
    >
      {needsAttention && (
        <span
          className="inline-block h-[5px] w-[5px] shrink-0 rounded-full"
          style={{ background: "var(--warning, #f59e0b)", animation: "pulse-subtle 1.5s ease-in-out infinite" }}
        />
      )}
      {!needsAttention && isStreaming && (
        <span
          className="inline-block h-[5px] w-[5px] shrink-0 rounded-full"
          style={{ background: "var(--tint)", animation: "pulse-subtle 1.5s ease-in-out infinite" }}
        />
      )}

      {isEditing ? (
        <input
          ref={editRef}
          value={editValue}
          onChange={(e) => onEditChange(e.target.value)}
          onBlur={onCommitRename}
          onKeyDown={(e) => {
            if (e.key === "Enter") onCommitRename();
            if (e.key === "Escape") onCancelEdit();
          }}
          className="min-w-0 flex-1 rounded-sm bg-transparent px-0.5 text-[12px] font-medium outline-none ring-1 ring-[var(--tint)]"
          style={{ color: "var(--fill-primary)" }}
          onClick={(e) => e.stopPropagation()}
        />
      ) : (
        <span
          className="min-w-0 flex-1 truncate"
          style={{ color: isActive ? "var(--fill-primary)" : "var(--fill-secondary)", fontWeight: isActive ? 600 : 400 }}
        >
          {chat.title || "新对话"}
        </span>
      )}

      <button
        onClick={(e) => { e.stopPropagation(); onClose(chat.id); }}
        className="flex h-4 w-4 shrink-0 items-center justify-center rounded-sm transition-opacity"
        style={{ color: "var(--fill-tertiary)", opacity: hovered || isActive ? 1 : 0 }}
      >
        <X size={10} strokeWidth={1.5} />
      </button>
    </div>
  );
}

const OVERFLOW_THRESHOLD = 8;

const NAV_ITEMS: { id: NavItem; icon: React.ComponentType<{ size?: number; strokeWidth?: number }>; label: string }[] = [
  { id: "chat", icon: MessageSquare, label: "对话" },
  { id: "workspace", icon: Layout, label: "工作室" },
  { id: "tasks", icon: ListTodo, label: "任务" },
  { id: "files", icon: FolderOpen, label: "文件" },
  { id: "connections", icon: Link2, label: "连接" },
];

export function ChatTabsBarView({
  chats, activeChatId, streamingChatIds, attentionChatIds,
  onSelect, onClose, onNew, onRename, onReorder,
}: ChatTabsBarViewProps) {
  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUIStore((s) => s.toggleSidebar);
  const layoutTier = useUIStore((s) => s.layoutTier);
  const activeNav = useUIStore((s) => s.activeNav);
  const setActiveNav = useUIStore((s) => s.setActiveNav);

  const isCompact = layoutTier === "compact";
  const [navMenuOpen, setNavMenuOpen] = useState(false);
  const navMenuBtnRef = useRef<HTMLButtonElement>(null);

  const [editingId, setEditingId] = useState<string | null>(null);
  const [editValue, setEditValue] = useState("");
  const editRef = useRef<HTMLInputElement>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const [showLeft, setShowLeft] = useState(false);
  const [showRight, setShowRight] = useState(false);

  const [overflowOpen, setOverflowOpen] = useState(false);
  const overflowRef = useRef<HTMLDivElement>(null);
  const overflowBtnRef = useRef<HTMLButtonElement>(null);

  const openChats = useMemo(() => chats.filter((c) => c.open), [chats]);
  const visibleChats = useMemo(() => openChats.slice(0, OVERFLOW_THRESHOLD), [openChats]);
  const overflowChats = useMemo(() => openChats.slice(OVERFLOW_THRESHOLD), [openChats]);
  const hasOverflow = overflowChats.length > 0;

  const dragIdRef = useRef<string | null>(null);

  const updateScrollArrows = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    setShowLeft(el.scrollLeft > 2);
    setShowRight(el.scrollLeft + el.clientWidth < el.scrollWidth - 2);
  }, []);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    updateScrollArrows();
    const ro = new ResizeObserver(updateScrollArrows);
    ro.observe(el);
    return () => ro.disconnect();
  }, [updateScrollArrows, openChats.length]);

  useEffect(() => {
    if (!overflowOpen && !navMenuOpen) return;
    const handler = (e: MouseEvent) => {
      if (overflowOpen) {
        if (overflowRef.current?.contains(e.target as Node)) return;
        if (overflowBtnRef.current?.contains(e.target as Node)) return;
        setOverflowOpen(false);
      }
      if (navMenuOpen) {
        if (navMenuBtnRef.current?.contains(e.target as Node)) return;
        setNavMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [overflowOpen, navMenuOpen]);

  const scrollLeft = useCallback(() => {
    scrollRef.current?.scrollBy({ left: -150, behavior: "smooth" });
  }, []);

  const scrollRight = useCallback(() => {
    scrollRef.current?.scrollBy({ left: 150, behavior: "smooth" });
  }, []);

  const handleDblClick = useCallback((chat: ChatMeta) => {
    setEditingId(chat.id);
    setEditValue(chat.title);
    setTimeout(() => editRef.current?.select(), 0);
  }, []);

  const commitRename = useCallback(() => {
    if (editingId && editValue.trim()) onRename(editingId, editValue.trim());
    setEditingId(null);
  }, [editingId, editValue, onRename]);

  const cancelEdit = useCallback(() => setEditingId(null), []);

  const handleSelect = useCallback((id: string) => {
    onSelect(id);
    setOverflowOpen(false);
  }, [onSelect]);

  const handleNew = useCallback(() => {
    onNew();
    setOverflowOpen(false);
  }, [onNew]);

  const handleSearchClick = useCallback(() => {
    window.dispatchEvent(new CustomEvent("xiaolin:toggle-search"));
  }, []);

  const handleDragStart = useCallback((e: React.DragEvent, chat: ChatMeta) => {
    dragIdRef.current = chat.id;
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", chat.id);
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
  }, []);

  const handleDrop = useCallback((e: React.DragEvent, target: ChatMeta) => {
    e.preventDefault();
    const fromId = dragIdRef.current;
    if (!fromId || fromId === target.id) return;
    const fromIdx = openChats.findIndex((c) => c.id === fromId);
    const toIdx = openChats.findIndex((c) => c.id === target.id);
    if (fromIdx >= 0 && toIdx >= 0) onReorder(fromIdx, toIdx);
    dragIdRef.current = null;
  }, [openChats, onReorder]);

  const overflowPos = useMemo(() => {
    if (!overflowOpen || !overflowBtnRef.current) return { top: 0, right: 0 };
    const rect = overflowBtnRef.current.getBoundingClientRect();
    return { top: rect.bottom + 4, right: window.innerWidth - rect.right };
  }, [overflowOpen]);

  return (
    <div
      className="flex shrink-0 items-center"
      style={{
        height: 36,
        borderBottom: "0.5px solid var(--separator)",
        background: "var(--bg-primary)",
      }}
    >
      {isCompact && (
        <>
          <button
            ref={navMenuBtnRef}
            onClick={() => setNavMenuOpen((v) => !v)}
            className={`${BTN_ICON.sm} ml-2`}
            style={{ color: navMenuOpen ? "var(--tint)" : "var(--fill-tertiary)" }}
            title="导航菜单"
          >
            <Menu size={16} strokeWidth={1.5} />
          </button>
          {navMenuOpen && createPortal(
            <div
              className="fixed rounded-lg py-1"
              style={{
                top: (navMenuBtnRef.current?.getBoundingClientRect().bottom ?? 0) + 4,
                left: (navMenuBtnRef.current?.getBoundingClientRect().left ?? 0),
                zIndex: 9999,
                minWidth: 160,
                background: "var(--bg-elevated)",
                border: "0.5px solid var(--border-subtle)",
                boxShadow: "var(--shadow-lg)",
                animation: "scale-spring var(--duration-normal) var(--ease-spring-subtle)",
                transformOrigin: "top left",
              }}
            >
              {NAV_ITEMS.map((item) => {
                const Icon = item.icon;
                const active = activeNav === item.id;
                return (
                  <button
                    key={item.id}
                    onClick={() => { setActiveNav(item.id); setNavMenuOpen(false); }}
                    className="flex w-full items-center gap-2.5 px-3 py-2 text-[12px] transition-colors hover:bg-[var(--bg-hover)]"
                    style={{ color: active ? "var(--tint)" : "var(--fill-secondary)", fontWeight: active ? 600 : 400 }}
                  >
                    <Icon size={14} strokeWidth={active ? 2 : 1.5} />
                    <span>{item.label}</span>
                  </button>
                );
              })}
              <div className="mx-2 my-1 h-px" style={{ background: "var(--separator)" }} />
              <button
                onClick={() => { toggleSidebar(); setNavMenuOpen(false); }}
                className="flex w-full items-center gap-2.5 px-3 py-2 text-[12px] transition-colors hover:bg-[var(--bg-hover)]"
                style={{ color: "var(--fill-secondary)" }}
              >
                <PanelLeftOpen size={14} strokeWidth={1.5} />
                <span>{sidebarCollapsed ? "显示会话列表" : "隐藏会话列表"}</span>
              </button>
            </div>,
            document.body,
          )}
        </>
      )}

      {!isCompact && sidebarCollapsed && (
        <button
          onClick={toggleSidebar}
          className={`${BTN_ICON.sm} ml-2`}
          style={{ color: "var(--fill-tertiary)" }}
          title="展开侧边栏"
        >
          <PanelLeftOpen size={16} strokeWidth={1.2} />
        </button>
      )}

      {showLeft && (
        <button onClick={scrollLeft} className={`${BTN_ICON.sm} shrink-0`} style={{ color: "var(--fill-quaternary)" }}>
          <ChevronLeft size={14} />
        </button>
      )}

      <div
        ref={scrollRef}
        className="flex min-w-0 flex-1 items-center"
        style={{ overflowX: "auto", scrollbarWidth: "none" }}
        onScroll={updateScrollArrows}
      >
        {visibleChats.map((chat) => (
          <TabItem
            key={chat.id}
            chat={chat}
            isActive={chat.id === activeChatId}
            isStreaming={streamingChatIds.has(chat.id)}
            needsAttention={attentionChatIds?.has(chat.id) ?? false}
            editingId={editingId}
            editValue={editValue}
            editRef={editRef}
            onSelect={handleSelect}
            onClose={onClose}
            onDoubleClick={handleDblClick}
            onEditChange={setEditValue}
            onCommitRename={commitRename}
            onCancelEdit={cancelEdit}
            onDragStart={handleDragStart}
            onDragOver={handleDragOver}
            onDrop={handleDrop}
          />
        ))}
      </div>

      {showRight && (
        <button onClick={scrollRight} className={`${BTN_ICON.sm} shrink-0`} style={{ color: "var(--fill-quaternary)" }}>
          <ChevronRight size={14} />
        </button>
      )}

      {hasOverflow && (
        <div className="relative shrink-0">
          <button
            ref={overflowBtnRef}
            onClick={() => setOverflowOpen(!overflowOpen)}
            className="flex h-[26px] items-center rounded-md px-1.5 text-[11px] font-medium transition-colors hover:bg-[var(--bg-hover)]"
            style={{ color: "var(--fill-tertiary)" }}
          >
            +{overflowChats.length}
          </button>
          {overflowOpen && createPortal(
            <div
              ref={overflowRef}
              className="fixed min-w-[200px] max-w-[280px] rounded-lg p-1"
              style={{
                top: overflowPos.top,
                right: overflowPos.right,
                zIndex: 9999,
                background: "var(--bg-elevated)",
                border: "0.5px solid var(--border-subtle)",
                boxShadow: "var(--shadow-lg)",
                animation: "scale-spring var(--duration-normal) var(--ease-spring-subtle)",
                transformOrigin: "top right",
              }}
            >
              {overflowChats.map((chat) => (
                <button
                  key={chat.id}
                  onClick={() => handleSelect(chat.id)}
                  className="flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-[12px] transition-colors hover:bg-[var(--bg-hover)]"
                  style={{ color: chat.id === activeChatId ? "var(--fill-primary)" : "var(--fill-secondary)" }}
                >
                  <MessageSquare {...ICON.sm} style={{ color: "var(--fill-quaternary)", flexShrink: 0 }} />
                  <span className="min-w-0 flex-1 truncate">{chat.title || "新对话"}</span>
                  {streamingChatIds.has(chat.id) && (
                    <span className="inline-block h-[5px] w-[5px] shrink-0 rounded-full" style={{ background: "var(--tint)", animation: "pulse-subtle 1.5s ease-in-out infinite" }} />
                  )}
                  {attentionChatIds?.has(chat.id) && (
                    <span className="inline-block h-[5px] w-[5px] shrink-0 rounded-full" style={{ background: "var(--warning, #f59e0b)", animation: "pulse-subtle 1.5s ease-in-out infinite" }} />
                  )}
                </button>
              ))}
            </div>,
            document.body,
          )}
        </div>
      )}

      <div className="mx-1 h-4 w-px shrink-0" style={{ background: "var(--separator)" }} />

      <button
        onClick={handleNew}
        className={`${BTN_ICON.sm} shrink-0`}
        style={{ color: "var(--tint)" }}
        title="新建会话"
      >
        <Plus size={14} strokeWidth={2} />
      </button>

      <div className="flex-1" />

      <button
        onClick={handleSearchClick}
        className={`${BTN_ICON.sm} mr-2 shrink-0`}
        style={{ color: "var(--fill-quaternary)" }}
        title="搜索"
      >
        <Search size={14} strokeWidth={1.2} />
      </button>
    </div>
  );
}

export interface ChatTabsBarProps {
  streamingChatIds: Set<string>;
  attentionChatIds?: Set<string>;
}

export function ChatTabsBar({ streamingChatIds, attentionChatIds }: ChatTabsBarProps) {
  const chatsRecord = useChatMetaStore((s) => s.chats);
  const chatOrder = useChatMetaStore((s) => s.chatOrder);
  const activeChatId = useChatMetaStore((s) => s.activeChatId);
  const setActiveChat = useChatMetaStore((s) => s.setActiveChat);
  const closeChat = useChatMetaStore((s) => s.closeChat);
  const newChat = useChatMetaStore((s) => s.newChat);
  const renameChat = useChatMetaStore((s) => s.renameChat);
  const reorderChats = useChatMetaStore((s) => s.reorderChats);

  const chats = useMemo(
    () => chatOrder.map((id) => chatsRecord[id]).filter((c): c is ChatMeta => c != null),
    [chatsRecord, chatOrder],
  );

  return (
    <ChatTabsBarView
      chats={chats}
      activeChatId={activeChatId}
      streamingChatIds={streamingChatIds}
      attentionChatIds={attentionChatIds}
      onSelect={setActiveChat}
      onClose={closeChat}
      onNew={() => newChat()}
      onRename={renameChat}
      onReorder={reorderChats}
    />
  );
}
