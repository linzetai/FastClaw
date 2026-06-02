import { DEFAULT_AGENT_ID } from "./chat-helpers";

const STORAGE_KEY = "xiaolin:ui-state";
const UI_STATE_VERSION = 2;

export interface PersistedUIState {
  version: number;
  activeAgentId: string;
  agentActiveChats: Record<string, string>;
  agentOpenChats: Record<string, string[]>;
}

export function loadUIState(): PersistedUIState | null {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as PersistedUIState;
    if (parsed.version !== UI_STATE_VERSION) return null;
    return parsed;
  } catch {
    return null;
  }
}

export function saveUIStateFromMeta(activeChatId: string, chats: Record<string, { open: boolean }>) {
  try {
    const openIds = Object.entries(chats)
      .filter(([_, c]) => c.open)
      .map(([id]) => id);
    const persisted: PersistedUIState = {
      version: UI_STATE_VERSION,
      activeAgentId: DEFAULT_AGENT_ID,
      agentActiveChats: { [DEFAULT_AGENT_ID]: activeChatId },
      agentOpenChats: { [DEFAULT_AGENT_ID]: openIds },
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(persisted));
  } catch { /* ignore quota errors */ }
}

export const _persisted = loadUIState();
