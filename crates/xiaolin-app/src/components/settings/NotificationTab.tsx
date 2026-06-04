import { useTranslation } from "react-i18next";
import { SectionTitle, SettingRow, Toggle } from "./SettingsShared";

export function NotificationTab({
  notifications,
  setNotifications,
  sounds,
  setSounds,
  autoScroll,
  setAutoScroll,
  loaded,
  persist,
}: {
  notifications: boolean;
  setNotifications: (v: boolean) => void;
  sounds: boolean;
  setSounds: (v: boolean) => void;
  autoScroll: boolean;
  setAutoScroll: (v: boolean) => void;
  loaded: boolean;
  persist: (key: string, value: boolean) => void;
}) {
  const { t } = useTranslation("settings");
  return (
    <div>
      <SectionTitle>{t("behaviorSection")}</SectionTitle>
      <div className="overflow-hidden rounded-[var(--radius-sm)]" style={{ background: "var(--bg-elevated)", border: "0.5px solid var(--separator-opaque)" }}>
        <SettingRow label={t("desktopNotification")} description={t("desktopNotificationDesc")}>
          <Toggle
            enabled={notifications}
            onChange={() => {
              setNotifications(!notifications);
              if (loaded) persist("notifications", !notifications);
            }}
          />
        </SettingRow>
        <SettingRow label={t("sounds")} description={t("soundsDesc")}>
          <Toggle
            enabled={sounds}
            onChange={() => {
              setSounds(!sounds);
              if (loaded) persist("sounds", !sounds);
            }}
          />
        </SettingRow>
        <SettingRow label={t("autoScroll")} description={t("autoScrollDesc")} isLast>
          <Toggle
            enabled={autoScroll}
            onChange={() => {
              setAutoScroll(!autoScroll);
              if (loaded) persist("autoScroll", !autoScroll);
            }}
          />
        </SettingRow>
      </div>
    </div>
  );
}
