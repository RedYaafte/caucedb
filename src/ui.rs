use crate::app::{App, Modal, PromptKind};
use ratatui::{prelude::*, widgets::*};
use unicode_width::UnicodeWidthStr;

// Reference / Workspace: a calm canvas, warm legible text and a single gold
// action accent. RGB colors keep contrast stable across terminal palettes.
pub const BG: Color = Color::Rgb(27, 30, 31);
pub const PANEL: Color = Color::Rgb(53, 49, 45);
pub const FG: Color = Color::Rgb(200, 192, 174);
pub const MUTED: Color = Color::Rgb(169, 159, 144);
pub const ACCENT: Color = Color::Rgb(226, 163, 95);
const SURFACE: Color = Color::Rgb(37, 41, 42);
const BRIGHT: Color = Color::Rgb(241, 233, 216);
const BORDER: Color = Color::Rgb(119, 116, 108);
const SOFT_BORDER: Color = Color::Rgb(85, 86, 80);
const ERROR: Color = Color::Rgb(241, 132, 114);

fn block(title: impl Into<String>, focused: bool) -> Block<'static> {
    Block::bordered()
        .title(format!(" {} ", title.into()))
        .border_style(Style::default().fg(if focused { ACCENT } else { SOFT_BORDER }))
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
        Constraint::Length(1),
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
    let compact_header = area.width < 70;
    let connection_label = if compact_header && connection.chars().count() > 14 {
        format!("{}…", connection.chars().take(13).collect::<String>())
    } else {
        connection.to_owned()
    };
    let state_label = if compact_header {
        if app.busy {
            "WORKING"
        } else if app.pending {
            "TX PENDING"
        } else {
            "TX OFF"
        }
        .to_owned()
    } else {
        state
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" ◇ CAUCEDB", Style::default().fg(BRIGHT).bold()),
            Span::styled(
                if compact_header { " / " } else { "   /   " },
                Style::default().fg(MUTED),
            ),
            Span::styled(connection_label, Style::default().fg(FG)),
            Span::styled(format!("  {state_label}"), Style::default().fg(ACCENT)),
        ]))
        .style(Style::default().bg(BG)),
        layout[0],
    );
    if area.width < 90 || area.height < 24 {
        panel(frame, app, app.focus, layout[1]);
    } else {
        let cols = Layout::horizontal([
            Constraint::Length((area.width / 5).clamp(24, 30)),
            Constraint::Min(40),
        ])
        .split(layout[1]);
        draw_sidebar(frame, app, cols[0]);
        draw_workspace(frame, app, cols[1]);
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
    } else if area.width < 60 {
        " Tab/Shift+Tab Panel   F1 Help"
    } else if area.width < 90 || area.height < 24 {
        match app.focus {
            0 => " Tab/Shift+Tab Panel   n New   Enter Connect   F1 Help",
            1 => " Tab/Shift+Tab Panel   / Filter   Enter Inspect   F1 Help",
            2 => " Tab/Shift+Tab Panel   F5 Statement   F6 Script   F1 Help",
            _ => " Tab/Shift+Tab Panel   ↑↓ Rows   ←→ Columns   F1 Help",
        }
    } else {
        match app.focus {
            0 => " n New   e Edit   Enter Connect   F7 Commit   F9 Rollback   F1 Help",
            1 => {
                " / Filter   Enter Columns   p Preview   F3 Schema   F7 Commit   F9 Rollback   F1 Help"
            }
            2 => " F5 Statement   F6 Script   Ctrl+S Save   Ctrl+O Open   F7 Commit   F9 Rollback   F1 Help",
            _ => " ↑↓ Rows   ←→ Columns   Enter Cell   [ ] Results   F7 Commit   F9 Rollback   F1 Help",
        }
    };
    frame.render_widget(
        Paragraph::new(keys).style(Style::default().bg(SURFACE).fg(FG)),
        layout[3],
    );
    if app.modal.is_some() {
        draw_modal(frame, app);
    }
}

fn label(frame: &mut Frame, area: Rect, text: impl Into<String>, style: Style) {
    if area.width > 0 && area.height > 0 {
        frame.render_widget(Paragraph::new(text.into()).style(style), area);
    }
}

