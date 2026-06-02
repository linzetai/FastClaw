import { saveUIStateFromMeta } from "./persistence";
import { useChatMetaStore as _chatMetaStore } from "./chat-meta-store";

export { useConfigStore } from "./config-store";
export * from "./types";

export { useChatMetaStore } from "./chat-meta-store";
export { useStreamStore, EMPTY_STREAM } from "./stream-store";
export { useQueueStore } from "./queue-store";
export { useUIStore } from "./ui-store";
export type { NavItem } from "./ui-store";
export {
  useActiveChatId,
  useActiveChatMeta,
  useActiveStream,
  useChatStream,
  useChatUsage,
  useActiveSubAgentRuns,
  useChatSubAgentRuns,
  useChatLastSegments,
  useChatQueue,
} from "./selectors";

_chatMetaStore.subscribe((state, prev) => {
  if (state.chats !== prev.chats || state.activeChatId !== prev.activeChatId) {
    saveUIStateFromMeta(state.activeChatId, state.chats);
  }
});
