import i18n from "../../i18n";
import type { Agent } from "./types";

export const idCounter = { nextId: 1 };
export const DEFAULT_AGENT_ID = "main";

export const INITIAL_AGENTS: Agent[] = [
  {
    id: DEFAULT_AGENT_ID, name: "Main Agent", initial: "M", color: "var(--tint)",
    tagline: i18n.t("common:defaultAgentTagline"), online: true, model: "",
  },
];

export function formatTime(d: Date): string {
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  if (diff < 60000) return i18n.t("common:timeJustNow");
  if (diff < 3600000) return i18n.t("common:timeMinutesAgo", { count: Math.floor(diff / 60000) });
  if (diff < 86400000) return i18n.t("common:timeHoursAgo", { count: Math.floor(diff / 3600000) });
  return d.toLocaleDateString(i18n.language === "zh" ? "zh-CN" : undefined, { month: "numeric", day: "numeric" });
}
