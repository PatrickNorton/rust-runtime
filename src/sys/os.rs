#[cfg(windows)]
pub const fn os_name() -> &'static str {
    "windows"
}

#[cfg(unix)]
pub const fn os_name() -> &'static str {
    "posix"
}
