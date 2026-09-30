use caucedb::{
    app::App,
    config::{self, Config},
    ui, Result,
};
use crossterm::{
    event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyEventKind},
    execute,
};
use std::{io, time::Duration};

fn main() {
    if let Err(e) = run() {
        eprintln!("caucedb: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|s| s == "--help" || s == "-h") {
        println!("caucedb [file.sql]\n  --snapshot [form]  Render a 120x40 terminal snapshot without Oracle\n  --help             Show this help\n\nF1: keys · F2: connection · F5: execute · Ctrl+Q: exit\nConfiguration: CAUCEDB_CONFIG or the platform configuration directory\nOracle Net: TNS_ADMIN · Oracle client: LD_LIBRARY_PATH (Linux)");
        return Ok(());
    }
    if let Some(log_path) = std::env::var_os("CAUCEDB_LOG") {
        let mut options = std::fs::OpenOptions::new();
        options.create(true).append(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(log_path)?;
        let _ = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(std::sync::Mutex::new(file))
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
    }
    let path = config::config_path()?;
    let mut app = App::new(Config::load(&path)?, path);
    if args.first().is_some_and(|s| s == "--snapshot") {
        if args.get(1).is_some_and(|s| s == "form") {
            app.modal = Some(caucedb::app::Modal::Form(Box::new(
                caucedb::app::ConnectionForm::new(config::Profile::default()),
            )));
        }
        let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 40))?;
        terminal.draw(|f| ui::draw(f, &mut app))?;
        let buffer = terminal.backend().buffer();
        for y in 0..40 {
            let line: String = (0..120)
                .map(|x| buffer[(x, y)].symbol().to_owned())
                .collect();
            println!("{}", line.trim_end());
        }
        return Ok(());
    }
    if let Some(file) = args.first() {
        app.open_initial(file)?;
    }
    let mut terminal = ratatui::init();
    let result = (|| -> Result<()> {
        execute!(io::stdout(), EnableBracketedPaste)?;
        while !app.quit {
            app.poll();
            terminal.draw(|f| ui::draw(f, &mut app))?;
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(k) if k.kind != KeyEventKind::Release => app.key(k),
                    Event::Paste(s) => app.paste(s),
                    _ => {}
                }
            }
        }
        Ok(())
    })();
    let _ = execute!(io::stdout(), DisableBracketedPaste);
    ratatui::restore();
    result
}