fn rule(frame: &mut Frame, x: u16, y: u16, width: u16, color: Color) {
    let buffer = frame.buffer_mut();
    for dx in 0..width {
        buffer[(x + dx, y)].set_symbol("─").set_fg(color);
    }
}

fn sidebar_selection(selected: bool, focused: bool) -> Style {
    Style::default()
        .bg(if selected {
            if focused {
                ACCENT
            } else {
                PANEL
            }
        } else {
            BG
        })
        .fg(if selected {
            if focused {
                BG
            } else {
                BRIGHT
            }
        } else {
            FG
        })
}

fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    frame.render_widget(Block::default().style(Style::default().bg(BG)), area);
    if area.width < 3 || area.height < 12 {
        return;
    }
    let x = area.x + 1;
    let width = area.width.saturating_sub(3);
    label(
        frame,
        Rect::new(x, area.y, width, 1),
        "Connections",
        Style::default()
            .fg(if app.focus == 0 { ACCENT } else { MUTED })
            .bold(),
    );
    let profile_rows = 3usize.min(app.config.connections.len().max(1));
    let profile_start = app
        .profiles_index
        .saturating_sub(profile_rows.saturating_sub(1));
    if app.config.connections.is_empty() {
        label(
            frame,
            Rect::new(x, area.y + 2, width, 1),
            "n  New connection",
            Style::default().fg(if app.focus == 0 { ACCENT } else { FG }),
        );
    } else {
        for (offset, profile) in app
            .config
            .connections
            .iter()
            .enumerate()
            .skip(profile_start)
            .take(profile_rows)
        {
            let y = area.y + 2 + (offset - profile_start) as u16;
            let selected = offset == app.profiles_index;
            let active = app.connected.as_ref().is_some_and(|c| c.id == profile.id);
            let marker = if selected && app.focus == 0 {
                "›"
            } else if active {
                "●"
            } else {
                " "
            };
            let style = sidebar_selection(selected, app.focus == 0);
            label(
                frame,
                Rect::new(x, y, width, 1),
                format!("{marker} {}", profile.name),
                style,
            );
        }
        if app.config.connections.len() == 1 {
            let status = if app.connected.is_some() {
                "connected"
            } else {
                "saved"
            };
            label(
                frame,
                Rect::new(x, area.y + 4, width, 1),
                format!("Oracle · {status}"),
                Style::default().fg(MUTED),
            );
        }
    }
    let divider_y = area.y + 6;
    rule(frame, x, divider_y, width, SOFT_BORDER);
    let schema = if app.schema.is_empty() {
        "Explorer".to_owned()
    } else {
        format!("SCHEMA / {}", app.schema)
    };
    label(
        frame,
        Rect::new(x, divider_y + 2, width, 1),
        schema,
        Style::default()
            .fg(if app.focus == 1 { ACCENT } else { MUTED })
            .bold(),
    );
    let files_space = if area.height >= 28 { 7 } else { 2 };
    let objects_y = divider_y + 4;
    let objects_bottom = area.bottom().saturating_sub(files_space);
    let capacity = objects_bottom.saturating_sub(objects_y) as usize;
    let objects = app.filtered_objects();
    if objects.is_empty() {
        let hint = if app.connected.is_none() {
            "Connect to inspect"
        } else {
            "No matching objects"
        };
        label(
            frame,
            Rect::new(x, objects_y, width, 1),
            hint,
            Style::default().fg(if app.focus == 1 { ACCENT } else { MUTED }),
        );
    } else if capacity > 0 {
        let start = app.object_index.saturating_sub(capacity.saturating_sub(1));
        for (index, object) in objects.iter().enumerate().skip(start).take(capacity) {
            let y = objects_y + (index - start) as u16;
            let selected = index == app.object_index;
            let style = sidebar_selection(selected, app.focus == 1);
            let marker = if selected && app.focus == 1 {
                "›"
            } else {
                " "
            };
            label(
                frame,
                Rect::new(x, y, width, 1),
                format!("{marker} {}", object.name),
                style,
            );
        }
    }
    if area.height >= 28 {
        let y = area.bottom() - 6;
        rule(frame, x, y, width, SOFT_BORDER);
        label(
            frame,
            Rect::new(x, y + 1, width, 1),
            "FILES",
            Style::default()
                .fg(if app.focus == 2 { ACCENT } else { MUTED })
                .bold(),
        );
        let file_start = app.doc.saturating_sub(2);
        for (index, doc) in app.docs.iter().enumerate().skip(file_start).take(3) {
            let selected = index == app.doc;
            let style = sidebar_selection(selected, app.focus == 2);
            label(
                frame,
                Rect::new(x, y + 2 + (index - file_start) as u16, width, 1),
                format!(
                    "{} {}",
                    if selected && app.focus == 2 {
                        "›"
                    } else {
                        " "
                    },
                    doc.title()
                ),
                style,
            );
        }
    }
}

