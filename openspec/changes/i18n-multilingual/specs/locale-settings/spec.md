## ADDED Requirements

### Requirement: Locale store with persistence
The system SHALL provide a `useLocaleStore` Zustand store with `locale` (UI language) and `responseLang` (LLM reply language) fields, persisted to localStorage under key `xiaolin-locale`.

#### Scenario: Store initialization with defaults
- **WHEN** the store initializes with no saved state
- **THEN** `locale` SHALL be `"zh"` and `responseLang` SHALL be `"zh"`

#### Scenario: Store persists across sessions
- **WHEN** the user sets `locale` to `"en"` and closes the app
- **THEN** upon reopening, the store SHALL restore `locale` as `"en"`

### Requirement: UI language setting
The settings page (GeneralTab) SHALL provide a UI language selector with options: 中文 (zh) and English (en). Changing this setting SHALL immediately update the UI language and persist the choice.

#### Scenario: User switches UI language to English
- **WHEN** the user selects "English" in the language setting
- **THEN** the `locale` state SHALL update to `"en"`, i18next SHALL switch to `"en"`, and all UI text SHALL render in English

### Requirement: Response language setting
The settings page (GeneralTab) SHALL provide a response language selector with options:
- 中文 (zh) — LLM replies in Chinese
- English (en) — LLM replies in English
- 跟随界面语言 (follow-ui) — LLM reply language matches the UI locale
- 自动 (auto) — no language directive injected, model decides based on user message

#### Scenario: Response language set to follow-ui
- **WHEN** `responseLang` is `"follow-ui"` and `locale` is `"en"`
- **THEN** the effective response language sent to the backend SHALL be `"en"`

#### Scenario: Response language set to auto
- **WHEN** `responseLang` is `"auto"`
- **THEN** no `response_language` field SHALL be sent to the backend (or sent as `null`)

### Requirement: Settings UI layout
The language settings SHALL appear in GeneralTab as a distinct section titled "语言 / Language" with two rows: one for UI language and one for response language.

#### Scenario: Settings section visibility
- **WHEN** the user opens the General settings tab
- **THEN** a "语言 / Language" section SHALL be visible with both language selectors
