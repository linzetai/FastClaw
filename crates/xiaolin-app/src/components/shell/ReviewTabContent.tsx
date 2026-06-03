import { useState, type CSSProperties } from "react";
import { ChevronDown, RotateCcw, Plus as PlusIcon } from "lucide-react";

interface FileChange {
  path: string;
  additions: number;
  deletions: number;
  staged: boolean;
}

const MOCK_FILES: FileChange[] = [
  { path: "src/components/shell/AppShell.tsx", additions: 35, deletions: 0, staged: false },
  { path: "src/components/shell/AppHeader.tsx", additions: 120, deletions: 0, staged: false },
  { path: "src/components/shell/AppSidebar.tsx", additions: 280, deletions: 0, staged: false },
  { path: "src/components/layout/AppLayout.tsx", additions: 12, deletions: 45, staged: false },
  { path: "src/index.css", additions: 18, deletions: 0, staged: true },
];

const fileStyle: CSSProperties = {
  display: "flex",
  alignItems: "center",
  padding: "7px 12px",
  fontSize: 12,
  fontFamily: "var(--font-mono)",
  fontWeight: 500,
  color: "var(--fill-primary)",
  cursor: "pointer",
  transition: "background 0.1s",
};

function FileRow({ file, active, onClick }: { file: FileChange; active: boolean; onClick: () => void }) {
  return (
    <div
      style={{
        ...fileStyle,
        background: active ? "var(--bg-hover)" : "transparent",
        borderBottom: "1px solid var(--border-shell-subtle)",
      }}
      onClick={onClick}
      onMouseEnter={(e) => { if (!active) e.currentTarget.style.background = "var(--bg-hover)"; }}
      onMouseLeave={(e) => { if (!active) e.currentTarget.style.background = active ? "var(--bg-hover)" : "transparent"; }}
    >
      <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
        {file.path.split("/").pop()}
      </span>
      <span style={{ marginLeft: "auto", fontSize: 11, display: "flex", gap: 4, fontWeight: 400, flexShrink: 0 }}>
        {file.additions > 0 && <span style={{ color: "var(--green-text)" }}>+{file.additions}</span>}
        {file.deletions > 0 && <span style={{ color: "var(--red-text)" }}>-{file.deletions}</span>}
      </span>
    </div>
  );
}

function SectionHeader({ title, count, defaultExpanded = true }: { title: string; count: number; defaultExpanded?: boolean }) {
  const [expanded, setExpanded] = useState(defaultExpanded);
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 5,
        padding: "7px 12px",
        fontSize: 12,
        color: "var(--fill-secondary)",
        borderBottom: "1px solid var(--border-shell-subtle)",
        cursor: "pointer",
      }}
      onClick={() => setExpanded(!expanded)}
    >
      <ChevronDown
        size={12}
        strokeWidth={2}
        style={{
          transition: "transform 0.15s",
          transform: expanded ? "rotate(0deg)" : "rotate(-90deg)",
        }}
      />
      <span style={{ fontWeight: 500 }}>{title}</span>
      <span
        style={{
          fontSize: 10,
          fontFamily: "var(--font-mono)",
          background: "var(--bg-hover)",
          padding: "1px 5px",
          borderRadius: 4,
          color: "var(--fill-tertiary)",
        }}
      >
        {count}
      </span>
    </div>
  );
}

export function ReviewTabContent() {
  const [activeFile, setActiveFile] = useState<string | null>(null);
  const staged = MOCK_FILES.filter((f) => f.staged);
  const unstaged = MOCK_FILES.filter((f) => !f.staged);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      {staged.length > 0 && (
        <>
          <SectionHeader title="Staged" count={staged.length} />
          {staged.map((f) => (
            <FileRow key={f.path} file={f} active={activeFile === f.path} onClick={() => setActiveFile(f.path)} />
          ))}
        </>
      )}
      {unstaged.length > 0 && (
        <>
          <SectionHeader title="Unstaged" count={unstaged.length} />
          {unstaged.map((f) => (
            <FileRow key={f.path} file={f} active={activeFile === f.path} onClick={() => setActiveFile(f.path)} />
          ))}
        </>
      )}
    </div>
  );
}

export function ReviewTabFooter() {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        padding: "8px 12px",
      }}
    >
      <button
        type="button"
        style={{
          display: "flex",
          alignItems: "center",
          gap: 4,
          padding: "5px 10px",
          borderRadius: 6,
          border: "1px solid var(--border-shell)",
          background: "transparent",
          color: "var(--fill-tertiary)",
          fontSize: 12,
          fontWeight: 500,
          cursor: "pointer",
          transition: "background 0.12s",
        }}
        onMouseEnter={(e) => { e.currentTarget.style.background = "var(--bg-hover)"; }}
        onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
      >
        <RotateCcw size={12} strokeWidth={1.75} />
        Revert all
      </button>
      <button
        type="button"
        style={{
          display: "flex",
          alignItems: "center",
          gap: 4,
          padding: "5px 10px",
          borderRadius: 6,
          border: "1px solid var(--green-text)",
          background: "transparent",
          color: "var(--green-text)",
          fontSize: 12,
          fontWeight: 500,
          cursor: "pointer",
          transition: "background 0.12s",
        }}
        onMouseEnter={(e) => { e.currentTarget.style.background = "var(--green-line)"; }}
        onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
      >
        <PlusIcon size={12} strokeWidth={1.75} />
        Stage all
      </button>
    </div>
  );
}