fn draw_workspace(frame: &mut Frame, app: &mut App, area: Rect) {
    let outer = Block::bordered()
        .border_style(Style::default().fg(SOFT_BORDER))
        .style(Style::default().bg(BG));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);
    if inner.width < 3 || inner.height < 9 {
        return;
    }
    let tabs = app
        .docs
        .iter()
        .enumerate()
        .map(|(i, doc)| Line::from(format!("{} {}", i + 1, doc.title())))
        .collect::<Vec<_>>();
    frame.render_widget(
        Tabs::new(tabs)
            .select(app.doc)
            .style(Style::default().fg(MUTED).bg(BG))
            .highlight_style(Style::default().fg(BG).bg(ACCENT).bold())
            .divider(" "),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );
    let remaining = inner.height - 1;
    let editor_height = (remaining * 48 / 100).clamp(4, remaining.saturating_sub(5));
    let editor = Rect::new(inner.x, inner.y + 1, inner.width, editor_height);
    draw_editor(frame, app, editor, false);
    let separator_y = editor.bottom();
    rule(frame, inner.x, separator_y, inner.width, SOFT_BORDER);
    let results = Rect::new(
        inner.x,
        separator_y + 1,
        inner.width,
        inner.bottom().saturating_sub(separator_y + 1),
    );
    draw_results(frame, app, results, false);
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
                        .highlight_style(sidebar_selection(true, app.focus == 0))
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
                        .highlight_style(sidebar_selection(true, app.focus == 1))
                        .highlight_symbol("› "),
                    area,
                    &mut state,
                );
            }
        }
        2 => {
            let parts = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(area);
            let tabs = app
                .docs
                .iter()
                .enumerate()
                .map(|(i, d)| Line::from(format!("{} {}", i + 1, d.title())))
                .collect::<Vec<_>>();
            frame.render_widget(
                Tabs::new(tabs)
                    .select(app.doc)
                    .style(Style::default().fg(MUTED))
                    .highlight_style(Style::default().fg(BG).bg(ACCENT).bold())
                    .divider(" "),
                parts[0],
            );
            draw_editor(frame, app, parts[1], true);
        }
        _ => draw_results(frame, app, area, true),
    }
}

fn draw_editor(frame: &mut Frame, app: &mut App, area: Rect, bordered: bool) {
    let inner = if bordered {
        let b = block("SQL editor · Ctrl+←/→ tabs", app.focus == 2);
        let inner = b.inner(area);
        frame.render_widget(b, area);
        inner
    } else {
        area
    };
    if inner.width < 3 || inner.height < 2 {
        return;
    }
    label(
        frame,
        Rect::new(inner.x + 1, inner.y, inner.width.saturating_sub(2), 1),
        "SQL EDITOR  ·  F5 Statement  F6 Script  Ctrl+S Save",
        Style::default()
            .fg(if app.focus == 2 { ACCENT } else { BRIGHT })
            .bold(),
    );
    let edit_area = Rect::new(
        inner.x + 1,
        inner.y + 1,
        inner.width.saturating_sub(2),
        inner.height - 1,
    );
    let focused = app.focus == 2 && app.modal.is_none();
    let doc = &mut app.docs[app.doc];
    doc.editor.set_block(Block::default());
    doc.editor.set_style(Style::default().bg(BG).fg(FG));
    doc.editor.set_cursor_line_style(Style::default().bg(PANEL));
    doc.editor.set_cursor_style(if focused {
        Style::default().fg(BG).bg(ACCENT)
    } else {
        Style::default()
    });
    frame.render_widget(&doc.editor, edit_area);
    // Syntax color follows the rendered viewport without changing geometry.
    highlight(frame.buffer_mut(), edit_area);
}

