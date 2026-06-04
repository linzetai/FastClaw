import { type ReactNode, useState, useMemo, useCallback, useRef } from "react";
import {
  FileText, Sparkles, Search, Code2,
  Lightbulb, PenTool, Zap, BookOpen, RefreshCw,
} from "lucide-react";
import { ICON } from "../../lib/ui-tokens";

interface Suggestion {
  text: string;
  icon: typeof FileText;
}

const SUGGESTION_POOL: Suggestion[] = [
  { text: "分析代码库结构，找出关键模块", icon: Search },
  { text: "设计一个 RESTful API 方案", icon: Sparkles },
  { text: "排查并修复当前存在的 Bug", icon: Zap },
  { text: "为核心函数编写单元测试", icon: Code2 },
  { text: "重构这段代码以提升可读性", icon: PenTool },
  { text: "生成技术文档或 README", icon: BookOpen },
  { text: "解释这个框架的最佳实践", icon: Lightbulb },
  { text: "审查最近的代码变更", icon: FileText },
];

function shuffle<T>(arr: T[]): T[] {
  const copy = [...arr];
  for (let i = copy.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [copy[i], copy[j]] = [copy[j], copy[i]];
  }
  return copy;
}

function extractProjectName(workDir: string | null): string | null {
  if (!workDir) return null;
  const segments = workDir.replace(/\/+$/, "").split("/");
  return segments[segments.length - 1] || null;
}

interface StreamEmptyStateProps {
  workDir: string | null;
  composerSlot: ReactNode;
  onPick: (text: string) => void;
}

export function StreamEmptyState({ workDir, composerSlot, onPick }: StreamEmptyStateProps) {
  const projectName = extractProjectName(workDir);

  const [seed, setSeed] = useState(0);
  const suggestions = useMemo(() => shuffle(SUGGESTION_POOL).slice(0, 3), [seed]);
  const refreshRef = useRef<SVGSVGElement>(null);

  const handleRefresh = useCallback(() => {
    setSeed((s) => s + 1);
    if (refreshRef.current) {
      refreshRef.current.style.transition = "transform 0.5s var(--ease-out)";
      refreshRef.current.style.transform = "rotate(360deg)";
      setTimeout(() => {
        if (refreshRef.current) {
          refreshRef.current.style.transition = "none";
          refreshRef.current.style.transform = "rotate(0deg)";
        }
      }, 500);
    }
  }, []);

  return (
    <div
      className="flex min-h-full flex-col items-center justify-center px-6"
      style={{ animation: "scale-in var(--duration-slow) var(--ease-out)" }}
    >
      <div className="w-full" style={{ maxWidth: 640 }}>
        {/* Title */}
        <h1
          className="mb-6 text-center text-[28px] font-semibold tracking-[-0.03em]"
          style={{
            color: "var(--fill-primary)",
            animation: "fade-slide-up var(--duration-slow) var(--ease-out) 0.05s backwards",
          }}
        >
          {projectName
            ? <>在 <span style={{ color: "var(--tint)" }}>{projectName}</span> 中构建什么？</>
            : "想要构建什么？"
          }
        </h1>

        {/* Composer (passed from parent) */}
        <div
          style={{
            animation: "fade-slide-up var(--duration-slow) var(--ease-out) 0.1s backwards",
          }}
        >
          {composerSlot}
        </div>

        {/* Suggestion rows */}
        <div className="mt-5 space-y-1">
          {suggestions.map((s, i) => {
            const Icon = s.icon;
            return (
              <button
                key={`${s.text}-${seed}`}
                onClick={() => onPick(s.text)}
                className="flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left text-[13px] transition-colors duration-100 hover:bg-[var(--bg-hover)]"
                style={{
                  color: "var(--fill-tertiary)",
                  animation: `fade-slide-up var(--duration-slow) var(--ease-out) ${0.15 + i * 0.06}s backwards`,
                }}
              >
                <Icon {...ICON.sm} className="shrink-0" style={{ opacity: 0.6 }} />
                <span>{s.text}</span>
              </button>
            );
          })}
          <div className="flex justify-end pt-1 pr-1">
            <button
              onClick={handleRefresh}
              className="flex items-center gap-1 rounded-md px-2 py-1 text-[11px] font-medium transition-colors duration-100 hover:bg-[var(--bg-hover)]"
              style={{ color: "var(--fill-quaternary)" }}
            >
              <RefreshCw ref={refreshRef} size={11} strokeWidth={1.8} />
              换一换
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
