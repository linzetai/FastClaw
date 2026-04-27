import { CollapsibleList, SectionHeader, Toggle } from "./common";
import type { AgentToolInfo } from "../../lib/api";

export function AgentTools({
  nonMcpTools,
  filteredTools,
  toolQuery,
  onToolQueryChange,
  onToolToggle,
  togglingTool,
}: {
  nonMcpTools: AgentToolInfo[];
  filteredTools: AgentToolInfo[];
  toolQuery: string;
  onToolQueryChange: (q: string) => void;
  onToolToggle: (toolId: string, enabled: boolean) => void;
  togglingTool: string | null;
}) {
  return (
    <>
      <div>
        <SectionHeader count={nonMcpTools.filter((t) => t.enabled).length} total={nonMcpTools.length} searchable query={toolQuery} onQueryChange={onToolQueryChange}>
          工具
        </SectionHeader>
        <CollapsibleList
          items={filteredTools}
          emptyText={toolQuery ? "无匹配工具" : "未获取到工具列表"}
          renderItem={(tool, _i, isLast) => (
            <div
              key={tool.id}
              className="flex items-center justify-between gap-2 px-3 py-2.5 transition-colors duration-100 hover:bg-[var(--bg-hover)]"
              style={{ borderBottom: isLast ? "none" : "0.5px solid var(--separator)", opacity: tool.enabled ? 1 : 0.55 }}
            >
              <div className="min-w-0 flex-1">
                <span className="block truncate text-[13px]" style={{ color: "var(--fill-primary)" }} title={tool.name}>{tool.name}</span>
                {tool.description && <div className="mt-0.5 truncate text-[11px]" style={{ color: "var(--fill-tertiary)" }} title={tool.description}>{tool.description}</div>}
              </div>
              <Toggle checked={tool.enabled} onChange={(v) => onToolToggle(tool.id, v)} disabled={togglingTool === tool.id} />
            </div>
          )}
        />
      </div>
    </>
  );
}
