pub mod app;
pub mod config;
pub mod db;
pub mod sql;
pub mod ui;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Oracle(#[from] oracle::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
