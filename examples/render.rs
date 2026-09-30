//! Capture actual Ratatui cells as SVG for visual review. Fixtures are synthetic.
use caucedb::{
    app::{App, ConnectionForm, Document, Modal},
    config::{Config, Profile},
    db::{DbObject, QueryResult},
    ui,
};
use ratatui::{backend::TestBackend, style::Color, Terminal};
use std::{fmt::Write, fs};
fn color(c: Color, background: bool) -> String {
    // TestBackend cannot resolve a terminal's palette. These fallbacks only
    // make the local SVG fixture readable; the real TUI emits ANSI/default
    // colors and lets the terminal resolve them.
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Reset => if background { "#111111" } else { "#eeeeee" }.into(),
        Color::Black => "#000000".into(),
        Color::Red => "#cc0000".into(),
        Color::Green => "#00aa00".into(),
        Color::Yellow => "#aaaa00".into(),
        Color::Blue => "#0000cc".into(),
        Color::Magenta => "#aa00aa".into(),
        Color::Cyan => "#00aaaa".into(),
        Color::Gray => "#aaaaaa".into(),
        Color::DarkGray => "#555555".into(),
        Color::LightRed => "#ff5555".into(),
        Color::LightGreen => "#55ff55".into(),
        Color::LightYellow => "#ffff55".into(),
        Color::LightBlue => "#5555ff".into(),
        Color::LightMagenta => "#ff55ff".into(),
        Color::LightCyan => "#55ffff".into(),
        Color::White => "#ffffff".into(),
        Color::Indexed(_) => "#808080".into(),
    }
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
fn capture(app: &mut App, w: u16, h: u16, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = Terminal::new(TestBackend::new(w, h))?;
    terminal.draw(|f| ui::draw(f, app))?;
    let buf = terminal.backend().buffer();
    let mut svg=format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\"><rect width=\"100%\" height=\"100%\" fill=\"#111823\"/>",w*10,h*20);
    for y in 0..h {
        for x in 0..w {
            let cell = &buf[(x, y)];
            write!(
                svg,
                "<rect x=\"{}\" y=\"{}\" width=\"10\" height=\"20\" fill=\"{}\"/>",
                x * 10,
                y * 20,
                color(cell.bg, true)
            )?;
            if cell.symbol() != " " {
                write!(svg,"<text x=\"{}\" y=\"{}\" fill=\"{}\" font-family=\"DejaVu Sans Mono\" font-size=\"16\">{}</text>",x*10,y*20+16,color(cell.fg,false),escape(cell.symbol()))?;
            }
        }
    }
    svg.push_str("</svg>");
    fs::write(format!(".local/screens/{name}.svg"), svg)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(".local/screens")?;
    let p = Profile {
        name: "Local Oracle · DEMO".into(),
        username: "TUI_TEST".into(),
        ..Profile::default()
    };
    let mut app = App::new(
        Config {
            connections: vec![p.clone()],
        },
        "unused".into(),
    );
    app.connected = Some(p.clone());
    app.schema = "TUI_TEST".into();
    app.focus = 2;
    app.docs=vec![Document::new("-- Synthetic preview fixture\nSELECT employee_id, name, department\nFROM employees\nWHERE department = 'Engineering'\nORDER BY name;",Some("workspace/team.sql".into()))];
    app.objects = [
        ("EMPLOYEES", "TABLE"),
        ("DEPARTMENTS", "TABLE"),
        ("PROJECTS", "TABLE"),
        ("ACTIVE_EMPLOYEES", "VIEW"),
        ("REPORTING", "PACKAGE"),
    ]
    .iter()
    .map(|(n, k)| DbObject {
        owner: "TUI_TEST".into(),
        name: (*n).into(),
        kind: (*k).into(),
    })
    .collect();
    app.results = vec![QueryResult {
        columns: vec!["EMPLOYEE_ID".into(), "NAME".into(), "DEPARTMENT".into()],
        rows: vec![
            vec!["101".into(), "Ana Torres".into(), "Engineering".into()],
            vec!["102".into(), "Diego Ruiz".into(), "Engineering".into()],
            vec!["103".into(), "Lucía Pérez".into(), "Engineering".into()],
        ],
        ..QueryResult::default()
    }];
    app.status = "Synthetic fixture · 3 rows · F7 Commit / F9 Rollback".into();
    capture(&mut app, 120, 40, "workbench")?;
    capture(&mut app, 80, 24, "narrow")?;
    for section in 0..4 {
        let mut form = ConnectionForm::new(p.clone());
        form.section = section;
        form.selected = form.visible()[2];
        app.modal = Some(Modal::Form(Box::new(form)));
        capture(&mut app, 120, 40, &format!("form-{section}"))?;
    }
    capture(&mut app, 60, 20, "form-small")?;
    Ok(())
}
