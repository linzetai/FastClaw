import { create } from "zustand";
import * as transport from "./transport";
import { useAgentStore } from "./agent-store";

export interface GatewayInfo {
  port: number;
  wsUrl: string;
  httpUrl: string;
  version: string;
}

interface GatewayState {
  mode: "ready" | "connecting" | "browser";
  info: GatewayInfo | null;
  connected: boolean;
  error: string | null;

  init: () => Promise<void>;
  setConnected: (v: boolean) => void;
}

let disconnectUnsub: (() => void) | null = null;
let reconnectedUnsub: (() => void) | null = null;
let sessionChangedUnsub: (() => void) | null = null;

async function syncBackendData() {
  try {
    const [agents, sessions] = await Promise.all([
      transport.listAgents(),
      transport.listSessions(50),
    ]);
    if (agents.length > 0) {
      useAgentStore.getState().syncAgentsFromBackend(agents);
      for (const agent of agents) {
        const agentSessions = sessions.filter(
          (s) => s.agentId === agent.agentId,
        );
        if (agentSessions.length > 0) {
          useAgentStore
            .getState()
            .syncSessionsForAgent(agent.agentId, agentSessions);
        }
      }
    }
  } catch {
    /* sync failure is non-fatal */
  }
}

export const useGatewayStore = create<GatewayState>((set) => ({
  mode: "connecting",
  info: null,
  connected: false,
  error: null,

  init: async () => {
    try {
      disconnectUnsub?.();
      reconnectedUnsub?.();
      sessionChangedUnsub?.();

      set({ mode: "connecting", error: null });

      if (transport.isTauri) {
        // Tauri mode: get gateway info from IPC, then connect via WebSocket
        const info = await transport.getGatewayInfo();
        if (!info) {
          throw new Error("Gateway not started");
        }

        // Always use WebSocket for communication
        disconnectUnsub = transport.onWsEvent("disconnected", () => {
          set({ connected: false });
        });
        reconnectedUnsub = transport.onWsEvent("reconnected", () => {
          set({ connected: true });
          syncBackendData();
        });

        try {
          await transport.connectWs(info.wsUrl);
          set({ mode: "ready", info, connected: true, error: null });
        } catch (e) {
          console.warn("WS connect failed:", e);
          set({ mode: "ready", info, connected: false, error: String(e) });
          return;
        }

        sessionChangedUnsub = transport.onSessionChanged(async (sid) => {
          try {
            const session = await transport.getSession(sid);
            if (session?.title) {
              const store = useAgentStore.getState();
              const agentId = session.agentId || store.activeAgentId;
              store.renameChat(agentId, sid, session.title);
            }
          } catch {
            /* ignore */
          }
        });

        await syncBackendData();
      } else {
        // Browser mode: check for gateway health endpoint
        const info = await fetchBrowserGatewayInfo();
        if (!info.wsUrl) {
          set({ mode: "browser", info: null, connected: false });
          return;
        }

        disconnectUnsub = transport.onWsEvent("disconnected", () => {
          set({ connected: false });
        });

        reconnectedUnsub = transport.onWsEvent("reconnected", () => {
          set({ connected: true });
          syncBackendData();
        });

        try {
          await transport.connectWs(info.wsUrl);
          set({ mode: "ready", info, connected: true });

          sessionChangedUnsub = transport.onSessionChanged(async (sid) => {
            try {
              const session = await transport.getSession(sid);
              if (session?.title) {
                const store = useAgentStore.getState();
                const agentId = session.agentId || store.activeAgentId;
                store.renameChat(agentId, sid, session.title);
              }
            } catch {
              /* ignore */
            }
          });

          await syncBackendData();
        } catch (e) {
          console.warn("WS connect failed:", e);
          set({ mode: "browser", info, connected: false });
        }
      }
    } catch (e) {
      set({ mode: "connecting", error: String(e) });
    }
  },

  setConnected: (v) => set({ connected: v }),
}));

async function fetchBrowserGatewayInfo(): Promise<GatewayInfo> {
  const port = 18888;
  const httpUrl = `http://127.0.0.1:${port}`;
  try {
    const resp = await fetch(`${httpUrl}/health`);
    if (resp.ok) {
      return {
        port,
        wsUrl: `ws://127.0.0.1:${port}/ws`,
        httpUrl,
        version: "dev",
      };
    }
  } catch {
    // gateway not running
  }
  return { port: 0, wsUrl: "", httpUrl: "", version: "dev-browser" };
}