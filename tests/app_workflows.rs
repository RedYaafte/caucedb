use caucedb::{
    app::{App, ConnectionForm, Document, Modal},
    config::{Config, Profile},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
fn key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.key(KeyEvent::new(code, modifiers));
}
#[test]
fn save_open_dirty_document_and_protect_exit() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("query.sql");
    let mut app = App::new(Config::default(), dir.path().join("connections.toml"));
    app.focus = 2;
    app.paste("SELECT '日本; México' FROM dual;".into());
    assert!(app.docs[0].dirty());
    key(&mut app, KeyCode::Char('s'), KeyModifiers::CONTROL);
    app.paste(path.to_str().unwrap().into());
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert!(!app.docs[0].dirty());
    assert!(path.exists());
    key(&mut app, KeyCode::Char('o'), KeyModifiers::CONTROL);
    app.paste(path.to_str().unwrap().into());
    key(&mut app, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(app.docs.len(), 2);
    assert_eq!(app.docs[0].text(), app.docs[1].text());
    app.paste("\n-- unsaved".into());
    key(&mut app, KeyCode::Char('q'), KeyModifiers::CONTROL);
    assert!(matches!(app.modal, Some(Modal::Confirm(_, _))));
    assert!(!app.quit);
    key(&mut app, KeyCode::Char('n'), KeyModifiers::NONE);
    assert!(app.modal.is_none());
    assert!(!app.quit);
}
#[test]
fn profile_form_saves_without_keyring_and_retains_validation_errors() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("connections.toml");
    let mut app = App::new(Config::default(), path.clone());
    app.modal = Some(Modal::Form(Box::new(ConnectionForm::new(
        Profile::default(),
    ))));
    key(&mut app, KeyCode::F(2), KeyModifiers::NONE);
    assert!(app.error);
    assert!(app.modal.is_some());
    assert!(matches!(&app.modal, Some(Modal::Form(f)) if f.action_attempted));
    assert!(!path.exists());
    if let Some(Modal::Form(f)) = &mut app.modal {
        f.fields[14].value = "scott".into();
        f.fields[15].value = "never-on-disk".into();
    }
    key(&mut app, KeyCode::F(2), KeyModifiers::NONE);
    assert!(!app.error, "{}", app.status);
    assert!(app.modal.is_none());
    assert!(app
        .status
        .contains("Saved connection 'New connection' · not connected"));
    let saved = std::fs::read_to_string(path).unwrap();
    assert!(!saved.contains("never-on-disk"));
    assert_eq!(app.config.connections.len(), 1);
}
#[test]
fn empty_and_trailing_newline_documents_are_not_dirty() {
    for text in ["", "\n", "select 1;\n", "a\n\n", "select 1;\r\n", "a\r\nb"] {
        assert!(!Document::new(text, None).dirty());
    }
}

#[test]
fn f1_opens_help_from_workbench_and_any_modal() {
    let mut app = App::new(Config::default(), "unused".into());

    key(&mut app, KeyCode::F(1), KeyModifiers::NONE);
    assert!(matches!(app.modal, Some(Modal::Help)));

    app.modal = Some(Modal::Form(Box::new(ConnectionForm::new(
        Profile::default(),
    ))));
    key(&mut app, KeyCode::F(1), KeyModifiers::NONE);
    assert!(matches!(app.modal, Some(Modal::Help)));

    app.modal = Some(Modal::Prompt(caucedb::app::PromptKind::Open, String::new()));
    key(&mut app, KeyCode::F(1), KeyModifiers::NONE);
    assert!(matches!(app.modal, Some(Modal::Help)));

    key(&mut app, KeyCode::Esc, KeyModifiers::NONE);
    assert!(app.modal.is_none());
}
