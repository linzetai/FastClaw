## ADDED Requirements

### Requirement: i18next initialization
The system SHALL initialize i18next with react-i18next on app startup, loading translations from bundled JSON resources for zh and en locales with namespace-based splitting.

#### Scenario: App starts with persisted locale
- **WHEN** the app starts and localStorage contains a saved locale of "en"
- **THEN** i18next SHALL initialize with `lng: "en"` and all UI text SHALL render in English

#### Scenario: App starts with no saved locale
- **WHEN** the app starts and no locale is saved in localStorage
- **THEN** i18next SHALL initialize with `lng: "zh"` (fallback) and all UI text SHALL render in Chinese

### Requirement: Translation namespace organization
The system SHALL organize translations into namespaces: common, chat, settings, sidebar, header, onboarding, notification. Each namespace SHALL have corresponding JSON files under `src/i18n/locales/{zh,en}/`.

#### Scenario: Namespace loading
- **WHEN** a component uses `useTranslation("settings")`
- **THEN** translations from the settings namespace SHALL be available via the `t()` function

### Requirement: All user-visible text uses i18n keys
All hardcoded Chinese text in TSX/TS component files SHALL be replaced with `t()` function calls referencing translation keys. No user-visible text SHALL remain as hardcoded string literals in component render output.

#### Scenario: Button label translation
- **WHEN** the UI renders a button with label "发送"
- **THEN** the label SHALL come from `t("common.send")` and display "发送" in zh or "Send" in en

#### Scenario: Placeholder translation
- **WHEN** the UI renders an input with placeholder "搜索会话..."
- **THEN** the placeholder SHALL come from `t("sidebar.searchSessions")` and display the locale-appropriate text

### Requirement: Dynamic language switching
The system SHALL support runtime language switching without requiring a page reload. When the locale changes, all mounted components SHALL re-render with the new translations.

#### Scenario: Switch from Chinese to English
- **WHEN** the user changes the locale from "zh" to "en" in settings
- **THEN** all visible UI text SHALL immediately update to English without a page reload

### Requirement: Translation completeness
Both zh and en translation files SHALL contain identical key sets. Every key present in zh SHALL also be present in en, and vice versa.

#### Scenario: Missing translation fallback
- **WHEN** a translation key exists in zh but is missing in en
- **THEN** the system SHALL display the zh fallback text (fallbackLng: "zh")
