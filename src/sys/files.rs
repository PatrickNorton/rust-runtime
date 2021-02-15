use crate::custom_types::bytes::LangBytes;
use crate::custom_types::exceptions::io_error;
use crate::custom_var::downcast_var;
use crate::first;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::env::{current_dir, set_current_dir};
use std::ffi::OsStr;
use std::fs::create_dir;
use std::io::{self};

#[cfg(unix)]
#[allow(clippy::unnecessary_wraps)]
fn create_os_str(bytes: &[u8]) -> Option<&OsStr> {
    use std::os::unix::ffi::OsStrExt;
    Option::Some(OsStr::from_bytes(bytes))
}

#[cfg(windows)]
fn create_os_str(bytes: &[u8]) -> Option<&OsStr> {
    std::str::from_utf8(bytes).ok().map(OsStr::new)
}

fn filename_err(runtime: &mut Runtime) -> FnResult {
    runtime.throw_quick(
        io_error(),
        "Invalid UTF-8 in filenames is not yet supported on Windows",
    )
}

fn os_err(err: io::Error, runtime: &mut Runtime) -> FnResult {
    runtime.throw_quick(io_error(), format!("{}", err))
}

fn os_do(
    arg: Variable,
    runtime: &mut Runtime,
    func: impl FnOnce(&OsStr) -> io::Result<()>,
) -> FnResult {
    let arg = downcast_var::<LangBytes>(arg).unwrap();
    let value = arg.get_value();
    match create_os_str(&value) {
        Option::Some(s) => match func(s) {
            Result::Ok(_) => runtime.return_0(),
            Result::Err(e) => os_err(e, runtime),
        },
        Option::None => filename_err(runtime),
    }
}

fn os_do_1<T: Into<Variable>>(
    arg: Variable,
    runtime: &mut Runtime,
    func: impl FnOnce(&OsStr) -> io::Result<T>,
) -> FnResult {
    let arg = downcast_var::<LangBytes>(arg).unwrap();
    let value = arg.get_value();
    match create_os_str(&value) {
        Option::Some(s) => match func(s) {
            Result::Ok(x) => runtime.return_1(x.into()),
            Result::Err(e) => os_err(e, runtime),
        },
        Option::None => filename_err(runtime),
    }
}

// Hack b/c of Rust's weird 'one type is more general than the other' error
// TODO: Report this error
macro_rules! wrap_fn {
    ($func:ident -> $cls:ty) => {{
        fn wrapper(arg: &OsStr) -> io::Result<$cls> {
            $func(arg)
        }
        wrapper
    }};
}

pub fn mkdir(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    os_do(first(args), runtime, wrap_fn!(create_dir -> ()))
}

pub fn chdir(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    os_do(first(args), runtime, wrap_fn!(set_current_dir -> ()))
}

pub fn getcwd(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    match current_dir() {
        Result::Ok(p) => runtime.return_1(StringVar::from(p.to_str().unwrap().to_owned()).into()),
        Result::Err(e) => os_err(e, runtime),
    }
}
