## ADDED Requirements

### Requirement: Plan banner clickable to toggle panel
The Plan mode banner in `StreamFooter` SHALL be clickable. Clicking it SHALL toggle a side panel (`PlanPanel`) that displays the plan file content.

#### Scenario: Click banner opens panel
- **WHEN** User clicks the Plan mode banner in StreamFooter
- **THEN** `PlanPanel` SHALL appear as a right-side panel (width ~360px) adjacent to the chat stream

#### Scenario: Click again closes panel
- **WHEN** User clicks the banner while `PlanPanel` is open
- **THEN** `PlanPanel` SHALL close

#### Scenario: Banner available in Agent mode with plan file
- **WHEN** Session is in Agent mode but `planFileExists` is true
- **THEN** StreamFooter SHALL show a subtle plan file indicator that is clickable to open `PlanPanel`

### Requirement: PlanPanel displays Markdown preview
`PlanPanel` SHALL fetch the plan file content via `transport.getPlanFile(sessionId)` and render it as Markdown.

#### Scenario: Plan file exists and renders
- **WHEN** `PlanPanel` opens and `getPlanFile` returns content
- **THEN** Panel SHALL render the content as formatted Markdown with `react-markdown` + `remark-gfm`

#### Scenario: Plan file does not exist
- **WHEN** `PlanPanel` opens and no plan file exists
- **THEN** Panel SHALL display "计划文件尚未创建" placeholder text

#### Scenario: Plan file updates reflected
- **WHEN** A `plan_file_update` event is received with `exists: true` while `PlanPanel` is open
- **THEN** Panel SHALL refetch and re-render the plan content

### Requirement: PlanPanel header with metadata
`PlanPanel` SHALL display a header with the plan file path (shortened) and a close button.

#### Scenario: Header shows path
- **WHEN** `PlanPanel` is open
- **THEN** Header SHALL show the plan file path with home directory shortened to `~/`

#### Scenario: Close button works
- **WHEN** User clicks the close button in `PlanPanel` header
- **THEN** Panel SHALL close
