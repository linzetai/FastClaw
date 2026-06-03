import { create } from "zustand";
import type { ComponentType } from "react";

export interface WorkspaceTab {
  id: string;
  label: string;
  icon: ComponentType<{ size?: number; strokeWidth?: number }>;
  component: ComponentType;
  footerComponent?: ComponentType;
  badge?: number | boolean;
  order?: number;
}

interface WorkspaceTabsState {
  tabs: WorkspaceTab[];
  activeTabId: string | null;
  panelOpen: boolean;

  registerTab: (tab: WorkspaceTab) => void;
  unregisterTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  setPanelOpen: (open: boolean) => void;
  togglePanel: () => void;
}

export const useWorkspaceTabs = create<WorkspaceTabsState>((set, get) => ({
  tabs: [],
  activeTabId: null,
  panelOpen: false,

  registerTab: (tab) => {
    set((s) => {
      if (s.tabs.some((t) => t.id === tab.id)) return s;
      const tabs = [...s.tabs, tab].sort((a, b) => (a.order ?? 99) - (b.order ?? 99));
      return { tabs, activeTabId: s.activeTabId ?? tab.id };
    });
  },

  unregisterTab: (id) => {
    set((s) => {
      const tabs = s.tabs.filter((t) => t.id !== id);
      const activeTabId =
        s.activeTabId === id ? (tabs[0]?.id ?? null) : s.activeTabId;
      return { tabs, activeTabId };
    });
  },

  setActiveTab: (id) => {
    const { tabs, panelOpen } = get();
    if (tabs.some((t) => t.id === id)) {
      set({ activeTabId: id, panelOpen: panelOpen || true });
    }
  },

  setPanelOpen: (open) => set({ panelOpen: open }),
  togglePanel: () => set((s) => ({ panelOpen: !s.panelOpen })),
}));
