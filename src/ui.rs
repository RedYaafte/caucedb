use crate::app::{App, Modal, PromptKind};
use ratatui::{prelude::*, widgets::*};
use unicode_width::UnicodeWidthStr;

// Inherit the user's terminal palette. `Reset` delegates background and
// foreground to the active OS/terminal theme; semantic ANSI colors (cyan,
// yellow, red and dark gray) are resolved by that same palette.
pub const BG: Color = Color::Reset;
pub const PANEL: Color = Color::DarkGray;
pub const FG: Color = Color::Reset;
pub const MUTED: Color = Color::DarkGray;
pub const ACCENT: Color = Color::Cyan;
pub const AMBER: Color = Color::Yellow;
const BORDER: Color = Color::DarkGray;
const ERROR: Color = Color::Red;

fn block(title: impl Into<String>, focused: bool) -> Block<'static> {
    Block::bordered()
        .title(format!(" {} ", title.into()))
        .border_style(Style::default().fg(if focused { ACCENT } else { BORDER }))
        .title_style(Style::default().fg(if focused { ACCENT } else { MUTED }))
        .style(Style::default().bg(BG).fg(FG))
}
pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    frame.render_widget(Block::default().style(Style::default().bg(BG).fg(FG)), area);
    if area.width < 45 || area.height < 14 {
        frame.render_widget(
            Paragraph::new(
                "CauceDB\nResize terminal to at least 45 × 14.\nCtrl+Q exits; Esc closes a dialog.",
            )
            .wrap(Wrap { trim: false }),
            area,
        );
        return;
    }
    let layout = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(5),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .split(area);
    let connection = app
        .connected
        .as_ref()
        .map(|p| p.name.as_str())
        .unwrap_or("Disconnected");
    let state = if app.busy {
        format!(
            "{} · {:.1}s",
            app.working_label.unwrap_or("Working"),
            app.started.elapsed().as_secs_f32()
        )
    } else if app.pending {
        "TRANSACTION · CHECK / COMMIT / ROLLBACK".into()
    } else {
        "AUTOCOMMIT OFF".into()
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" CAUCEDB / ", Style::default().fg(ACCENT).bold()),
            Span::raw(connection),
            Span::styled(format!("   {state}"), Style::default().fg(AMBER)),
        ])),
        layout[0],
    );
    if area.width < 90 || area.height < 24 {
        panel(frame, app, app.focus, layout[1]);
    } else {
        let cols = Layout::horizontal([
            Constraint::Length((area.width / 4).clamp(24, 34)),
            Constraint::Min(40),
        ])
        .split(layout[1]);
        let left = Layout::vertical([Constraint::Length(8), Constraint::Min(5)]).split(cols[0]);
        let right = Layout::vertical([Constraint::Percentage(52), Constraint::Percentage(48)])
            .split(cols[1]);
        panel(frame, app, 0, left[0]);
        panel(frame, app, 1, left[1]);
        panel(frame, app, 2, right[0]);
        panel(frame, app, 3, right[1]);
    }
    let status = format!(" {}{}", if app.error { "ERROR · " } else { "" }, app.status);
    frame.render_widget(
        Paragraph::new(status)
            .style(Style::default().fg(if app.error { ERROR } else { FG }))
            .wrap(Wrap { trim: false }),
        layout[2],
    );
    let keys = if let Some(modal) = &app.modal {
        match modal {
            Modal::Form(_) => " PgUp/PgDn Section · Tab Field · Ctrl+U Clear · F8 Request cancel",
            Modal::Help | Modal::Cell(_) => " ↑↓ / PgUp/PgDn Scroll · Esc Close",
            Modal::Prompt(_, _) => " Enter Apply · Esc Cancel · Ctrl+U Clear",
            Modal::Confirm(_, _) => " y Confirm · n / Esc Cancel",
        }
    } else {
        match app.focus {
            0 => " n New   e Edit   Enter Connect   Tab Panel   F1 Help   Ctrl+Q Quit",
            1 => {
                " / Filter   Enter Columns   2 Keys   3 Indexes   4 Source   p Preview   F3 Schema"
            }
            2 => " F5 Statement/selection   F6 Script   Ctrl+S Save   Ctrl+O Open   Tab Results",
            _ => " ↑↓ Rows   ←→ Columns   Enter Cell   [ ] Results   F7 Commit   F9 Rollback",
        }
    };
    frame.render_widget(
        Paragraph::new(keys).style(Style::default().bg(PANEL).fg(ACCENT)),
        layout[3],
    );
    if app.modal.is_some() {
        draw_modal(frame, app);
    }
}
fn panel(frame: &mut Frame, app: &mut App, index: usize, area: Rect) {
    match index {
        0 => {
            let b = block("1  Connections", app.focus == 0);
            if app.config.connections.is_empty() {
                frame.render_widget(
                    Paragraph::new("\n  No saved connections\n\n  n  New Oracle connection")
                        .style(Style::default().fg(MUTED))
                        .block(b),
                    area,
                );
            } else {
                let items: Vec<ListItem> = app
                    .config
                    .connections
                    .iter()
                    .map(|p| {
                        let active = app.connected.as_ref().is_some_and(|c| c.id == p.id);
                        ListItem::new(format!("{} {}", if active { "●" } else { " " }, p.name))
                    })
                    .collect();
                let mut state = ListState::default().with_selected(Some(app.profiles_index));
                frame.render_stateful_widget(
                    List::new(items)
                        .block(b)
                        .highlight_style(Style::default().bg(PANEL).fg(ACCENT))
                        .highlight_symbol("› "),
                    area,
                    &mut state,
                );
            }
        }
        1 => {
            let title = if app.schema.is_empty() {
                "2  Explorer".into()
            } else {
                format!(
                    "2  {}{}",
                    app.schema,
                    if app.filter.is_empty() {
                        String::new()
                    } else {
                        format!(" /{}", app.filter)
                    }
                )
            };
            let b = block(title, app.focus == 1);
            let objects = app.filtered_objects();
            if objects.is_empty() {
                frame.render_widget(
                    Paragraph::new(if app.connected.is_none() {
                        "\n  Connect to inspect schemas,\n  tables, views and PL/SQL."
                    } else {
                        "\n  No matching objects.\n  F3 Switch schema · r Refresh"
                    })
                    .style(Style::default().fg(MUTED))
                    .block(b),
                    area,
                );
            } else {
                let items: Vec<ListItem> = objects
                    .iter()
                    .map(|o| {
                        ListItem::new(vec![
                            Line::from(o.name.clone()),
                            Line::styled(
                                format!("  {}", o.kind.to_lowercase()),
                                Style::default().fg(MUTED),
                            ),
                        ])
                    })
                    .collect();
                let mut state = ListState::default().with_selected(Some(app.object_index));
                frame.render_stateful_widget(
                    List::new(items)
                        .block(b)
                        .highlight_style(Style::default().bg(PANEL).fg(ACCENT))
                        .highlight_symbol("› "),
                    area,
                    &mut state,
                );
            }
        }
        2 => {
            let parts = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(area);
            let tabs: Vec<Line> = app
                .docs
                .iter()
                .enumerate()
                .map(|(i, d)| Line::from(format!("{} {}", i + 1, d.title())))
                .collect();
            frame.render_widget(
                Tabs::new(tabs)
                    .select(app.doc)
                    .style(Style::default().fg(MUTED))
                    .highlight_style(Style::default().fg(ACCENT).bg(PANEL))
                    .divider("│"),
                parts[0],
            );
            let focused = app.focus == 2 && app.modal.is_none();
            let doc = &mut app.docs[app.doc];
            doc.editor
                .set_block(block("3  SQL editor  ·  Ctrl+←/→ tabs", focused));
            doc.editor.set_style(Style::default().bg(BG).fg(FG));
            doc.editor.set_cursor_line_style(Style::default().bg(PANEL));
            doc.editor.set_cursor_style(if focused {
                Style::default().fg(BG).bg(ACCENT)
            } else {
                Style::default()
            });
            frame.render_widget(&doc.editor, parts[1]);
            // Color visible SQL tokens after the editor has applied viewport,
            // selection and cursor backgrounds. Syntax never changes geometry.
            highlight(
                frame.buffer_mut(),
                parts[1].inner(Margin {
                    horizontal: 1,
                    vertical: 1,
                }),
            );
        }
        _ => {
            let title = format!(
                "4  Results  {}/{}",
                if app.results.is_empty() {
                    0
                } else {
                    app.result + 1
                },
                app.results.len()
            );
            let b = block(title, app.focus == 3);
            let Some(result) = app.results.get(app.result) else {
                frame.render_widget(Paragraph::new("\n  Query results appear here.\n\n  F5 Execute statement or selection\n  F6 Execute the entire SQL file").style(Style::default().fg(MUTED)).block(b),area);
                return;
            };
            if result.columns.is_empty() {
                frame.render_widget(
                    Paragraph::new(format!("\n  {}", result.message)).block(b),
                    area,
                );
                return;
            }
            let count = ((area.width.saturating_sub(4)) / 20).max(1) as usize;
            let start = app.column;
            let end = (start + count).min(result.columns.len());
            let widths: Vec<Constraint> = (start..end).map(|_| Constraint::Min(16)).collect();
            let rows = result.rows.iter().map(|r| {
                Row::new(
                    r[start..end]
                        .iter()
                        .map(|c| Cell::from(c.replace(['\n', '\r', '\t'], " "))),
                )
            });
            let table = Table::new(rows, widths)
                .header(
                    Row::new(
                        result.columns[start..end]
                            .iter()
                            .map(|s| Cell::from(s.clone())),
                    )
                    .style(Style::default().fg(ACCENT).bg(PANEL))
                    .height(1),
                )
                .block(b.title_bottom(format!(
                    " row {}/{} · column {}/{}{} ",
                    app.row + usize::from(!result.rows.is_empty()),
                    result.rows.len(),
                    app.column + 1,
                    result.columns.len(),
                    if result.truncated {
                        " · LIMIT REACHED"
                    } else {
                        ""
                    }
                )))
                .row_highlight_style(Style::default().bg(PANEL))
                .column_spacing(2);
            let mut state = TableState::default().with_selected(Some(app.row));
            frame.render_stateful_widget(table, area, &mut state);
        }
    }
}
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let w = width.min(area.width.saturating_sub(2));
    let h = height.min(area.height.saturating_sub(2));
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}
fn draw_modal(frame: &mut Frame, app: &App) {
    let mut available = frame.area();
    available.height = available.height.saturating_sub(3);
    let area = centered(
        available,
        90,
        match app.modal {
            Some(Modal::Form(_)) => 28,
            Some(Modal::Help) => 28,
            Some(Modal::Cell(_)) => 26,
            _ => 16,
        },
    );
    frame.render_widget(Clear, area);
    let Some(modal) = &app.modal else {
        return;
    };
    match modal {
        Modal::Form(form) => {
            let b = block("Oracle connection", true);
            let inner = b.inner(area);
            frame.render_widget(b, area);
            let compact = inner.height < 22 || inner.width < 80;
            let parts = Layout::vertical([
                Constraint::Length(if compact { 1 } else { 2 }),
                Constraint::Min(2),
                Constraint::Length(if compact { 2 } else { 3 }),
                Constraint::Length(2),
            ])
            .split(inner);
            frame.render_widget(
                Tabs::new(["Details", "Advanced", "User info", "Proxy User"])
                    .select(form.section)
                    .highlight_style(Style::default().fg(ACCENT).bg(PANEL))
                    .style(Style::default().fg(MUTED)),
                parts[0],
            );
            let visible = form.visible();
            let capacity = (parts[1].height / 2).max(1) as usize;
            let selected = visible
                .iter()
                .position(|i| *i == form.selected)
                .unwrap_or(0);
            let offset = selected.saturating_sub(capacity.saturating_sub(1));
            for (row, index) in visible.iter().skip(offset).take(capacity).enumerate() {
                let field = &form.fields[*index];
                let focused = *index == form.selected;
                let rect = Rect::new(
                    parts[1].x + 1,
                    parts[1].y + row as u16 * 2,
                    parts[1].width.saturating_sub(2),
                    2,
                );
                let width = (rect.width / 2).min(29);
                let cols =
                    Layout::horizontal([Constraint::Length(width), Constraint::Min(1)]).split(rect);
                frame.render_widget(
                    Paragraph::new(field.label).style(Style::default().fg(if focused {
                        ACCENT
                    } else {
                        MUTED
                    })),
                    cols[0],
                );
                let value = if field.secret {
                    "•".repeat(field.value.chars().count())
                } else if field.options == ["false", "true"] {
                    if field.value == "true" {
                        "[x]".into()
                    } else {
                        "[ ]".into()
                    }
                } else {
                    field.value.clone()
                };
                let suffix = if !field.options.is_empty() {
                    " ‹ ›"
                } else if focused {
                    "▏"
                } else {
                    ""
                };
                let shown = format!("{value}{suffix}");
                let shown = tail(&shown, cols[1].width.saturating_sub(1) as usize);
                frame.render_widget(
                    Paragraph::new(shown)
                        .style(Style::default().bg(if focused { PANEL } else { BG }).fg(FG)),
                    cols[1],
                );
            }
            let hint = if compact {
                "Tab Fields · PgUp/PgDn Sections · Ctrl+U Clear"
            } else if form.section == 1 {
                "TNS uses Oracle Net (TNS_ADMIN). Custom descriptors own their timeouts.\nWallet applies to TCPS Details connections; TNS uses sqlnet.ora."
            } else if form.section == 3 {
                "Username in User info is the target user. Proxy credentials authenticate.\nSave password in User info also controls the proxy password."
            } else {
                "Tab/↑↓ Field · PgUp/PgDn Section · ←→ Option · Ctrl+U Clear\nPassword is masked and saved only in the system keyring."
            };
            let feedback = if app.busy
                && matches!(
                    app.working_label,
                    Some("Testing Oracle connection" | "Connecting to Oracle")
                ) {
                Some((
                    format!(
                        "{}… {:.1}s · please wait\nF8 requests cancellation; setup may wait for its timeout.",
                        app.working_label.unwrap_or("Working"),
                        app.started.elapsed().as_secs_f32()
                    ),
                    AMBER,
                ))
            } else if form.action_attempted {
                Some((
                    format!("{}{}", if app.error { "ERROR · " } else { "" }, app.status),
                    if app.error { ERROR } else { ACCENT },
                ))
            } else {
                None
            };
            frame.render_widget(
                Paragraph::new(feedback.as_ref().map_or(hint, |(text, _)| text.as_str()))
                    .style(
                        Style::default().fg(feedback.as_ref().map_or(MUTED, |(_, color)| *color)),
                    )
                    .wrap(Wrap { trim: false }),
                parts[2],
            );
            frame.render_widget(
                Paragraph::new(" F5 Test (Ping)     F6 Connect     F2 Save     Esc Cancel")
                    .style(Style::default().fg(ACCENT).bg(PANEL))
                    .wrap(Wrap { trim: false }),
                parts[3],
            );
        }
        Modal::Prompt(kind, value) => {
            let (title, hint) = match kind {
                PromptKind::Open => ("Open SQL file", "UTF-8 file path"),
                PromptKind::Save => (
                    "Save SQL file",
                    "File path (.sql is added when no extension is given)",
                ),
                PromptKind::Search => ("Search SQL", "Regular expression; matches wrap around"),
                PromptKind::Schema => ("Switch schema", "Exact name from available schemas"),
                PromptKind::Filter => (
                    "Filter objects",
                    "Object name or type; empty clears the filter",
                ),
            };
            let extra = if matches!(kind, PromptKind::Schema) {
                app.schemas.join("  ")
            } else {
                String::new()
            };
            frame.render_widget(
                Paragraph::new(format!(
                    "\n{hint}\n\n{}▏\n\n{extra}\n\nEnter Apply · Esc Cancel · Ctrl+U Clear",
                    tail(value, area.width.saturating_sub(4) as usize)
                ))
                .block(block(title, true))
                .wrap(Wrap { trim: false }),
                area,
            );
        }
        Modal::Confirm(_, text) => frame.render_widget(
            Paragraph::new(format!("\n{text}\n\n y Confirm     n / Esc Cancel"))
                .block(block("Confirm", true))
                .wrap(Wrap { trim: false }),
            area,
        ),
        Modal::Help => frame.render_widget(
            Paragraph::new(HELP)
                .scroll((app.detail_scroll, 0))
                .block(block("Keyboard reference", true))
                .wrap(Wrap { trim: false }),
            area,
        ),
        Modal::Cell(text) => frame.render_widget(
            Paragraph::new(format!("{text}\n\nEsc Close"))
                .block(block("Detail / messages · ↑↓ Scroll · Esc Close", true))
                .wrap(Wrap { trim: false })
                .scroll((app.detail_scroll, 0)),
            area,
        ),
    }
}
fn tail(s: &str, width: usize) -> String {
    let mut result = String::new();
    let mut used = 0;
    for c in s.chars().rev() {
        let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if used + w > width {
            break;
        }
        result.insert(0, c);
        used += w;
    }
    result
}
fn highlight(buffer: &mut Buffer, area: Rect) {
    const WORDS: &[&str] = &[
        "SELECT",
        "FROM",
        "WHERE",
        "JOIN",
        "LEFT",
        "RIGHT",
        "INNER",
        "OUTER",
        "ON",
        "AS",
        "AND",
        "OR",
        "NOT",
        "NULL",
        "IS",
        "IN",
        "ORDER",
        "BY",
        "GROUP",
        "HAVING",
        "INSERT",
        "INTO",
        "VALUES",
        "UPDATE",
        "SET",
        "DELETE",
        "CREATE",
        "TABLE",
        "VIEW",
        "DROP",
        "ALTER",
        "BEGIN",
        "END",
        "DECLARE",
        "COMMIT",
        "ROLLBACK",
        "WITH",
        "UNION",
        "ALL",
        "CASE",
        "WHEN",
        "THEN",
        "ELSE",
        "DISTINCT",
        "FETCH",
        "FIRST",
        "ROWS",
        "ONLY",
        "PROCEDURE",
        "FUNCTION",
        "RETURN",
        "NUMBER",
        "VARCHAR2",
    ];
    for y in area.y..area.bottom() {
        let text: String = (area.x..area.right())
            .map(|x| buffer[(x, y)].symbol().to_owned())
            .collect();
        let mut offset = 0;
        for word in text.split_inclusive(|c: char| !c.is_alphanumeric() && c != '_') {
            let token = word.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
            if WORDS.contains(&token.to_ascii_uppercase().as_str()) {
                for x in area.x + offset as u16
                    ..(area.x + (offset + token.width()) as u16).min(area.right())
                {
                    if buffer[(x, y)].bg != ACCENT {
                        buffer[(x, y)].set_fg(ACCENT);
                    }
                }
            }
            offset += word.width();
        }
    }
}
const HELP:&str="\n  GLOBAL\n  Tab / Shift+Tab   Switch panel (narrow terminals show one panel)\n  Ctrl+Q            Exit with unsaved-file / transaction protection\n  F2                Edit connection · F3 Switch schema\n  F5 / F6           Execute statement or selection / entire file\n  F7 / F9           Commit / Rollback · F8 Cancel query\n  F10               Messages, errors and execution history\n\n  FILES & EDITOR\n  Ctrl+N / Ctrl+O   New / Open .sql file\n  Ctrl+S / F4       Save / Save As · Ctrl+W Close file\n  Ctrl+Left/Right   Previous / next document tab\n  Shift+arrows      Select text · Ctrl+Z Undo · Ctrl+Y Redo\n  Ctrl+F            Search (regular expression)\n\n  EXPLORER\n  / Filter · r Refresh · Enter/1 Columns · 2 Keys · 3 Indexes\n  4 PL/SQL source · p Open and execute data preview\n\n  RESULTS\n  Arrows Navigate · PgUp/PgDn Scroll · Enter Cell detail\n  [ / ] Previous / next result (last 20 retained)\n\n  CONNECTIONS: n New · e Edit · Enter Connect · d Disconnect\n  Esc closes this reference.";

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn rendered_text(app: &mut App, width: u16, height: u16) -> String {
        let backend = ratatui::backend::TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }
    #[test]
    fn renders_all_sizes_and_forms() {
        for (w, h) in [(120, 40), (80, 24), (45, 14), (30, 10)] {
            let mut app = App::new(crate::config::Config::default(), "unused".into());
            let backend = ratatui::backend::TestBackend::new(w, h);
            let mut terminal = Terminal::new(backend).unwrap();
            for focus in 0..4 {
                app.focus = focus;
                terminal.draw(|f| draw(f, &mut app)).unwrap();
            }
            app.modal = Some(Modal::Form(Box::new(crate::app::ConnectionForm::new(
                crate::config::Profile::default(),
            ))));
            terminal.draw(|f| draw(f, &mut app)).unwrap();
        }
    }

    #[test]
    fn help_modal_renders_its_content() {
        let mut app = App::new(crate::config::Config::default(), "unused".into());
        app.key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::F(1),
            crossterm::event::KeyModifiers::NONE,
        ));
        let backend = ratatui::backend::TestBackend::new(100, 32);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw(f, &mut app)).unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("Keyboard reference"));
        assert!(rendered.contains("GLOBAL"));
        assert!(rendered.contains("F5 / F6"));
    }

    #[test]
    fn connection_form_shows_validation_progress_and_result() {
        let mut app = App::new(crate::config::Config::default(), "unused".into());
        app.modal = Some(Modal::Form(Box::new(crate::app::ConnectionForm::new(
            crate::config::Profile::default(),
        ))));
        for action in [KeyCode::F(5), KeyCode::F(6), KeyCode::F(2)] {
            app.key(KeyEvent::new(action, KeyModifiers::NONE));
            assert!(app.error);
            for (width, height) in [(100, 32), (45, 14)] {
                let rendered = rendered_text(&mut app, width, height);
                assert_eq!(
                    rendered.matches("ERROR · Username is required").count(),
                    2,
                    "the form and status must both show the error at {width}x{height}"
                );
            }
        }

        app.error = false;
        app.busy = true;
        app.working_label = Some("Testing Oracle connection");
        assert!(rendered_text(&mut app, 100, 32).contains("Testing Oracle connection"));
        assert!(rendered_text(&mut app, 100, 32).contains("please wait"));
        assert!(rendered_text(&mut app, 45, 14).contains("Testing Oracle connection"));

        app.working_label = Some("Connecting to Oracle");
        assert!(rendered_text(&mut app, 100, 32).contains("Connecting to Oracle"));

        app.busy = false;
        app.working_label = None;
        app.message("Test passed · authenticated query and ping", false);
        assert!(rendered_text(&mut app, 100, 32).contains("Test passed"));
    }
}
