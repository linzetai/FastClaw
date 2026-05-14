import { ChevronDown, ChevronUp, Clock } from "lucide-react";

interface QueueIndicatorProps {
  count: number;
  expanded: boolean;
  onToggle: () => void;
}

export function QueueIndicator({ count, expanded, onToggle }: QueueIndicatorProps) {
  if (count === 0) return null;

  return (
    <button
      onClick={onToggle}
      className="flex w-full items-center gap-1.5 rounded-lg px-3 py-1.5 text-[12px] transition-colors duration-100"
      style={{
        background: "var(--tint)",
        color: "#fff",
        opacity: 0.85,
        cursor: "pointer",
      }}
    >
      <Clock size={14} strokeWidth={2} />
      <span className="flex-1 text-left font-medium">
        {count} 条消息待发送
      </span>
      {expanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
    </button>
  );
}
