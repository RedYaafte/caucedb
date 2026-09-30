//! Runs only when explicitly requested against the disposable Docker database.
use caucedb::{
    config::Profile,
    db::{Command, Database, Event, OracleDb, Worker},
    sql,
};
use std::time::Duration;

fn profile() -> Profile {
    Profile {
        name: "Docker test".into(),
        host: "127.0.0.1".into(),
        port: 1522,
        username: std::env::var("ORACLE_TEST_USER").unwrap_or("tui_test".into()),
        password: std::env::var("ORACLE_TEST_PASSWORD")
            .expect("Set ORACLE_TEST_PASSWORD for the disposable database"),
        ..Profile::default()
    }
}
fn drain(worker: &Worker) -> Vec<Event> {
    let mut events = Vec::new();
    loop {
        let event = worker
            .rx
            .recv_timeout(Duration::from_secs(45))
            .expect("Worker timed out");
        let done = matches!(event, Event::Done);
        events.push(event);
        if done {
            return events;
        }
    }
}
fn no_errors(events: &[Event]) {
    for e in events {
        if let Event::Error(s) = e {
            panic!("{s}");
        }
    }
}

#[test]
#[ignore = "requires Docker Oracle and Instant Client; see README"]
fn oracle_queries_metadata_transactions_and_plsql() {
    let p = profile();
    let mut db = OracleDb::connect(&p).unwrap();
    let mut observer = OracleDb::connect(&p).unwrap();
    db.conn.ping().unwrap();
    let name = format!("TUI_{}", &uuid::Uuid::new_v4().simple().to_string()[..16]).to_uppercase();
    db.execute(
        &format!(
            "CREATE TABLE {name} (id NUMBER PRIMARY KEY, label VARCHAR2(80), note CLOB, data BLOB)"
        ),
        100,
    )
    .unwrap();
    let checks = || {
        db.execute(&format!("INSERT INTO {name} VALUES (1, 'México; 日本', TO_CLOB('test CLOB'), hextoraw('CAFE'))"),100).unwrap();
        assert_eq!(
            observer
                .execute(&format!("SELECT count(*) FROM {name}"), 100)
                .unwrap()
                .rows[0][0],
            "0"
        );
        db.commit().unwrap();
        assert_eq!(
            observer
                .execute(&format!("SELECT count(*) FROM {name}"), 100)
                .unwrap()
                .rows[0][0],
            "1"
        );
        let data = db.execute(&format!("SELECT * FROM {name}"), 100).unwrap();
        assert_eq!(data.rows[0][1], "México; 日本");
        assert!(data.rows[0][2].contains("test CLOB"));
        assert!(data.rows[0][3].contains("CAFE"));
        db.execute(&format!("INSERT INTO {name}(id) VALUES (2)"), 100)
            .unwrap();
        db.rollback().unwrap();
        assert_eq!(
            db.execute(&format!("SELECT count(*) FROM {name}"), 100)
                .unwrap()
                .rows[0][0],
            "1"
        );
        let script=format!("BEGIN INSERT INTO {name}(id,label) VALUES (3,q'[a;b]'); END;\n/\nSELECT * FROM {name};");
        let stmts = sql::split(&script).unwrap();
        assert_eq!(stmts.len(), 2);
        for s in stmts {
            db.execute(&s.text, 100).unwrap();
        }
        db.rollback().unwrap();
        let owner = db.current_schema().unwrap();
        assert!(db.schemas().unwrap().contains(&owner));
        let objects = db.objects(&owner).unwrap();
        let object = objects.iter().find(|o| o.name == name).unwrap();
        for tab in 0..3 {
            assert!(!db
                .execute(&caucedb::db::inspect_sql(object, tab), 100)
                .unwrap()
                .rows
                .is_empty());
        }
        let r = db
            .execute("SELECT level n FROM dual CONNECT BY level <= 200", 10)
            .unwrap();
        assert_eq!(r.rows.len(), 10);
        assert!(r.truncated);
        let r = db
            .execute("SELECT CAST(NULL AS VARCHAR2(1)) n FROM dual", 10)
            .unwrap();
        assert_eq!(r.rows[0][0], "NULL");
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(checks));
    db.rollback().unwrap();
    db.execute(&format!("DROP TABLE {name} PURGE"), 100)
        .unwrap();
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

#[test]
#[ignore = "requires Docker Oracle and Instant Client; see README"]
fn worker_test_errors_cancel_disconnect_and_shutdown() {
    let worker = Worker::new();
    worker
        .send(Command::Connect(Box::new(profile()), true))
        .unwrap();
    let events = drain(&worker);
    no_errors(&events);
    assert!(events.iter().any(|e| matches!(e, Event::Tested(_))));
    worker
        .send(Command::Run("SELECT 1 FROM dual".into(), 10))
        .unwrap();
    assert!(drain(&worker).iter().any(|e| matches!(e, Event::Error(_))));
    worker
        .send(Command::Connect(Box::new(profile()), false))
        .unwrap();
    no_errors(&drain(&worker));
    worker
        .send(Command::Run(
            "SELECT 1 FROM dual; SELECT * FROM table_that_does_not_exist; SELECT 3 FROM dual;"
                .into(),
            10,
        ))
        .unwrap();
    let events = drain(&worker);
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, Event::Result(_)))
            .count(),
        1
    );
    assert!(events.iter().any(|e| matches!(e, Event::Error(_))));
    worker
        .send(Command::Run(
            "BEGIN DBMS_SESSION.SLEEP(10); END;\n/\nSELECT 999 FROM dual;".into(),
            10,
        ))
        .unwrap();
    std::thread::sleep(Duration::from_millis(500));
    worker.cancel();
    let events = drain(&worker);
    assert!(events.iter().any(|e| matches!(e, Event::Error(_))));
    assert!(!events.iter().any(|e| matches!(e, Event::Result(_))));
    worker
        .send(Command::Run("SELECT 1 FROM dual".into(), 10))
        .unwrap();
    no_errors(&drain(&worker));
    worker.send(Command::Disconnect).unwrap();
    no_errors(&drain(&worker));
    worker
        .send(Command::Run("SELECT 1 FROM dual".into(), 10))
        .unwrap();
    assert!(drain(&worker).iter().any(|e| matches!(e, Event::Error(_))));
}

