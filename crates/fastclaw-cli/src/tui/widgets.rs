use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::state::*;

pub(crate) fn draw_popup(f: &mut Frame, popup: &PopupKind, select_state: Option<&super::state::SelectState>) {
    let area = f.area();
    let popup_area = centered_rect(60, 60, area);

    f.render_widget(Clear, popup_area);

    match popup {
        PopupKind::Help => {
            let mut lines = vec![
                Line::from(Span::styled(
                    " FastClaw Help",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::default(),
            ];

            let section_style = Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD);
            let key_style = Style::default().fg(Color::Yellow);
            let desc_style = Style::default().fg(Color::White);

            // Keyboard shortcuts section
            lines.push(Line::from(Span::styled(" Keyboard Shortcuts", section_style)));
            let shortcuts: &[(&str, &str)] = &[
                ("Ctrl+C", "Quit"),
                ("Ctrl+L", "Clear + new session"),
                ("Esc", "Cancel stream / 2x clear input"),
                ("Shift+Tab", "Toggle Plan/Agent mode"),
                ("Enter", "Send message"),
                ("Shift+Enter", "Newline (multi-line)"),
                ("Tab", "Auto-complete commands"),
                ("Ctrl+R", "History search"),
                ("Ctrl+W", "Delete word"),
                ("↑/↓", "History navigation"),
                ("Shift+↑↓ / PgUp/Dn", "Scroll messages"),
            ];
            for (k, v) in shortcuts {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {k:<22}"), key_style),
                    Span::styled(*v, desc_style),
                ]));
            }

            lines.push(Line::default());

            // Slash commands by category
            let groups: &[(&str, &[(&str, &str)])] = &[
                (
                    "Session",
                    &[
                        ("/clear", "Clear history and free up context"),
                        ("/resume", "Resume a previous conversation"),
                        ("/branch", "Branch the current conversation"),
                        ("/rename", "Rename current conversation"),
                        ("/export", "Export conversation to file"),
                    ],
                ),
                (
                    "Model",
                    &[
                        ("/model", "Set or show the AI model"),
                        ("/agent", "Deprecated (single-agent mode)"),
                        ("/agents", "Deprecated (single-agent mode)"),
                    ],
                ),
                (
                    "Context",
                    &[
                        ("/context", "Show context window usage"),
                        ("/compact", "Summarize and compact context"),
                        ("/files", "List files in context"),
                    ],
                ),
                (
                    "Development",
                    &[
                        ("/diff", "View uncommitted changes"),
                        ("/undo", "Undo last edit"),
                        ("/plan", "Toggle Plan/Agent mode"),
                        ("/todo", "Show todo list"),
                        ("/skills", "List available skills"),
                    ],
                ),
                (
                    "System",
                    &[
                        ("/doctor", "Diagnose installation"),
                        ("/mcp", "MCP server status"),
                        ("/config", "Show configuration"),
                        ("/status", "Show connection status"),
                        ("/cost", "Show session cost"),
                        ("/stats", "Show token/time stats"),
                    ],
                ),
            ];

            for (group_name, cmds) in groups {
                lines.push(Line::from(Span::styled(
                    format!(" {group_name}"),
                    section_style,
                )));
                for (cmd, desc) in *cmds {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {cmd:<14}"), key_style),
                        Span::styled(*desc, desc_style),
                    ]));
                }
            }

            lines.push(Line::default());
            lines.push(Line::from(vec![
                Span::styled("  Aliases: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "/new /reset /quit /continue /feedback /settings /rewind",
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                " Press Esc/Enter to close ",
                Style::default().fg(Color::DarkGray),
            )));

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Help ");
            let help_area = centered_rect(70, 80, area);
            f.render_widget(Clear, help_area);
            f.render_widget(
                Paragraph::new(lines)
                    .block(block)
                    .wrap(Wrap { trim: false }),
                help_area,
            );
        }
        PopupKind::AskQuestion {
            question, options, ..
        } => {
            let mut lines = vec![
                Line::from(Span::styled(
                    "Agent Question",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::default(),
                Line::from(Span::raw(format!("  {question}"))),
                Line::default(),
            ];
            for (i, (_, label)) in options.iter().enumerate() {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {}. ", i + 1),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(label.clone()),
                ]));
            }
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                " Press number to answer, Esc to dismiss ",
                Style::default().fg(Color::DarkGray),
            )));

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Question ");
            f.render_widget(
                Paragraph::new(lines)
                    .block(block)
                    .wrap(Wrap { trim: false }),
                popup_area,
            );
        }
        PopupKind::ModelPicker => {
            if let Some(sel) = select_state {
                let mut lines = vec![
                    Line::from(Span::styled(
                        " Select Model",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )),
                    Line::default(),
                ];
                if !sel.filter.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("  Filter: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(sel.filter.clone(), Style::default().fg(Color::Yellow)),
                    ]));
                    lines.push(Line::default());
                }
                for (vi, &idx) in sel.filtered_indices.iter().enumerate() {
                    let item = &sel.items[idx];
                    let is_selected = vi == sel.selected;
                    let marker = if item.is_current { "● " } else { "  " };
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Rgb(50, 50, 80))
                            .add_modifier(Modifier::BOLD)
                    } else if item.is_current {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {marker}{}", item.label), style),
                        Span::styled(
                            format!("  {}", item.description),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));
                }
                if sel.filtered_indices.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "  No matching models",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                lines.push(Line::default());
                lines.push(Line::from(Span::styled(
                    " ↑↓ navigate · Enter select · Type to filter · Esc close ",
                    Style::default().fg(Color::DarkGray),
                )));

                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" Model ");
                let picker_area = centered_rect(60, 70, f.area());
                f.render_widget(Clear, picker_area);
                f.render_widget(
                    Paragraph::new(lines)
                        .block(block)
                        .wrap(Wrap { trim: false }),
                    picker_area,
                );
            }
        }
        PopupKind::Sessions(sessions) => {
            let mut lines = vec![
                Line::from(Span::styled(
                    "Recent Sessions",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::default(),
            ];
            for s in sessions {
                let id = s["id"].as_str().unwrap_or("?");
                let agent = s["agentId"].as_str().unwrap_or("?");
                let msgs = s["messageCount"].as_i64().unwrap_or(0);
                let updated = s["updatedAt"].as_str().unwrap_or("?");
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {}", &id[..id.len().min(12)]),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(format!("  agent={agent} msgs={msgs} {updated}")),
                ]));
            }
            lines.push(Line::default());
            lines.push(Line::from(Span::raw(
                "  Use /resume <id> to restore a session",
            )));
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                " Press Esc/Enter to close ",
                Style::default().fg(Color::DarkGray),
            )));

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Sessions ");
            f.render_widget(
                Paragraph::new(lines)
                    .block(block)
                    .wrap(Wrap { trim: false }),
                popup_area,
            );
        }
    }
}

pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
