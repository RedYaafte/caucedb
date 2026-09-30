use crate::{config::Profile, sql, Error, Result};
use oracle::{Connection, Connector, Privilege};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Default)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub affected: u64,
    pub elapsed: Duration,
    pub truncated: bool,
    pub message: String,
}
#[derive(Clone, Debug)]
pub struct DbObject {
    pub owner: String,
    pub name: String,
    pub kind: String,
}

pub trait Database {
    fn execute(&mut self, sql: &str, limit: usize) -> Result<QueryResult>;
    fn schemas(&self) -> Result<Vec<String>>;
    fn objects(&self, owner: &str) -> Result<Vec<DbObject>>;
    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
}
pub struct OracleDb {
    pub conn: Arc<Connection>,
}
impl OracleDb {
    pub fn current_schema(&self) -> Result<String> {
        Ok(self.conn.query_row_as::<String>(
            "SELECT sys_context('USERENV','CURRENT_SCHEMA') FROM dual",
            &[],
        )?)
    }
    pub fn connect(p: &Profile) -> Result<Self> {
        p.validate()?;
        let (user, password) = p.credentials();
        let mut builder = Connector::new(user, password, p.connect_string());
        if p.auth == "External" {
            builder.external_auth(true);
        }
        match p.role.as_str() {
            "SYSDBA" => {
                builder.privilege(Privilege::Sysdba);
            }
            "SYSOPER" => {
                builder.privilege(Privilege::Sysoper);
            }
            _ => {}
        }
        let mut conn = builder.connect()?;
        conn.set_autocommit(false);
        conn.set_call_timeout(Some(Duration::from_secs(p.call_timeout as u64)))?;
        conn.set_module("caucedb")?;
        Ok(Self {
            conn: Arc::new(conn),
        })
    }
}
impl Database for OracleDb {
    fn execute(&mut self, source: &str, limit: usize) -> Result<QueryResult> {
        let started = Instant::now();
        let mut stmt = self
            .conn
            .statement(source)
            .fetch_array_size(100)
            .lob_locator()
            .build()?;
        let mut result = QueryResult::default();
        let mut bytes_used = 0usize;
        if stmt.is_query() {
            let rows = stmt.query(&[])?;
            result.columns = rows
                .column_info()
                .iter()
                .map(|c| c.name().to_owned())
                .collect();
            for row in rows {
                let row = row?;
                if result.rows.len() >= limit {
                    result.truncated = true;
                    break;
                }
                let mut cells = Vec::with_capacity(result.columns.len());
                for i in 0..result.columns.len() {
                    use oracle::sql_type::{Blob, Clob, Nclob, OracleType};
                    use std::io::Read;
                    let cell: std::result::Result<Option<String>, oracle::Error> =
                        match row.column_info()[i].oracle_type() {
                            OracleType::CLOB => row.get::<_, Option<Clob>>(i).map(|lob| {
                                lob.map(|v| {
                                    let mut bytes = Vec::new();
                                    match v.take(4096).read_to_end(&mut bytes) {
                                        Ok(_) => format!(
                                            "{} [CLOB preview]",
                                            String::from_utf8_lossy(&bytes)
                                        ),
                                        Err(e) => format!("<LOB read error: {e}>"),
                                    }
                                })
                            }),
                            OracleType::NCLOB => row.get::<_, Option<Nclob>>(i).map(|lob| {
                                lob.map(|v| {
                                    let mut bytes = Vec::new();
                                    match v.take(4096).read_to_end(&mut bytes) {
                                        Ok(_) => format!(
                                            "{} [NCLOB preview]",
                                            String::from_utf8_lossy(&bytes)
                                        ),
                                        Err(e) => format!("<LOB read error: {e}>"),
                                    }
                                })
                            }),
                            OracleType::BLOB => row.get::<_, Option<Blob>>(i).map(|lob| {
                                lob.map(|v| {
                                    let mut bytes = Vec::new();
                                    match v.take(128).read_to_end(&mut bytes) {
                                        Ok(_) => format!(
                                            "{} [BLOB hex preview]",
                                            bytes
                                                .iter()
                                                .map(|b| format!("{b:02X}"))
                                                .collect::<String>()
                                        ),
                                        Err(e) => format!("<LOB read error: {e}>"),
                                    }
                                })
                            }),
                            _ => row.get::<_, Option<String>>(i),
                        };
                    let value = match cell {
                        Ok(Some(s)) => {
                            if s.chars().count() > 4096 {
                                format!("{}… [truncated]", s.chars().take(4096).collect::<String>())
                            } else {
                                s
                            }
                        }
                        Ok(None) => "NULL".into(),
                        Err(e) => format!("<unsupported: {e}>"),
                    };
                    cells.push(value);
                }
                bytes_used += cells.iter().map(String::len).sum::<usize>();
                if bytes_used > 8 * 1024 * 1024 {
                    result.truncated = true;
                    break;
                }
                result.rows.push(cells);
            }
            result.message = format!(
                "{} rows{}",
                result.rows.len(),
                if result.truncated {
                    " · limit reached"
                } else {
                    ""
                }
            );
        } else {
            stmt.execute(&[])?;
            result.affected = stmt.row_count()?;
            result.message = format!("{} rows affected", result.affected);
        }
        result.elapsed = started.elapsed();
        Ok(result)
    }
    fn schemas(&self) -> Result<Vec<String>> {
        self.conn
            .query_as::<String>("SELECT DISTINCT owner FROM all_objects UNION SELECT sys_context('USERENV','CURRENT_SCHEMA') FROM dual ORDER BY 1", &[])?
            .map(|r| r.map_err(Error::from))
            .collect()
    }
    fn objects(&self, owner: &str) -> Result<Vec<DbObject>> {
        self.conn.query_as::<(String, String)>("SELECT object_name, object_type FROM all_objects WHERE owner = :1 AND object_type IN ('TABLE','VIEW','MATERIALIZED VIEW','PROCEDURE','FUNCTION','PACKAGE','SEQUENCE','SYNONYM','TRIGGER','TYPE') ORDER BY object_type, object_name", &[&owner])?
            .map(|r| r.map(|(name, kind)| DbObject { owner: owner.into(), name, kind }).map_err(Error::from)).collect()
    }
    fn commit(&self) -> Result<()> {
        self.conn.commit()?;
        Ok(())
    }
    fn rollback(&self) -> Result<()> {
        self.conn.rollback()?;
        Ok(())
    }
}

