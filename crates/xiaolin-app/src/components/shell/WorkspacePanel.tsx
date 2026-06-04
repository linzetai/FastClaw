import type { CSSProperties } from "react";
import { useTranslation } from "react-i18next";
import { Plus, Maximize2, PanelRightClose, FileText } from "lucide-react";
import { useWorkspaceTabs } from "./workspace-tabs";

const tabBtnStyle: CSSProperties = {
  padding: "4px 8px",
  fontSize: 12,
  fontWeight: 500,
  borderRadius: 5,
  cursor: "pointer",
  border: "none",
  background: "transparent",
  transition: "all 0.1s",
};

const iconBtnStyle: CSSProperties = {
  width: 24,
  height: 24,
  borderRadius: 5,
  border: "none",
  background: "transparent",
  cursor: "pointer",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  color: "var(--fill-quaternary)",
  transition: "background 0.12s",
};

export function WorkspacePanel() {
  const { t } = useTranslation("common");
  const tabs = useWorkspaceTabs((s) => s.tabs);
  const activeTabId = useWorkspaceTabs((s) => s.activeTabId);
  const setActiveTab = useWorkspaceTabs((s) => s.setActiveTab);
  const panelOpen = useWorkspaceTabs((s) => s.panelOpen);
  const togglePanel = useWorkspaceTabs((s) => s.togglePanel);

  if (!panelOpen || tabs.length === 0) return null;

  const activeTab = tabs.find((t) => t.id === activeTabId) ?? tabs[0];
  const ActiveComponent = activeTab?.component;
  const FooterComponent = activeTab?.footerComponent;

  return (
    <div
      className="workspace-panel"
      style={{
        width: "var(--panel-w)",
        minWidth: "var(--panel-w)",
        flexShrink: 0,
        display: "flex",
        flexDirection: "column",
        borderLeft: "1px solid var(--border-shell-subtle)",
        minHeight: 0,
      }}
    >
      {/* Tab bar */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          padding: "7px 10px 5px",
          gap: 2,
          borderBottom: "1px solid var(--border-shell-subtle)",
        }}
      >
        <button
          type="button"
          style={iconBtnStyle}
          onMouseEnter={(e) => { e.currentTarget.style.background = "var(--bg-hover)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
        >
          <FileText size={14} strokeWidth={1.7} />
        </button>

        {tabs.map((tab) => {
          const Icon = tab.icon;
          const active = tab.id === (activeTabId ?? tabs[0]?.id);
          return (
            <button
              key={tab.id}
              type="button"
              style={{
                ...tabBtnStyle,
                color: active ? "var(--fill-primary)" : "var(--fill-quaternary)",
                background: active ? "var(--bg-hover)" : "transparent",
              }}
              onClick={() => setActiveTab(tab.id)}
              onMouseEnter={(e) => { if (!active) e.currentTarget.style.color = "var(--fill-secondary)"; }}
              onMouseLeave={(e) => { if (!active) e.currentTarget.style.color = "var(--fill-quaternary)"; }}
            >
              <span style={{ display: "inline-flex", alignItems: "center", gap: 4 }}>
                <Icon size={14} strokeWidth={1.7} />
                {tab.label}
                {tab.badge != null && tab.badge !== false && (
                  <span
                    style={{
                      fontSize: 10,
                      background: "var(--tint)",
                      color: "#fff",
                      borderRadius: 8,
                      padding: "0 5px",
                      minWidth: 16,
                      textAlign: "center",
                      lineHeight: "16px",
                    }}
                  >
                    {tab.badge === true ? "" : tab.badge}
                  </span>
                )}
              </span>
            </button>
          );
        })}

        <button
          type="button"
          style={{ ...iconBtnStyle, fontSize: 13, width: "auto", padding: "4px 7px" }}
          onMouseEnter={(e) => { e.currentTarget.style.background = "var(--bg-hover)"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
        >
          <Plus size={14} strokeWidth={1.7} />
        </button>

        <div style={{ flex: 1 }} />

        <div style={{ display: "flex", gap: 1 }}>
          <button
            type="button"
            style={iconBtnStyle}
            title={t("maximizePanel")}
            onMouseEnter={(e) => { e.currentTarget.style.background = "var(--bg-hover)"; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
          >
            <Maximize2 size={13} strokeWidth={1.7} />
          </button>
          <button
            type="button"
            style={iconBtnStyle}
            title={t("closePanel")}
            onClick={togglePanel}
            onMouseEnter={(e) => { e.currentTarget.style.background = "var(--bg-hover)"; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
          >
            <PanelRightClose size={13} strokeWidth={1.7} />
          </button>
        </div>
      </div>

      {/* Body */}
      <div style={{ flex: 1, overflowY: "auto", minHeight: 0 }}>
        {ActiveComponent && <ActiveComponent />}
      </div>

      {/* Footer */}
      {FooterComponent && (
        <div style={{ borderTop: "1px solid var(--border-shell-subtle)" }}>
          <FooterComponent />
        </div>
      )}
    </div>
  );
}
