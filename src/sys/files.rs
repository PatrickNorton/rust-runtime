use crate::first;
use crate::looping::{IterAttrs, IterResult, NativeIterator};
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::sys::{os_do, os_do_1, os_err};
use crate::variable::{FnResult, Variable};
use std::cell::RefCell;
use std::env::{current_dir, set_current_dir};
use std::ffi::OsStr;
use std::fs::{create_dir, read_dir, ReadDir};
use std::io;
use std::rc::Rc;

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
        self.inner_next(runtime).map(From::from)
    }
}
