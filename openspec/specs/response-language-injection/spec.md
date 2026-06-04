## ADDED Requirements

### Requirement: Frontend sends response language preference
The frontend SHALL include a `response_language` field in WebSocket messages (or connection parameters) sent to the backend. The field value SHALL be the resolved language code (`"zh"`, `"en"`, or `null` for auto).

#### Scenario: Response language is zh
- **WHEN** `responseLang` is `"zh"`
- **THEN** the WebSocket message SHALL include `response_language: "zh"`

#### Scenario: Response language is follow-ui with en locale
- **WHEN** `responseLang` is `"follow-ui"` and `locale` is `"en"`
- **THEN** the WebSocket message SHALL include `response_language: "en"`

#### Scenario: Response language is auto
- **WHEN** `responseLang` is `"auto"`
- **THEN** the WebSocket message SHALL include `response_language: null` or omit the field

### Requirement: Backend parses response language from WebSocket
The gateway SHALL parse the `response_language` field from incoming WebSocket messages and pass it to the AgentRuntime as the language preference.

#### Scenario: Gateway receives zh language
- **WHEN** a WebSocket message contains `response_language: "zh"`
- **THEN** the gateway SHALL pass `language_preference: Some("zh-CN")` to the AgentRuntime

#### Scenario: Gateway receives null language
- **WHEN** a WebSocket message contains `response_language: null` or the field is absent
- **THEN** the gateway SHALL pass `language_preference: None` to the AgentRuntime

### Requirement: PromptContext receives language preference
The `PromptContext.language_preference` field SHALL be set from the WebSocket-provided value instead of the current hardcoded `None`. This controls both the system prompt section language selection and the language_section() directive injection.

#### Scenario: Language preference controls prompt sections
- **WHEN** `language_preference` is `Some("zh-CN")`
- **THEN** system prompt sections SHALL use `_zh()` variants AND `language_section()` SHALL output `<language_preference>Respond in: zh-CN</language_preference>`

#### Scenario: No language preference uses English defaults
- **WHEN** `language_preference` is `None`
- **THEN** system prompt sections SHALL use `_en()` variants AND `language_section()` SHALL output `None` (no language directive)

### Requirement: Backward compatibility
The `response_language` WebSocket field SHALL be optional. Existing clients that do not send this field SHALL continue to work with the current default behavior (English prompts, no language directive).

#### Scenario: Legacy client without response_language
- **WHEN** a WebSocket message does not contain a `response_language` field
- **THEN** the backend SHALL behave as if `language_preference: None` (English prompts, no reply language directive)
