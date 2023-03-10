#[derive(Debug)]
pub enum Error {
    FailedToLockMutex,
    FailedToGetExecutablePath,
    FailedToRegisterShortcut,
    FailedToGetNSWindow,
    FailedToGetNSWorkspaceClass,
    FailedToCheckWindowVisibility,
    FailedToHideWindow,
    FailedToShowWindow,
}