#[test]
#[ignore = "requires Docker proxy setup from dev/proxy.sql"]
fn proxy_authentication() {
    let mut p = profile();
    p.proxy_enabled = true;
    p.proxy_username = "tui_proxy".into();
    p.proxy_password = p.password.clone();
    let mut db = OracleDb::connect(&p).unwrap();
    let r=db.execute("SELECT sys_context('USERENV','SESSION_USER'), sys_context('USERENV','PROXY_USER') FROM dual",10).unwrap();
    assert_eq!(r.rows[0], vec!["TUI_TEST", "TUI_PROXY"]);
}

#[test]
#[ignore = "requires Docker Oracle and Instant Client; see README"]
fn application_connects_and_executes_a_sql_document() {
    use caucedb::app::{App, ConnectionForm, Modal};
    use caucedb::config::Config;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let dir = tempfile::tempdir().unwrap();
    let mut app = App::new(Config::default(), dir.path().join("connections.toml"));
    app.modal = Some(Modal::Form(Box::new(ConnectionForm::new(profile()))));
    app.key(KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE));
    let wait = |app: &mut App| {
        let deadline = std::time::Instant::now() + Duration::from_secs(45);
        while app.busy {
            assert!(std::time::Instant::now() < deadline, "{}", app.status);
            std::thread::sleep(Duration::from_millis(20));
            app.poll();
        }
        assert!(!app.error, "{}", app.status);
    };
    wait(&mut app);
    assert!(app.modal.is_none());
    assert_eq!(app.schema, "TUI_TEST");
    assert!(app.connected.is_some());
    app.open_initial("examples/inspect.sql").unwrap();
    app.key(KeyEvent::new(KeyCode::F(6), KeyModifiers::NONE));
    wait(&mut app);
    assert_eq!(app.results.len(), 3);
    assert_eq!(app.results[0].rows[0][0], "TUI_TEST");
    app.key(KeyEvent::new(KeyCode::F(9), KeyModifiers::NONE));
    wait(&mut app);
    assert!(!app.pending);
    app.key(KeyEvent::new(KeyCode::F(3), KeyModifiers::NONE));
    wait(&mut app);
    assert!(app.schemas.contains(&"TUI_TEST".into()));
}