fn draw_results(frame: &mut Frame, app: &App, area: Rect, bordered: bool) {
    let inner = if bordered {
        let b = block("Results", app.focus == 3);
        let inner = b.inner(area);
        frame.render_widget(b, area);
        inner
    } else {
        area
    };
    if inner.width < 8 || inner.height < 3 {
        return;
    }
    let title = app
        .results
        .get(app.result)
        .map_or("RESULTS".to_owned(), |r| {
            format!(
                "RESULTS  {}/{}  ·  {} rows  ·  {} columns",
                app.result + 1,
                app.results.len(),
                r.rows.len(),
                r.columns.len()
            )
        });
    label(
        frame,
        Rect::new(inner.x + 1, inner.y, inner.width.saturating_sub(2), 1),
        title,
        Style::default()
            .fg(if app.focus == 3 { ACCENT } else { BRIGHT })
            .bold(),
    );
    let Some(result) = app.results.get(app.result) else {
        label(
            frame,
            Rect::new(inner.x + 1, inner.y + 2, inner.width.saturating_sub(2), 1),
            "Query results appear here · F5 statement / F6 script",
            Style::default().fg(MUTED),
        );
        return;
    };
    if result.columns.is_empty() {
        label(
            frame,
            Rect::new(inner.x + 1, inner.y + 2, inner.width.saturating_sub(2), 1),
            &result.message,
            Style::default().fg(FG),
        );
        return;
    }
    let grid_area = Rect::new(
        inner.x + 1,
        inner.y + 1,
        inner.width.saturating_sub(2),
        inner.height.saturating_sub(2),
    );
    draw_result_grid(frame, grid_area, result, app.row, app.column);
    let position = format!(
        "row {}/{} · column {}/{}{}",
        app.row + usize::from(!result.rows.is_empty()),
        result.rows.len(),
        app.column + 1,
        result.columns.len(),
        if result.truncated {
            " · LIMIT REACHED"
        } else {
            ""
        }
    );
    label(
        frame,
        Rect::new(
            inner.x + 1,
            inner.bottom() - 1,
            inner.width.saturating_sub(2),
            1,
        ),
        position,
        Style::default().fg(MUTED),
    );
}

