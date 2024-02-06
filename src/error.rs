#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to get NSWindow")]
    FailedToGetNSWindow,
    #[error("failed to get NSObject class")]
    FailedToCheckWindowVisibility,
    #[error("failed to hide window")]
    FailedToHideWindow,
    #[error("failed to show window")]
    FailedToShowWindow,
    #[error("tauri err: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("rwLock: {0}")]
    RwLock(String),
    #[error("mutex: {0}")]
    Mutex(String),
    #[error("other: {0}")]
    Other(String),
}
