#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub type PlatformSocket = windows::NamedPipeSocket;

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub type PlatformSocket = unix::UnixConnection;

#[cfg(all(not(unix), not(windows)))]
pub type PlatformSocket = ();
