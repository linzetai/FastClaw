import { create } from "zustand";
import * as api from "../api";

export interface DisplayConfig {
  toolCallGroupThreshold: number;
}

export interface ConfigStoreState {
  display: DisplayConfig;
  setDisplayConfig: (partial: Partial<DisplayConfig>) => void;
  loadDisplayConfig: () => Promise<void>;
}

const DEFAULT_DISPLAY: DisplayConfig = {
  toolCallGroupThreshold: 3,
};

export const useConfigStore = create<ConfigStoreState>((set, get) => ({
  display: { ...DEFAULT_DISPLAY },

  setDisplayConfig: (partial) => {
    const next = { ...get().display, ...partial };
    set({ display: next });
    api.setConfig("display", next).catch(() => {});
  },

  loadDisplayConfig: async () => {
    try {
      const data = await api.getConfig("display");
      const cfg = (data as { key?: string; value?: Partial<DisplayConfig> } | null);
      const val = cfg?.value ?? cfg;
      if (val && typeof val === "object" && "toolCallGroupThreshold" in val) {
        set({ display: { ...DEFAULT_DISPLAY, ...val as Partial<DisplayConfig> } });
      }
    } catch { /* use defaults */ }
  },
}));
