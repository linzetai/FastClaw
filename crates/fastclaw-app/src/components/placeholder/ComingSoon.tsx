import { Users, Layout, ListTodo, FolderOpen, Link2, Sparkles } from "lucide-react";
import { ICON } from "../../lib/ui-tokens";

const PAGE_META: Record<string, { icon: typeof Sparkles; desc: string }> = {
  专家: {
    icon: Users,
    desc: "构建你的 AI 专家团队，让每位专家专注于各自领域",
  },
  工作室: {
    icon: Layout,
    desc: "可视化工作流编排，将多个 Agent 串联成自动化流水线",
  },
  任务: {
    icon: ListTodo,
    desc: "追踪和管理 Agent 执行的所有任务与进度",
  },
  文件: {
    icon: FolderOpen,
    desc: "统一管理对话中产生的文档、代码和资源文件",
  },
  连接: {
    icon: Link2,
    desc: "对接外部服务与 API，扩展 Agent 的能力边界",
  },
};

export function ComingSoon({ title }: { title?: string }) {
  const meta = title ? PAGE_META[title] : undefined;
  const Icon = meta?.icon ?? Sparkles;

  return (
    <div
      className="relative flex h-full flex-col items-center justify-center gap-4"
      style={{ background: "var(--bg-primary)", animation: "scale-in var(--duration-slow) var(--ease-out)" }}
    >
      <div
        className="relative flex h-14 w-14 items-center justify-center rounded-[var(--radius-lg)]"
        style={{
          background: "var(--tint-bg)",
          color: "var(--tint)",
          boxShadow: "0 0 0 4px var(--tint-subtle)",
          animation: "icon-float 3s ease-in-out infinite",
        }}
      >
        <Icon size={24} strokeWidth={1.5} />
      </div>
      {title && (
        <h3
          className="relative text-[15px] font-semibold tracking-[-0.01em]"
          style={{ color: "var(--fill-primary)" }}
        >
          {title}
        </h3>
      )}
      <p className="relative max-w-[280px] text-center text-[13px] leading-relaxed" style={{ color: "var(--fill-tertiary)" }}>
        {meta?.desc ?? "功能正在路上了，敬请期待"}
      </p>
      <span
        className="relative mt-2 inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-[11px] font-medium"
        style={{
          background: "var(--tint-subtle)",
          color: "var(--tint)",
          border: "0.5px solid var(--border-subtle)",
        }}
      >
        <Sparkles {...ICON.sm} />
        即将推出
      </span>
    </div>
  );
}
