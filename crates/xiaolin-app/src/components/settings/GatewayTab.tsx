import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useGatewayStore } from "../../lib/store";
import * as api from "../../lib/api";
import { SectionTitle } from "./SettingsShared";


export function GatewayTab() {
  const { t } = useTranslation("settings");
  const gwInfo = useGatewayStore((s) => s.info);
  const gwMode = useGatewayStore((s) => s.mode);
  const connected = useGatewayStore((s) => s.connected);

  const [gwConfig, setGwConfig] = useState<{ port?: number; host?: string } | null>(null);

  useEffect(() => {
    api.getConfig("gateway").then((data) => {
      const cfg = data as { key?: string; value?: { port?: number; host?: string } } | null;
      setGwConfig((cfg?.value ?? cfg) as { port?: number; host?: string } | null);
    }).catch(() => {});
  }, []);

  const modeLabel = gwMode === "ready" ? t("gatewayMode_ready") : gwMode === "browser" ? t("gatewayMode_browser") : t("gatewayMode_connecting");

  return (
    <div className="space-y-6">
      <div className="space-y-2">
        <SectionTitle>{t("gatewayStatus")}</SectionTitle>
        {(() => {
          const rows = [
            { label: t("gatewayMode"), value: modeLabel },
            { label: t("gatewayState"), value: connected ? t("connected") : t("disconnected"), dot: connected },
            ...(gwInfo ? [
              { label: t("port"), value: String(gwInfo.port) },
              { label: t("versionLabel"), value: gwInfo.version },
              { label: "WebSocket", value: gwInfo.wsUrl },
              { label: "HTTP", value: gwInfo.httpUrl },
            ] : []),
          ];
          return (
            <div className="overflow-hidden rounded-[var(--radius-sm)]" style={{ background: "var(--bg-elevated)", border: "0.5px solid var(--separator-opaque)" }}>
              {rows.map(({ label, value, dot }, idx) => (
                <div key={label} className="flex items-center justify-between gap-3 px-4 py-2.5" style={idx < rows.length - 1 ? { borderBottom: "0.5px solid var(--separator)" } : undefined}>
                  <span className="shrink-0 text-[13px]" style={{ color: "var(--fill-secondary)" }}>{label}</span>
                  <div className="flex min-w-0 items-center gap-1.5">
                    {dot !== undefined && (
                      <span className="inline-block h-[6px] w-[6px] shrink-0 rounded-full" style={{ background: dot ? "var(--green)" : "var(--red)" }} />
                    )}
                    <span className="min-w-0 truncate text-[13px] font-medium font-mono" style={{ color: "var(--fill-primary)" }} title={value}>{value}</span>
                  </div>
                </div>
              ))}
            </div>
          );
        })()}
      </div>
      {gwConfig && (
        <div>
          <SectionTitle>{t("gatewayConfig")}</SectionTitle>
          <div className="overflow-hidden rounded-[var(--radius-sm)]" style={{ background: "var(--bg-elevated)", border: "0.5px solid var(--separator-opaque)" }}>
            <div className="px-4 py-2.5" style={gwConfig.host ? { borderBottom: "0.5px solid var(--separator)" } : undefined}>
              <span className="text-[11px]" style={{ color: "var(--fill-tertiary)" }}>{t("configPort")}</span>
              <div className="text-[13px] font-mono" style={{ color: "var(--fill-primary)" }}>{gwConfig.port ?? t("defaultValue")}</div>
            </div>
            {gwConfig.host && (
              <div className="px-4 py-2.5">
                <span className="text-[11px]" style={{ color: "var(--fill-tertiary)" }}>{t("bindAddress")}</span>
                <div className="text-[13px] font-mono" style={{ color: "var(--fill-primary)" }}>{gwConfig.host}</div>
              </div>
            )}
          </div>
          <p className="mt-2 text-[11px]" style={{ color: "var(--fill-quaternary)" }}>
            {t("gatewayConfigHint")}
          </p>
        </div>
      )}
    </div>
  );
}