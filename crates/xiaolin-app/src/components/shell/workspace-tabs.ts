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

const isTauri =
  typeof window !== "undefined" &&
  ("__TAURI_INTERNALS__" in window || "__TAURI__" in window);

const PANEL_WIDTH = 360;

async function resizeWindowForPanel(opening: boolean, prePanelWidth: number | null): Promise<number | null> {
  if (!isTauri) return null;

  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const { currentMonitor } = await import("@tauri-apps/api/window");
    const win = getCurrentWindow();

    if (await win.isMaximized()) return null;

    const size = await win.innerSize();
    const pos = await win.outerPosition();
    const monitor = await currentMonitor();

    if (opening) {
      if (monitor) {
        const availableRight = monitor.position.x + monitor.size.width;
        const windowRight = pos.x + size.width + PANEL_WIDTH;
        if (windowRight > availableRight) return null;
      }
      const savedWidth = size.width;
      await win.setSize(new (await import("@tauri-apps/api/dpi")).LogicalSize(
        size.toLogical((await win.scaleFactor())).width + PANEL_WIDTH,
        size.toLogical((await win.scaleFactor())).height,
      ));
      return savedWidth;
    } else {
      if (prePanelWidth != null) {
        const scale = await win.scaleFactor();
        const logicalSize = size.toLogical(scale);
        await win.setSize(new (await import("@tauri-apps/api/dpi")).LogicalSize(
          logicalSize.width - PANEL_WIDTH,
          logicalSize.height,
        ));
      }
      return null;
    }
  } catch {
    return null;
  }
}

interface WorkspaceTabsState {
  tabs: WorkspaceTab[];
  activeTabId: string | null;
  panelOpen: boolean;
  prePanelWidth: number | null;

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
  prePanelWidth: null,

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
      if (!panelOpen) {
        set({ activeTabId: id, panelOpen: true });
        resizeWindowForPanel(true, null).then((saved) => {
          if (saved != null) set({ prePanelWidth: saved });
        });
      } else {
        set({ activeTabId: id });
      }
    }
  },

  setPanelOpen: (open) => {
    const { panelOpen, prePanelWidth } = get();
    if (open === panelOpen) return;
    set({ panelOpen: open });
    if (open) {
      resizeWindowForPanel(true, null).then((saved) => {
        if (saved != null) set({ prePanelWidth: saved });
      });
    } else {
      resizeWindowForPanel(false, prePanelWidth).then(() => {
        set({ prePanelWidth: null });
      });
    }
  },

  togglePanel: () => {
    const { panelOpen, prePanelWidth } = get();
    const next = !panelOpen;
    set({ panelOpen: next });
    if (next) {
      resizeWindowForPanel(true, null).then((saved) => {
        if (saved != null) set({ prePanelWidth: saved });
      });
    } else {
      resizeWindowForPanel(false, prePanelWidth).then(() => {
        set({ prePanelWidth: null });
      });
    }
  },
}));