fn draw_result_grid(
    frame: &mut Frame,
    area: Rect,
    result: &crate::db::QueryResult,
    row: usize,
    column: usize,
) {
    if area.width < 9 || area.height < 3 {
        return;
    }
    let row_number_width = (result.rows.len().max(1).to_string().len() + 2).max(5) as u16;
    let mut widths = vec![row_number_width]; // Includes the right rule.
    let mut indices = Vec::new();
    let used = 1u16;
    for index in column.min(result.columns.len().saturating_sub(1))..result.columns.len() {
        let header = result.columns[index].width();
        let sample = result
            .rows
            .iter()
            .take(32)
            .filter_map(|r| r.get(index))
            .map(|s| s.width())
            .max()
            .unwrap_or(0);
        let wanted = (header.max(sample) + 2).clamp(12, 28) as u16;
        let remaining = area.width.saturating_sub(used + widths.iter().sum::<u16>());
        if remaining < 4 {
            break;
        }
        let width = wanted.min(remaining);
        widths.push(width);
        indices.push(index);
    }
    if indices.last() == Some(&result.columns.len().saturating_sub(1)) {
        let spare = area.width.saturating_sub(1 + widths.iter().sum::<u16>());
        if let Some(last) = widths.last_mut() {
            *last += spare;
        }
    }
    let grid_width = 1 + widths.iter().sum::<u16>();
    let mut boundaries = vec![area.x];
    let mut x = area.x;
    for width in &widths {
        x += *width;
        boundaries.push(x);
    }
    let header_style = Style::default().fg(BRIGHT).bg(SURFACE).bold();
    frame.render_widget(
        Block::default().style(Style::default().bg(SURFACE)),
        Rect::new(area.x, area.y, grid_width, 1),
    );
    label(
        frame,
        Rect::new(area.x + 2, area.y, row_number_width - 2, 1),
        "#",
        header_style,
    );
    for (slot, index) in indices.iter().enumerate() {
        label(
            frame,
            Rect::new(
                boundaries[slot + 1] + 2,
                area.y,
                widths[slot + 1].saturating_sub(2),
                1,
            ),
            &result.columns[*index],
            header_style,
        );
    }
    let max_rows = (area.height.saturating_sub(2) / 2) as usize;
    let start = row.saturating_sub(max_rows.saturating_sub(1));
    let visible = result
        .rows
        .iter()
        .enumerate()
        .skip(start)
        .take(max_rows)
        .collect::<Vec<_>>();
    for (offset, (index, cells)) in visible.iter().enumerate() {
        let y = area.y + 2 + offset as u16 * 2;
        let selected = *index == row;
        let style = Style::default()
            .bg(if selected { PANEL } else { BG })
            .fg(if selected { BRIGHT } else { FG });
        frame.render_widget(
            Block::default().style(style),
            Rect::new(area.x, y, grid_width, 1),
        );
        label(
            frame,
            Rect::new(area.x + 2, y, row_number_width - 2, 1),
            format!("{}", index + 1),
            style,
        );
        for (slot, col) in indices.iter().enumerate() {
            let value = cells
                .get(*col)
                .map_or("", String::as_str)
                .replace(['\n', '\r', '\t'], " ");
            label(
                frame,
                Rect::new(
                    boundaries[slot + 1] + 2,
                    y,
                    widths[slot + 1].saturating_sub(2),
                    1,
                ),
                value,
                style,
            );
        }
    }
    let bottom = area.y + 1 + visible.len() as u16 * 2;
    for y in area.y..=bottom.min(area.bottom() - 1) {
        let is_rule = y > area.y && (y - area.y) % 2 == 1;
        for (i, bx) in boundaries.iter().enumerate() {
            let symbol = if is_rule {
                if i == 0 {
                    "├"
                } else if i == boundaries.len() - 1 {
                    "┤"
                } else {
                    "┼"
                }
            } else {
                "│"
            };
            frame.buffer_mut()[(*bx, y)]
                .set_symbol(symbol)
                .set_fg(BORDER);
        }
        if is_rule {
            for dx in 1..grid_width - 1 {
                let px = area.x + dx;
                if !boundaries.contains(&px) {
                    frame.buffer_mut()[(px, y)].set_symbol("─").set_fg(BORDER);
                }
            }
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
                    .highlight_style(Style::default().fg(BG).bg(ACCENT).bold())
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
                    ACCENT,
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
            let actions = if parts[3].width < 65 {
                "F5 Test  F6 Connect\nF2 Save  Esc Cancel"
            } else {
                " F5 Test (Ping)     F6 Connect     F2 Save     Esc Cancel"
            };
            frame.render_widget(
                Paragraph::new(actions)
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
    fn reference_workspace_uses_approved_gold_accent() {
        assert_eq!(ACCENT, Color::Rgb(226, 163, 95));
    }

    #[test]
    fn reference_workspace_renders_navigation_and_result_grid() {
        let mut app = App::new(crate::config::Config::default(), "unused".into());
        app.schema = "APP_DEV".into();
        app.results = vec![crate::db::QueryResult {
            columns: vec!["ID".into(), "CUSTOMER".into(), "STATUS".into()],
            rows: vec![
                vec!["1".into(), "Ana".into(), "PAID".into()],
                vec!["2".into(), "Diego".into(), "PENDING".into()],
            ],
            ..Default::default()
        }];
        app.focus = 3;
        app.row = 1;
        let mut terminal = Terminal::new(ratatui::backend::TestBackend::new(120, 40)).unwrap();
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        let rendered = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        for expected in [
            "Connections",
            "SCHEMA / APP_DEV",
            "FILES",
            "SQL EDITOR",
            "RESULTS",
            "CUSTOMER",
            "F1 Help",
        ] {
            assert!(rendered.contains(expected), "missing {expected}");
        }
        assert!(
            rendered.contains('┼'),
            "result columns and rows need intersections"
        );
        assert!(
            buffer.content().iter().any(|cell| cell.bg == ACCENT),
            "active SQL tab should use the gold accent"
        );
    }

    #[test]
    fn sidebar_focus_colors_connections_explorer_and_active_file() {
        let profile = crate::config::Profile {
            name: "DEV_ORACLE".into(),
            ..Default::default()
        };
        let mut app = App::new(
            crate::config::Config {
                connections: vec![profile],
            },
            "unused".into(),
        );
        app.schema = "APP_DEV".into();
        app.objects = vec![crate::db::DbObject {
            owner: "APP_DEV".into(),
            name: "ORDERS".into(),
            kind: "TABLE".into(),
        }];
        app.docs = (0..4)
            .map(|i| {
                crate::app::Document::new(
                    "SELECT 1 FROM dual;",
                    Some(format!("file-{i}.sql").into()),
                )
            })
            .collect();
        app.doc = 3;
        let mut terminal = Terminal::new(ratatui::backend::TestBackend::new(120, 40)).unwrap();
        for (focus, title_y, row_y) in [(0, 2, 4), (1, 10, 12), (2, 33, 36)] {
            app.focus = focus;
            terminal.draw(|frame| draw(frame, &mut app)).unwrap();
            let buffer = terminal.backend().buffer();
            assert_eq!(buffer[(1, title_y)].fg, ACCENT, "section {focus} title");
            assert_eq!(buffer[(1, row_y)].bg, ACCENT, "section {focus} selection");
        }
        app.focus = 3;
        terminal.draw(|frame| draw(frame, &mut app)).unwrap();
        let buffer = terminal.backend().buffer();
        for row_y in [4, 12, 36] {
            assert_eq!(
                buffer[(1, row_y)].bg,
                PANEL,
                "inactive selection at {row_y}"
            );
        }
    }

    #[test]
    fn result_grid_stays_in_bounds_on_narrow_terminal() {
        let mut app = App::new(crate::config::Config::default(), "unused".into());
        app.focus = 3;
        app.column = 2;
        app.results = vec![crate::db::QueryResult {
            columns: vec!["A".into(), "B".into(), "VERY_LONG_COLUMN_NAME".into()],
            rows: vec![vec![
                "1".into(),
                "2".into(),
                "A long value with Unicode: Lucía".into(),
            ]],
            ..Default::default()
        }];
        let rendered = rendered_text(&mut app, 45, 14);
        assert!(rendered.contains("VERY_LONG"));
        assert!(rendered.contains('┼'));
    }

    #[test]
    fn minimum_size_keeps_panel_navigation_and_form_actions_visible() {
        let mut app = App::new(crate::config::Config::default(), "unused".into());
        for focus in 0..4 {
            app.focus = focus;
            let rendered = rendered_text(&mut app, 45, 14);
            assert!(rendered.contains("Tab/Shift+Tab Panel"));
            assert!(rendered.contains("F1 Help"));
        }
        app.modal = Some(Modal::Form(Box::new(crate::app::ConnectionForm::new(
            crate::config::Profile::default(),
        ))));
        let rendered = rendered_text(&mut app, 45, 14);
        assert!(rendered.contains("F2 Save"));
        assert!(rendered.contains("Esc Cancel"));
    }

    #[test]
    fn result_row_number_expands_for_four_digits() {
        let mut app = App::new(crate::config::Config::default(), "unused".into());
        app.focus = 3;
        app.row = 999;
        app.results = vec![crate::db::QueryResult {
            columns: vec!["ID".into()],
            rows: (1..=1000).map(|i| vec![i.to_string()]).collect(),
            ..Default::default()
        }];
        assert!(rendered_text(&mut app, 80, 24).contains("1000│"));
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
