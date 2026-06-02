import type { Agent } from "./types";

export const idCounter = { nextId: 1 };
export const DEFAULT_AGENT_ID = "main";

export const INITIAL_AGENTS: Agent[] = [
  {
    id: DEFAULT_AGENT_ID, name: "Main Agent", initial: "M", color: "var(--tint)",
    tagline: "通用智能助手", online: true, model: "",
  },
];

export function formatTime(d: Date): string {
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  if (diff < 60000) return "刚刚";
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`;
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`;
  return d.toLocaleDateString("zh-CN", { month: "numeric", day: "numeric" });
}