pub enum Command {
    Connect(Box<Profile>, bool),
    Run(String, usize),
    Schemas,
    Objects(String),
    Commit,
    Rollback,
    Disconnect,
    Shutdown,
}
pub enum Event {
    Connected(String),
    Tested(Duration),
    Result(QueryResult),
    Schemas(Vec<String>),
    Objects(Vec<DbObject>),
    Transaction(bool),
    Disconnected,
    Message(String),
    Error(String),
    Done,
}
pub struct Worker {
    tx: Sender<Command>,
    pub rx: Receiver<Event>,
    connection: Arc<Mutex<Option<Arc<Connection>>>>,
    cancelled: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Default for Worker {
    fn default() -> Self {
        Self::new()
    }
}
impl Worker {
    pub fn new() -> Self {
        let (tx, commands) = mpsc::channel();
        let (events, rx) = mpsc::channel();
        let connection = Arc::new(Mutex::new(None));
        let shared = connection.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let stop = cancelled.clone();
        let thread = std::thread::spawn(move || {
            let mut db: Option<OracleDb> = None;
            for command in commands {
                let operation = match &command {
                    Command::Connect(_, true) => "test",
                    Command::Connect(_, false) => "connect",
                    Command::Run(_, _) => "execute",
                    Command::Schemas => "schemas",
                    Command::Objects(_) => "objects",
                    Command::Commit => "commit",
                    Command::Rollback => "rollback",
                    Command::Disconnect => "disconnect",
                    Command::Shutdown => "shutdown",
                };
                tracing::info!(operation, "database operation");
                if matches!(command, Command::Shutdown) {
                    break;
                }
                let result = handle(command, &mut db, &events, &shared, &stop);
                if let Err(e) = result {
                    tracing::warn!(
                        operation,
                        "database operation failed; details are available in the TUI"
                    );
                    let text = e.to_string();
                    let text = if text.contains("DPI-1047") {
                        "Oracle Client not found. Install Oracle Instant Client and set LD_LIBRARY_PATH before starting. See README.md.".into()
                    } else {
                        text
                    };
                    let _ = events.send(Event::Error(text));
                }
                if events.send(Event::Done).is_err() {
                    break;
                }
            }
            if let Some(db) = db {
                let _ = db.rollback();
            }
        });
        Self {
            tx,
            rx,
            connection,
            cancelled,
            thread: Some(thread),
        }
    }
    pub fn send(&self, command: Command) -> Result<()> {
        self.cancelled.store(false, Ordering::SeqCst);
        self.tx
            .send(command)
            .map_err(|_| Error::Message("Database worker stopped".into()))
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        let shared = self.connection.clone();
        std::thread::spawn(move || {
            if let Ok(guard) = shared.lock() {
                if let Some(conn) = guard.as_ref() {
                    let _ = conn.break_execution();
                }
            }
        });
    }
}
fn handle(
    command: Command,
    db: &mut Option<OracleDb>,
    events: &Sender<Event>,
    shared: &Mutex<Option<Arc<Connection>>>,
    cancelled: &AtomicBool,
) -> Result<()> {
    let emit = |e| {
        let _ = events.send(e);
    };
    match command {
        Command::Shutdown => {}
        Command::Disconnect => {
            if let Some(active) = db.as_ref() {
                active.rollback()?;
            }
            if let Ok(mut guard) = shared.lock() {
                *guard = None;
            }
            *db = None;
            emit(Event::Transaction(false));
            emit(Event::Disconnected);
        }
        Command::Connect(p, test) => {
            let started = Instant::now();
            let new = OracleDb::connect(&p)?;
            new.conn.ping()?;
            new.conn.query_row("SELECT 1 FROM dual", &[])?;
            if test {
                emit(Event::Tested(started.elapsed()));
            } else {
                let schema = new.current_schema()?;
                if let Some(previous) = db.as_ref() {
                    previous.rollback()?;
                }
                if let Ok(mut guard) = shared.lock() {
                    *guard = Some(new.conn.clone());
                }
                *db = Some(new);
                emit(Event::Connected(schema));
                emit(Event::Transaction(false));
            }
        }
        command => {
            let db = db
                .as_mut()
                .ok_or_else(|| Error::Message("Connect to Oracle first (F2)".into()))?;
            match command {
                Command::Run(source, limit) => {
                    let statements = sql::split(&source)?;
                    if statements.is_empty() {
                        return Err(Error::Message("No SQL to execute".into()));
                    }
                    if statements.len() > 1000 {
                        return Err(Error::Message(
                            "Scripts are limited to 1000 statements per run".into(),
                        ));
                    }
                    for (index, stmt) in statements.iter().enumerate() {
                        if cancelled.load(Ordering::SeqCst) {
                            return Err(Error::Message(
                                "Script cancelled before next statement".into(),
                            ));
                        }
                        let keyword = sql::first_keyword(&stmt.text);
                        // PL/SQL may perform arbitrary DML, commit or DDL: show a conservative state.
                        if !["SELECT", "WITH", "EXPLAIN"].contains(&keyword.as_str())
                            || stmt.text.to_ascii_uppercase().contains("FOR UPDATE")
                        {
                            emit(Event::Transaction(true));
                        }
                        if [
                            "CREATE", "ALTER", "DROP", "TRUNCATE", "GRANT", "REVOKE", "COMMENT",
                        ]
                        .contains(&keyword.as_str())
                        {
                            emit(Event::Message(
                                "Oracle DDL commits implicitly, including preceding DML.".into(),
                            ));
                        }
                        match db.execute(&stmt.text, limit) {
                            Ok(mut r) => {
                                if keyword == "COMMIT" || (keyword == "ROLLBACK" && !stmt.text.to_ascii_uppercase().contains(" TO ")) { emit(Event::Transaction(false)); }
                                r.message = format!("Statement {} · {} · {:.0} ms", index+1, r.message, r.elapsed.as_secs_f64()*1000.0);
                                emit(Event::Result(r));
                            }
                            Err(e) => return Err(Error::Message(format!("Statement {} (line {}): {e}. Script stopped; preceding changes may remain pending.", index+1, stmt.line))),
                        }
                    }
                }
                Command::Schemas => emit(Event::Schemas(db.schemas()?)),
                Command::Objects(owner) => emit(Event::Objects(db.objects(&owner)?)),
                Command::Commit => {
                    db.commit()?;
                    emit(Event::Transaction(false));
                    emit(Event::Message("Committed".into()));
                }
                Command::Rollback => {
                    db.rollback()?;
                    emit(Event::Transaction(false));
                    emit(Event::Message("Rolled back".into()));
                }
                Command::Connect(_, _) | Command::Disconnect | Command::Shutdown => unreachable!(),
            }
        }
    }
    Ok(())
}

pub fn inspect_sql(object: &DbObject, tab: usize) -> String {
    let owner = sql::literal(&object.owner);
    let name = sql::literal(&object.name);
    match tab {
        0 => format!("SELECT column_name, data_type, data_length, data_precision, data_scale, nullable, data_default FROM all_tab_columns WHERE owner={owner} AND table_name={name} ORDER BY column_id"),
        1 => format!("SELECT c.constraint_name, c.constraint_type, c.status, cc.column_name, c.r_owner, c.r_constraint_name FROM all_constraints c LEFT JOIN all_cons_columns cc ON cc.owner=c.owner AND cc.constraint_name=c.constraint_name WHERE c.owner={owner} AND c.table_name={name} ORDER BY c.constraint_name, cc.position"),
        2 => format!("SELECT i.index_name, i.uniqueness, i.status, c.column_name, c.column_position FROM all_indexes i LEFT JOIN all_ind_columns c ON c.index_owner=i.owner AND c.index_name=i.index_name WHERE i.table_owner={owner} AND i.table_name={name} ORDER BY i.index_name, c.column_position"),
        _ => format!("SELECT type, line, text FROM all_source WHERE owner={owner} AND name={name} ORDER BY type, line"),
    }
}
