import { useAgentStore } from "./index";

/**
 * Subscribe only to the active agent's chat data.
 * Other agents' changes won't trigger re-render because the selector
 * returns the same object reference when activeAgentId hasn't changed.
 */
export function useActiveAgentChats() {
  return useAgentStore((s) => s.agentChats[s.activeAgentId]);
}

/**
 * Subscribe only to the active chat's stream for the active agent.
 * Returns undefined if no active chat.
 */
export function useActiveChatStream() {
  return useAgentStore((s) => {
    const ac = s.agentChats[s.activeAgentId];
    if (!ac) return undefined;
    return ac.chatList.find((c) => c.id === ac.activeChatId);
  });
}

/**
 * Get just the chatList for a specific agent.
 */
export function useAgentChatList(agentId: string) {
  return useAgentStore((s) => s.agentChats[agentId]?.chatList);
}
