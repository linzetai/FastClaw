import { create } from "zustand";

export type NavItem = "chat" | "workspace" | "tasks" | "files" | "connections";
export type LayoutTier = "compact" | "standard" | "wide";

const SIDEBAR_WIDTH_KEY = "xiaolin:sidebar-width";
const DEFAULT_SIDEBAR_WIDTH = 240;
const MIN_SIDEBAR_WIDTH = 180;
const MAX_SIDEBAR_WIDTH = 400;

function loadSidebarWidth(): number {
  try {
    const raw = localStorage.getItem(SIDEBAR_WIDTH_KEY);
    if (!raw) return DEFAULT_SIDEBAR_WIDTH;
    const val = Number(raw);
    if (Number.isFinite(val) && val >= MIN_SIDEBAR_WIDTH && val <= MAX_SIDEBAR_WIDTH) return val;
  } catch { /* ignore */ }
  return DEFAULT_SIDEBAR_WIDTH;
}

export interface UIState {
  detailOpen: boolean;
  sidebarCollapsed: boolean;
  sidebarWidth: number;
  activeNav: NavItem;
  layoutTier: LayoutTier;

  toggleDetail: () => void;
  closeDetail: () => void;
  toggleSidebar: () => void;
  setSidebarWidth: (w: number) => void;
  resetSidebarWidth: () => void;
  setActiveNav: (nav: NavItem) => void;
  setLayoutTier: (tier: LayoutTier) => void;
}

export { DEFAULT_SIDEBAR_WIDTH, MIN_SIDEBAR_WIDTH, MAX_SIDEBAR_WIDTH };

export const useUIStore = create<UIState>((set) => ({
  detailOpen: false,
  sidebarCollapsed: false,
  sidebarWidth: loadSidebarWidth(),
  activeNav: "chat" as NavItem,
  layoutTier: "standard" as LayoutTier,

  toggleDetail: () => set((s) => ({ detailOpen: !s.detailOpen })),
  closeDetail: () => set({ detailOpen: false }),
  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
  setSidebarWidth: (w) => {
    const clamped = Math.round(Math.min(MAX_SIDEBAR_WIDTH, Math.max(MIN_SIDEBAR_WIDTH, w)));
    try { localStorage.setItem(SIDEBAR_WIDTH_KEY, String(clamped)); } catch { /* ignore */ }
    set({ sidebarWidth: clamped });
  },
  resetSidebarWidth: () => {
    try { localStorage.setItem(SIDEBAR_WIDTH_KEY, String(DEFAULT_SIDEBAR_WIDTH)); } catch { /* ignore */ }
    set({ sidebarWidth: DEFAULT_SIDEBAR_WIDTH });
  },
  setActiveNav: (nav) => set({ activeNav: nav }),
  setLayoutTier: (tier) => set({ layoutTier: tier }),
}));
