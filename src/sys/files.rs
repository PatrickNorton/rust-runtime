use crate::custom_types::bytes::LangBytes;
use crate::custom_types::exceptions::io_error;
use crate::custom_var::downcast_var;
use crate::first;
use crate::looping::{IterAttrs, IterOk, IterResult, NativeIterator};
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::cell::RefCell;
use std::env::{current_dir, set_current_dir};
use std::ffi::OsStr;
use std::fs::{create_dir, read_dir, ReadDir};
use std::io;
use std::rc::Rc;

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

pub fn list_dir(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    os_do_1(first(args), runtime, |x| read_dir(x).map(ListDir::new))
}

#[derive(Debug)]
struct ListDir {
    entry: RefCell<ReadDir>,
}

impl ListDir {
    pub fn new(entry: ReadDir) -> Rc<ListDir> {
        Rc::new(ListDir {
            entry: RefCell::new(entry),
        })
    }

    fn inner_next(&self, runtime: &mut Runtime) -> Result<Option<Variable>, ()> {
        match self.entry.borrow_mut().next() {
            Option::None => Result::Ok(Option::None),
            Option::Some(Result::Ok(_)) => todo!(),
            Option::Some(Result::Err(e)) => {
                os_err(e, runtime)?;
                unreachable!()
            }
        }
    }
}

impl IterAttrs for ListDir {
    fn next_fn(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let var = self.inner_next(runtime)?;
        runtime.return_1(var.into())
    }

    fn get_type() -> Type {
        unimplemented!()
    }
}

impl NativeIterator for ListDir {
    fn next(self: Rc<Self>, runtime: &mut Runtime) -> IterResult {
        self.inner_next(runtime).map(IterOk::Normal)
    }
}
