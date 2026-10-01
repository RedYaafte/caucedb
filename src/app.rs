use crate::{
    config::{self, Config, Profile},
    db::{self, Command, DbObject, Event, QueryResult, Worker},
    sql, Error, Result,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{path::PathBuf, time::Instant};
use tui_textarea::{CursorMove, TextArea};

pub struct Document {
    pub editor: TextArea<'static>,
    pub path: Option<PathBuf>,
    pub saved: String,
    pub crlf: bool,
}
impl Document {
    pub fn new(text: &str, path: Option<PathBuf>) -> Self {
        let normalized = text.replace("\r\n", "\n");
        let mut editor = TextArea::new(normalized.split('\n').map(String::from).collect());
        editor.set_line_number_style(ratatui::style::Style::default().fg(crate::ui::MUTED));
        editor.set_tab_length(4);
        Self {
            editor,
            path,
            saved: normalized,
            crlf: text.contains("\r\n"),
        }
    }
    pub fn text(&self) -> String {
        self.editor.lines().join("\n")
    }
    pub fn dirty(&self) -> bool {
        self.editor
            .lines()
            .iter()
            .map(String::as_str)
            .ne(self.saved.split('\n'))
    }
    pub fn title(&self) -> String {
        format!(
            "{}{}",
            self.path
                .as_ref()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or("Untitled.sql".into()),
            if self.dirty() { " *" } else { "" }
        )
    }
}
pub struct Field {
    pub label: &'static str,
    pub value: String,
    pub options: &'static [&'static str],
    pub secret: bool,
    pub section: usize,
}
pub struct ConnectionForm {
    pub original: Profile,
    pub fields: Vec<Field>,
    pub selected: usize,
    pub section: usize,
    pub action_attempted: bool,
}
impl ConnectionForm {
    pub fn new(p: Profile) -> Self {
        let mut fields = Vec::new();
        let mut field = |label, value: String, options, secret, section| {
            fields.push(Field {
                label,
                value,
                options,
                secret,
                section,
            })
        };
        field("Connection Name", p.name.clone(), &[], false, 4); // 0
        field("Connection Type", p.engine.clone(), &["Oracle"], false, 4);
        field("Hostname", p.host.clone(), &[], false, 0);
        field("Port", p.port.to_string(), &[], false, 0);
        field(
            "Type",
            p.address_type.clone(),
            &["Service Name", "SID", "TNS / Descriptor"],
            false,
            0,
        );
        field("Service Name / SID", p.service.clone(), &[], false, 0);
        field("Protocol", p.protocol.clone(), &["TCP", "TCPS"], false, 0);
        field(
            "TNS alias / Descriptor",
            p.descriptor.clone(),
            &[],
            false,
            1,
        );
        field("Wallet directory (TCPS)", p.wallet.clone(), &[], false, 1);
        field(
            "Connect timeout (seconds)",
            p.connect_timeout.to_string(),
            &[],
            false,
            1,
        );
        field(
            "Call timeout (seconds)",
            p.call_timeout.to_string(),
            &[],
            false,
            1,
        );
        field("Result row limit", p.row_limit.to_string(), &[], false, 1);
        field(
            "Authentication Type",
            p.auth.clone(),
            &["Default", "External"],
            false,
            2,
        );
        field(
            "Role",
            p.role.clone(),
            &["Default", "SYSDBA", "SYSOPER"],
            false,
            2,
        );
        field(
            "Username (target for proxy)",
            p.username.clone(),
            &[],
            false,
            2,
        );
        field("Password", p.password.clone(), &[], true, 2);
        field(
            "Save password",
            p.save_password.to_string(),
            &["false", "true"],
            false,
            2,
        );
        field(
            "Use proxy user",
            p.proxy_enabled.to_string(),
            &["false", "true"],
            false,
            3,
        );
        field("Proxy username", p.proxy_username.clone(), &[], false, 3);
        field("Proxy password", p.proxy_password.clone(), &[], true, 3);
        Self {
            original: p,
            fields,
            selected: 0,
            section: 0,
            action_attempted: false,
        }
    }
    pub fn visible(&self) -> Vec<usize> {
        self.fields
            .iter()
            .enumerate()
            .filter(|(_, f)| f.section == 4 || f.section == self.section)
            .map(|(i, _)| i)
            .collect()
    }
    pub fn profile(&self) -> Result<Profile> {
        let v = |i: usize| self.fields[i].value.clone();
        let mut p = self.original.clone();
        let number = |i: usize| {
            v(i).parse::<u32>().map_err(|_| {
                Error::Message(format!(
                    "{} must be a positive number",
                    self.fields[i].label
                ))
            })
        };
        p.name = v(0);
        p.engine = v(1);
        p.host = v(2);
        p.port =
            u16::try_from(number(3)?).map_err(|_| Error::Message("Port must be 1–65535".into()))?;
        p.address_type = v(4);
        p.service = v(5);
        p.protocol = v(6);
        p.descriptor = v(7);
        p.wallet = v(8);
        p.connect_timeout = number(9)?;
        p.call_timeout = number(10)?;
        p.row_limit = number(11)? as usize;
        p.auth = v(12);
        p.role = v(13);
        p.username = v(14);
        p.password = v(15);
        p.save_password = v(16) == "true";
        p.proxy_enabled = v(17) == "true";
        p.proxy_username = v(18);
        p.proxy_password = v(19);
        p.validate()?;
        Ok(p)
    }
    pub fn key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::PageDown || key.code == KeyCode::PageUp {
            self.section = (self.section + if key.code == KeyCode::PageDown { 1 } else { 3 }) % 4;
            self.selected = self.visible()[2];
            return;
        }
        let visible = self.visible();
        let at = visible
            .iter()
            .position(|i| *i == self.selected)
            .unwrap_or(0);
        match key.code {
            KeyCode::Tab | KeyCode::Down => self.selected = visible[(at + 1) % visible.len()],
            KeyCode::BackTab | KeyCode::Up => {
                self.selected = visible[(at + visible.len() - 1) % visible.len()]
            }
            _ => {
                let f = &mut self.fields[self.selected];
                if !f.options.is_empty() {
                    if matches!(
                        key.code,
                        KeyCode::Left | KeyCode::Right | KeyCode::Char(' ') | KeyCode::Enter
                    ) {
                        let at = f.options.iter().position(|s| *s == f.value).unwrap_or(0);
                        let delta = if key.code == KeyCode::Left {
                            f.options.len() - 1
                        } else {
                            1
                        };
                        f.value = f.options[(at + delta) % f.options.len()].into();
                        self.action_attempted = false;
                    }
                } else if key.modifiers.contains(KeyModifiers::CONTROL)
                    && key.code == KeyCode::Char('u')
                {
                    f.value.clear();
                    self.action_attempted = false;
                } else {
                    match key.code {
                        KeyCode::Char(c)
                            if !key
                                .modifiers
                                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                        {
                            f.value.push(c);
                            self.action_attempted = false;
                        }
                        KeyCode::Backspace => {
                            f.value.pop();
                            self.action_attempted = false;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
#[derive(Clone)]
pub enum PromptKind {
    Open,
    Save,
    Search,
    Schema,
    Filter,
}
pub enum Confirm {
    Quit,
    CloseDoc,
    Connect(Box<Profile>),
    Overwrite(PathBuf),
    Disconnect,
}
pub enum Modal {
    Form(Box<ConnectionForm>),
    Prompt(PromptKind, String),
    Confirm(Confirm, String),
    Help,
    Cell(String),
}

pub struct App {
    pub config: Config,
    pub config_path: PathBuf,
    pub worker: Worker,
    pub profiles_index: usize,
    pub objects: Vec<DbObject>,
    pub object_index: usize,
    pub filter: String,
    pub schemas: Vec<String>,
    pub schema: String,
    pub connected: Option<Profile>,
    pub docs: Vec<Document>,
    pub doc: usize,
    pub focus: usize,
    pub results: Vec<QueryResult>,
    pub result: usize,
    pub row: usize,
    pub column: usize,
    pub modal: Option<Modal>,
    pub busy: bool,
    pub pending: bool,
    pub quit: bool,
    pub status: String,
    pub error: bool,
    pub messages: Vec<String>,
    pub started: Instant,
    pub working_label: Option<&'static str>,
    pub followup: Option<Command>,
    pub close_form_on_connect: bool,
    pub connecting: Option<Profile>,
    pub detail_scroll: u16,
}
impl App {
    pub fn new(config: Config, config_path: PathBuf) -> Self {
        Self {
            config,
            config_path,
            worker: Worker::new(),
            profiles_index: 0,
            objects: vec![],
            object_index: 0,
            filter: String::new(),
            schemas: vec![],
            schema: String::new(),
            connected: None,
            docs: vec![Document::new("", None)],
            doc: 0,
            focus: 0,
            results: vec![],
            result: 0,
            row: 0,
            column: 0,
            modal: None,
            busy: false,
            pending: false,
            quit: false,
            status: "Welcome. Press n to add your first Oracle connection; F1 for help.".into(),
            error: false,
            messages: vec![],
            started: Instant::now(),
            working_label: None,
            followup: None,
            close_form_on_connect: false,
            connecting: None,
            detail_scroll: 0,
        }
    }
    pub fn message(&mut self, s: impl Into<String>, error: bool) {
        self.status = s.into();
        self.error = error;
        self.messages.push(self.status.clone());
        if self.messages.len() > 100 {
            self.messages.remove(0);
        }
    }
    pub fn send(&mut self, command: Command) {
        if self.busy {
            self.message(
                "An operation is running. F8 cancels the active query.",
                true,
            );
            return;
        }
        let label = match &command {
            Command::Connect(_, true) => "Testing Oracle connection",
            Command::Connect(_, false) => "Connecting to Oracle",
            Command::Run(_, _) => "Running SQL",
            Command::Schemas => "Loading schemas",
            Command::Objects(_) => "Loading objects",
            Command::Commit => "Committing transaction",
            Command::Rollback => "Rolling back transaction",
            Command::Disconnect => "Disconnecting",
            Command::Shutdown => "Shutting down",
        };
        match self.worker.send(command) {
            Ok(()) => {
                self.busy = true;
                self.started = Instant::now();
                self.working_label = Some(label);
                self.message(format!("{label}…"), false);
            }
            Err(e) => self.message(e.to_string(), true),
        }
    }
    pub fn poll(&mut self) {
        while let Ok(event) = self.worker.rx.try_recv() {
            match event {
                Event::Connected(schema) => {
                    self.connected = self.connecting.take();
                    if let Some(p) = self.connected.as_mut() {
                        p.password.clear();
                        p.proxy_password.clear();
                    }
                    self.schema = schema.clone();
                    self.objects.clear();
                    self.results.clear();
                    self.object_index = 0;
                    self.filter.clear();
                    if self.close_form_on_connect {
                        self.modal = None;
                    }
                    self.close_form_on_connect = false;
                    self.followup = Some(Command::Objects(schema));
                    self.focus = 2;
                    self.message("Connected · autocommit OFF", false);
                }
                Event::Tested(t) => self.message(
                    format!(
                        "Test passed · authenticated query and ping · {:.0} ms",
                        t.as_secs_f64() * 1000.
                    ),
                    false,
                ),
                Event::Result(r) => {
                    self.message(r.message.clone(), false);
                    self.results.push(r);
                    if self.results.len() > 20 {
                        self.results.remove(0);
                    }
                    self.result = self.results.len() - 1;
                    self.row = 0;
                    self.column = 0;
                }
                Event::Schemas(s) => {
                    self.schemas = s;
                    self.modal = Some(Modal::Prompt(PromptKind::Schema, self.schema.clone()));
                    self.message("Enter a schema from the list", false);
                }
                Event::Objects(o) => {
                    self.objects = o;
                    self.object_index = 0;
                    self.message(
                        format!("{} objects in {}", self.objects.len(), self.schema),
                        false,
                    );
                }
                Event::Transaction(p) => self.pending = p,
                Event::Disconnected => {
                    self.connected = None;
                    self.pending = false;
                    self.objects.clear();
                    self.schema.clear();
                    self.message("Disconnected", false);
                }
                Event::Message(s) => self.message(s, false),
                Event::Error(e) => {
                    if self.close_form_on_connect {
                        self.connecting = None;
                        self.close_form_on_connect = false;
                    }
                    self.followup = None;
                    self.message(e, true);
                }
                Event::Done => {
                    self.busy = false;
                    self.working_label = None;
                    if let Some(c) = self.followup.take() {
                        self.send(c);
                    }
                }
            }
        }
    }
    pub fn filtered_objects(&self) -> Vec<&DbObject> {
        let f = self.filter.to_uppercase();
        self.objects
            .iter()
            .filter(|o| f.is_empty() || o.name.to_uppercase().contains(&f) || o.kind.contains(&f))
            .collect()
    }
    pub fn key(&mut self, key: KeyEvent) {
        // Help is a global escape hatch: it must work while a form, prompt,
        // confirmation or another detail dialog is open as well as from the
        // main workbench. The previous modal-first dispatch swallowed F1 in
        // those states.
        if key.code == KeyCode::F(1) {
            self.detail_scroll = 0;
            self.modal = Some(Modal::Help);
            return;
        }
        if self.modal.is_some() {
            self.modal_key(key);
            return;
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl {
            match key.code {
                KeyCode::Char('q') => {
                    self.request_quit();
                    return;
                }
                KeyCode::Char('n') => {
                    self.docs.push(Document::new("", None));
                    self.doc = self.docs.len() - 1;
                    self.focus = 2;
                    return;
                }
                KeyCode::Char('o') => {
                    self.modal = Some(Modal::Prompt(PromptKind::Open, String::new()));
                    return;
                }
                KeyCode::Char('s') => {
                    self.save_document(false);
                    return;
                }
                KeyCode::Char('w') => {
                    if self.docs[self.doc].dirty() {
                        self.modal = Some(Modal::Confirm(
                            Confirm::CloseDoc,
                            "Discard unsaved changes in this SQL file?".into(),
                        ));
                    } else {
                        self.close_doc();
                    }
                    return;
                }
                KeyCode::Char('f') => {
                    self.modal = Some(Modal::Prompt(PromptKind::Search, String::new()));
                    return;
                }
                KeyCode::Left => {
                    self.doc = (self.doc + self.docs.len() - 1) % self.docs.len();
                    return;
                }
                KeyCode::Right => {
                    self.doc = (self.doc + 1) % self.docs.len();
                    return;
                }
                _ => {}
            }
        }
        match key.code {
            KeyCode::F(2) => self.edit_connection(),
            KeyCode::F(3) => self.send(Command::Schemas),
            KeyCode::F(4) => self.save_document(true),
            KeyCode::F(5) => self.run_sql(false),
            KeyCode::F(6) => self.run_sql(true),
            KeyCode::F(7) => self.send(Command::Commit),
            KeyCode::F(8) => {
                self.worker.cancel();
                self.message(
                    "Cancellation requested (connection setup uses its timeout)",
                    false,
                );
            }
            KeyCode::F(9) => self.send(Command::Rollback),
            KeyCode::F(10) => {
                self.detail_scroll = 0;
                self.modal = Some(Modal::Cell(self.messages.join("\n\n")));
            }
            KeyCode::Tab => self.focus = (self.focus + 1) % 4,
            KeyCode::BackTab => self.focus = (self.focus + 3) % 4,
            KeyCode::Esc => self.focus = 0,
            _ if self.focus == 2 => {
                self.docs[self.doc].editor.input(key);
            }
            _ if self.focus == 0 => match key.code {
                KeyCode::Char('n') => {
                    self.modal = Some(Modal::Form(Box::new(ConnectionForm::new(
                        Profile::default(),
                    ))))
                }
                KeyCode::Char('e') => self.edit_connection(),
                KeyCode::Char('d') => {
                    if self.connected.is_some() {
                        self.modal = Some(Modal::Confirm(
                            Confirm::Disconnect,
                            "Disconnect? Pending work will be rolled back.".into(),
                        ));
                    }
                }
                KeyCode::Up => self.profiles_index = self.profiles_index.saturating_sub(1),
                KeyCode::Down => {
                    self.profiles_index = (self.profiles_index + 1)
                        .min(self.config.connections.len().saturating_sub(1))
                }
                KeyCode::Enter => self.connect_selected(),
                _ => {}
            },
            _ if self.focus == 1 => match key.code {
                KeyCode::Up => self.object_index = self.object_index.saturating_sub(1),
                KeyCode::Down => {
                    self.object_index =
                        (self.object_index + 1).min(self.filtered_objects().len().saturating_sub(1))
                }
                KeyCode::Char('/') => {
                    self.modal = Some(Modal::Prompt(PromptKind::Filter, self.filter.clone()))
                }
                KeyCode::Char('r') => self.send(Command::Objects(self.schema.clone())),
                KeyCode::Enter => self.inspect(0),
                KeyCode::Char('1') => self.inspect(0),
                KeyCode::Char('2') => self.inspect(1),
                KeyCode::Char('3') => self.inspect(2),
                KeyCode::Char('4') => self.inspect(3),
                KeyCode::Char('p') => self.preview(),
                _ => {}
            },
            _ if self.focus == 3 => {
                let len = self.results.get(self.result).map_or(0, |r| r.rows.len());
                let cols = self.results.get(self.result).map_or(0, |r| r.columns.len());
                match key.code {
                    KeyCode::Down => self.row = (self.row + 1).min(len.saturating_sub(1)),
                    KeyCode::Up => self.row = self.row.saturating_sub(1),
                    KeyCode::PageDown => self.row = (self.row + 20).min(len.saturating_sub(1)),
                    KeyCode::PageUp => self.row = self.row.saturating_sub(20),
                    KeyCode::Home => self.row = 0,
                    KeyCode::End => self.row = len.saturating_sub(1),
                    KeyCode::Left => self.column = self.column.saturating_sub(1),
                    KeyCode::Right => self.column = (self.column + 1).min(cols.saturating_sub(1)),
                    KeyCode::Char('[') => {
                        self.result = self.result.saturating_sub(1);
                        self.row = 0;
                        self.column = 0;
                    }
                    KeyCode::Char(']') => {
                        self.result = (self.result + 1).min(self.results.len().saturating_sub(1));
                        self.row = 0;
                        self.column = 0;
                    }
                    KeyCode::Enter => {
                        if let Some(cell) = self
                            .results
                            .get(self.result)
                            .and_then(|r| r.rows.get(self.row))
                            .and_then(|r| r.get(self.column))
                        {
                            self.detail_scroll = 0;
                            self.modal = Some(Modal::Cell(cell.clone()));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    fn edit_connection(&mut self) {
        let mut p = self
            .config
            .connections
            .get(self.profiles_index)
            .cloned()
            .unwrap_or_default();
        if let Err(e) = p.load_secrets() {
            self.message(e.to_string(), true);
        }
        self.modal = Some(Modal::Form(Box::new(ConnectionForm::new(p))));
    }
    fn connect_selected(&mut self) {
        if let Some(mut p) = self.config.connections.get(self.profiles_index).cloned() {
            if p.load_secrets().is_err() || (p.auth == "Default" && p.credentials().1.is_empty()) {
                let mut f = ConnectionForm::new(p);
                f.section = 2;
                f.selected = 15;
                self.modal = Some(Modal::Form(Box::new(f)));
            } else {
                self.request_connect(p);
            }
        } else {
            self.edit_connection();
        }
    }
    fn request_connect(&mut self, p: Profile) {
        if self.busy {
            self.message("Wait for the current operation before connecting", true);
            return;
        }
        if self.pending {
            self.modal = Some(Modal::Confirm(
                Confirm::Connect(Box::new(p)),
                "Switch connection? The previous session's uncommitted work will be rolled back."
                    .into(),
            ));
        } else {
            self.connect(p);
        }
    }
    fn connect(&mut self, p: Profile) {
        self.connecting = Some(p.clone());
        self.close_form_on_connect = true;
        self.send(Command::Connect(Box::new(p), false));
    }
    pub fn run_sql(&mut self, all: bool) {
        let doc = &self.docs[self.doc];
        let text = doc.text();
        let source = if all {
            text
        } else if let Some(((r1, c1), (r2, c2))) = doc.editor.selection_range() {
            let mut selected = Vec::new();
            for r in r1..=r2 {
                let s = &doc.editor.lines()[r];
                let a = if r == r1 { c1 } else { 0 };
                let b = if r == r2 { c2 } else { s.chars().count() };
                selected.push(s.chars().skip(a).take(b - a).collect::<String>());
            }
            selected.join("\n")
        } else {
            let (row, col) = doc.editor.cursor();
            let offset = doc
                .editor
                .lines()
                .iter()
                .take(row)
                .map(|l| l.len() + 1)
                .sum::<usize>()
                + doc.editor.lines()[row]
                    .chars()
                    .take(col)
                    .map(char::len_utf8)
                    .sum::<usize>();
            match sql::split(&text) {
                Ok(stmts) => stmts
                    .iter()
                    .find(|s| s.range.contains(&offset))
                    .or_else(|| stmts.last().filter(|s| offset >= s.range.end))
                    .map(|s| s.text.clone())
                    .unwrap_or_default(),
                Err(e) => {
                    self.message(e.to_string(), true);
                    return;
                }
            }
        };
        let limit = self.connected.as_ref().map_or(1000, |p| p.row_limit);
        self.send(Command::Run(source, limit));
    }
    fn inspect(&mut self, tab: usize) {
        if let Some(o) = self.filtered_objects().get(self.object_index) {
            let query = db::inspect_sql(o, tab);
            self.send(Command::Run(query, 1000));
        }
    }
    fn preview(&mut self) {
        if let Some(o) = self.filtered_objects().get(self.object_index) {
            if !["TABLE", "VIEW", "MATERIALIZED VIEW", "SYNONYM"].contains(&o.kind.as_str()) {
                self.inspect(3);
                return;
            }
            let text = format!(
                "SELECT *\nFROM {}.{};",
                sql::quote_identifier(&o.owner),
                sql::quote_identifier(&o.name)
            );
            self.docs.push(Document::new(&text, None));
            self.doc = self.docs.len() - 1;
            self.focus = 2;
            self.run_sql(true);
        }
    }
    fn request_quit(&mut self) {
        if self.busy {
            self.message(
                "Cancel the active operation with F8 and wait before exiting",
                true,
            );
            return;
        }
        if self.pending || self.docs.iter().any(Document::dirty) {
            self.modal = Some(Modal::Confirm(
                Confirm::Quit,
                "Exit? Unsaved files will be discarded and pending transactions rolled back."
                    .into(),
            ));
        } else {
            self.quit = true;
        }
    }
    fn close_doc(&mut self) {
        self.docs.remove(self.doc);
        if self.docs.is_empty() {
            self.docs.push(Document::new("", None));
        }
        self.doc = self.doc.min(self.docs.len() - 1);
    }
    fn save_document(&mut self, save_as: bool) {
        if save_as || self.docs[self.doc].path.is_none() {
            self.modal = Some(Modal::Prompt(
                PromptKind::Save,
                self.docs[self.doc]
                    .path
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ));
        } else if let Some(path) = self.docs[self.doc].path.clone() {
            self.write_document(path);
        }
    }
    fn write_document(&mut self, path: PathBuf) {
        let text = self.docs[self.doc].text();
        let file_text = if self.docs[self.doc].crlf {
            text.replace('\n', "\r\n")
        } else {
            text.clone()
        };
        match config::atomic_write(&path, file_text.as_bytes()) {
            Ok(()) => {
                self.docs[self.doc].saved = text;
                self.docs[self.doc].path = Some(path.clone());
                self.message(format!("Saved {}", path.display()), false);
            }
            Err(e) => self.message(e.to_string(), true),
        }
    }
    fn modal_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::F(8) && self.busy {
            self.worker.cancel();
            self.message("Cancellation requested", false);
            return;
        }
        let modal = self.modal.take().unwrap();
        if key.code == KeyCode::Esc {
            return;
        }
        match modal {
            Modal::Form(mut form) => {
                let action = match key.code {
                    KeyCode::F(5) => Some(0),
                    KeyCode::F(6) => Some(1),
                    KeyCode::F(2) => Some(2),
                    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(2),
                    _ => None,
                };
                if let Some(action) = action {
                    form.action_attempted = true;
                    match form.profile() {
                        Err(e) => {
                            self.message(e.to_string(), true);
                            self.modal = Some(Modal::Form(form));
                        }
                        Ok(p) => match action {
                            0 => {
                                self.send(Command::Connect(Box::new(p), true));
                                self.modal = Some(Modal::Form(form));
                            }
                            1 => {
                                self.modal = Some(Modal::Form(form));
                                self.request_connect(p);
                            }
                            _ => {
                                let name = p.name.clone();
                                let result = self.save_profile(p);
                                match result {
                                    Ok(()) => self.message(
                                        format!("Saved connection '{name}' · not connected"),
                                        false,
                                    ),
                                    Err(e) => {
                                        self.message(e.to_string(), true);
                                        self.modal = Some(Modal::Form(form));
                                    }
                                }
                            }
                        },
                    }
                } else {
                    form.key(key);
                    self.modal = Some(Modal::Form(form));
                }
            }
            Modal::Prompt(kind, mut value) => match key.code {
                KeyCode::Enter => {
                    if let Err(e) = self.submit_prompt(kind.clone(), &value) {
                        self.message(e.to_string(), true);
                        self.modal = Some(Modal::Prompt(kind, value));
                    }
                }
                KeyCode::Backspace => {
                    value.pop();
                    self.modal = Some(Modal::Prompt(kind, value));
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.modal = Some(Modal::Prompt(kind, String::new()))
                }
                KeyCode::Char(c)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
                {
                    value.push(c);
                    self.modal = Some(Modal::Prompt(kind, value));
                }
                _ => self.modal = Some(Modal::Prompt(kind, value)),
            },
            Modal::Confirm(action, text) => {
                if key.code == KeyCode::Char('y') {
                    match action {
                        Confirm::Quit => self.quit = true,
                        Confirm::CloseDoc => self.close_doc(),
                        Confirm::Connect(p) => self.connect(*p),
                        Confirm::Overwrite(p) => self.write_document(p),
                        Confirm::Disconnect => {
                            if !self.busy {
                                self.send(Command::Disconnect);
                            }
                        }
                    }
                } else if key.code != KeyCode::Char('n') {
                    self.modal = Some(Modal::Confirm(action, text));
                }
            }
            Modal::Help => {
                match key.code {
                    KeyCode::Down => self.detail_scroll = self.detail_scroll.saturating_add(1),
                    KeyCode::Up => self.detail_scroll = self.detail_scroll.saturating_sub(1),
                    KeyCode::PageDown => self.detail_scroll = self.detail_scroll.saturating_add(10),
                    KeyCode::PageUp => self.detail_scroll = self.detail_scroll.saturating_sub(10),
                    _ => {}
                }
                self.modal = Some(Modal::Help);
            }
            Modal::Cell(text) => {
                match key.code {
                    KeyCode::Down => self.detail_scroll = self.detail_scroll.saturating_add(1),
                    KeyCode::Up => self.detail_scroll = self.detail_scroll.saturating_sub(1),
                    KeyCode::PageDown => self.detail_scroll = self.detail_scroll.saturating_add(10),
                    KeyCode::PageUp => self.detail_scroll = self.detail_scroll.saturating_sub(10),
                    _ => {}
                }
                self.modal = Some(Modal::Cell(text));
            }
        }
    }
    fn save_profile(&mut self, mut p: Profile) -> Result<()> {
        let previous = self.config.connections.iter().position(|c| c.id == p.id);
        if p.save_password || previous.is_some_and(|i| self.config.connections[i].save_password) {
            p.persist_secrets()?;
        }
        p.password.clear();
        p.proxy_password.clear();
        let mut connections = self.config.connections.clone();
        if let Some(i) = previous {
            connections[i] = p;
        } else {
            connections.push(p);
        }
        let config = Config { connections };
        config.save(&self.config_path)?;
        self.profiles_index = previous.unwrap_or(config.connections.len() - 1);
        self.config = config;
        Ok(())
    }
    fn submit_prompt(&mut self, kind: PromptKind, value: &str) -> Result<()> {
        match kind {
            PromptKind::Open => {
                let path = PathBuf::from(value);
                let metadata = std::fs::metadata(&path)?;
                if metadata.len() > 8 * 1024 * 1024 {
                    return Err(Error::Message(
                        "SQL files are limited to 8 MiB in this MVP".into(),
                    ));
                }
                let text = std::fs::read_to_string(&path)?;
                self.docs.push(Document::new(&text, Some(path)));
                self.doc = self.docs.len() - 1;
                self.focus = 2;
            }
            PromptKind::Save => {
                if value.trim().is_empty() {
                    return Err(Error::Message("Enter a file path".into()));
                }
                let mut path = PathBuf::from(value);
                if path.extension().is_none() {
                    path.set_extension("sql");
                }
                if path.exists() {
                    self.modal = Some(Modal::Confirm(
                        Confirm::Overwrite(path),
                        "Replace this existing file?".into(),
                    ));
                } else {
                    self.write_document(path);
                }
            }
            PromptKind::Search => {
                self.docs[self.doc]
                    .editor
                    .set_search_pattern(value)
                    .map_err(|e| Error::Message(e.to_string()))?;
                self.docs[self.doc].editor.search_forward(true);
                self.focus = 2;
            }
            PromptKind::Schema => {
                if !self.schemas.iter().any(|s| s == value) {
                    return Err(Error::Message(
                        "Choose an exact schema name from the list".into(),
                    ));
                }
                self.schema = value.into();
                self.filter.clear();
                self.send(Command::Objects(value.into()));
            }
            PromptKind::Filter => {
                self.filter = value.into();
                self.object_index = 0;
            }
        }
        Ok(())
    }
    pub fn paste(&mut self, text: String) {
        match &mut self.modal {
            Some(Modal::Form(form)) => {
                let field = &mut form.fields[form.selected];
                if field.options.is_empty() {
                    field.value.push_str(&text.replace(['\n', '\r'], ""));
                    form.action_attempted = false;
                }
            }
            Some(Modal::Prompt(_, value)) => value.push_str(&text.replace(['\n', '\r'], "")),
            None if self.focus == 2 => {
                self.docs[self.doc].editor.insert_str(text);
            }
            _ => {}
        }
    }
    pub fn open_initial(&mut self, path: &str) -> Result<()> {
        self.submit_prompt(PromptKind::Open, path)?;
        self.docs.remove(0);
        self.doc = 0;
        self.docs[0].editor.move_cursor(CursorMove::Top);
        Ok(())
    }
}
