import { create } from "zustand";

export interface PtySession {
  id: string;
  chatId?: string;
  status: "connecting" | "connected" | "closed";
  name?: string;
  cwd?: string;
  exitCode?: number;
  source?: "user" | "agent";
}

interface PtyState {
  sessions: PtySession[];
  activeSessionId: string | null;

  addSession: (session: PtySession) => void;
  updateSession: (id: string, patch: Partial<PtySession>) => void;
  removeSession: (id: string) => void;
  setActiveSession: (id: string | null) => void;
  getSessionsForChat: (chatId: string) => PtySession[];
}

let sessionCounter = 0;

export const usePtyStore = create<PtyState>((set, get) => ({
  sessions: [],
  activeSessionId: null,

  addSession: (session) => {
    sessionCounter++;
    const named = { ...session, name: session.name ?? `Shell ${sessionCounter}` };
    set((s) => ({
      sessions: [...s.sessions, named],
      activeSessionId: named.id,
    }));
  },

  updateSession: (id, patch) => {
    set((s) => ({
      sessions: s.sessions.map((sess) =>
        sess.id === id ? { ...sess, ...patch } : sess
      ),
    }));
  },

  removeSession: (id) => {
    set((s) => {
      const sessions = s.sessions.filter((sess) => sess.id !== id);
      const activeSessionId =
        s.activeSessionId === id
          ? sessions[sessions.length - 1]?.id ?? null
          : s.activeSessionId;
      return { sessions, activeSessionId };
    });
  },

  setActiveSession: (id) => {
    set({ activeSessionId: id });
  },

  getSessionsForChat: (chatId) => {
    return get().sessions.filter((s) => !s.chatId || s.chatId === chatId);
  },
}));
