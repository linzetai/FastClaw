import { create } from "zustand";
import * as transport from "../transport";

export type TokenUsageDaily = transport.TokenUsageDailyData;
export type ToolCallDaily = transport.ToolCallDailyData;
export type CostSummary = transport.CostSummaryData;
export type SessionCostSummary = transport.SessionCostData;

export interface CostStoreState {
  summary: CostSummary | null;
  dailyTokens: TokenUsageDaily[];
  toolStats: ToolCallDaily[];
  sessions: SessionCostSummary[];
  loading: boolean;
  error: string | null;
  fetchSummary: () => Promise<void>;
  fetchDailyTokens: (start?: string, end?: string) => Promise<void>;
  fetchToolStats: (start?: string, end?: string) => Promise<void>;
  fetchSessions: (limit?: number) => Promise<void>;
  fetchAll: (start?: string, end?: string) => Promise<void>;
}

export const useCostStore = create<CostStoreState>((set) => ({
  summary: null,
  dailyTokens: [],
  toolStats: [],
  sessions: [],
  loading: false,
  error: null,

  fetchSummary: async () => {
    try {
      const data = await transport.costSummary();
      set({ summary: data });
    } catch (e: unknown) {
      set({ error: (e as Error).message });
    }
  },

  fetchDailyTokens: async (start?: string, end?: string) => {
    try {
      const data = await transport.costDaily(start, end);
      set({ dailyTokens: data });
    } catch (e: unknown) {
      set({ error: (e as Error).message });
    }
  },

  fetchToolStats: async (start?: string, end?: string) => {
    try {
      const data = await transport.costTools(start, end);
      set({ toolStats: data });
    } catch (e: unknown) {
      set({ error: (e as Error).message });
    }
  },

  fetchSessions: async (limit = 20) => {
    try {
      const data = await transport.costSessions(limit);
      set({ sessions: data });
    } catch (e: unknown) {
      set({ error: (e as Error).message });
    }
  },

  fetchAll: async (start?: string, end?: string) => {
    set({ loading: true, error: null });
    try {
      const [summaryData, dailyData, toolData, sessionData] = await Promise.all([
        transport.costSummary(),
        transport.costDaily(start, end),
        transport.costTools(start, end),
        transport.costSessions(20),
      ]);

      set({
        summary: summaryData,
        dailyTokens: dailyData,
        toolStats: toolData,
        sessions: sessionData,
        loading: false,
      });
    } catch (e: unknown) {
      set({ loading: false, error: (e as Error).message });
    }
  },
}));
