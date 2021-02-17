use crate::function::{Function, NativeFunction};
use crate::sys::files::{chdir, getcwd, mkdir};
use crate::variable::Variable;
use std::path::MAIN_SEPARATOR;

mod files;

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
    let func = match x {
        "FILE_SEPARATOR" => return MAIN_SEPARATOR.into(),
        _ => get_syscall(x),
    };
    Function::Native(func).into()
}

#[inline]
pub fn get_syscall(x: &str) -> NativeFunction {
    match x {
        "mkdir" => mkdir,
        "chdir" => chdir,
        "getcwd" => getcwd,
        _ => unimplemented!("sys.{}", x),
    }
}
