use crate::custom_types::bytes::LangBytes;
use crate::custom_types::exceptions::io_error;
use crate::custom_var::downcast_var;
use crate::function::{Function, NativeFunction};
use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};
use std::ffi::OsStr;
use std::io;
use std::path::MAIN_SEPARATOR;

use files::{chdir, getcwd, list_dir, mkdir};
use metadata::metadata;

mod files;
mod metadata;

// Numbers borrowed from https://filippo.io/linux-syscall-table/
pub fn sys_name(x: usize) -> &'static str {
    match x {
        79 => "getcwd",
        80 => "chdir",
        83 => "mkdir",
        _ => unimplemented!("syscall no. {}", x),
    }
}

pub fn get_value(x: &str) -> Variable {
    match x {
        "FILE_SEPARATOR" => MAIN_SEPARATOR.into(),
        _ => Function::Native(get_syscall(x)).into(),
    }
}

#[inline]
pub fn get_syscall(x: &str) -> NativeFunction {
    match x {
        "mkdir" => mkdir,
        "chdir" => chdir,
        "getcwd" => getcwd,
        "listdir" => list_dir,
        "metadata" => metadata,
        _ => unimplemented!("sys.{}", x),
    }
}

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
