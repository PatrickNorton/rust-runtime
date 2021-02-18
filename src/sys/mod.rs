use crate::function::{Function, NativeFunction};
use crate::sys::files::{chdir, getcwd, list_dir, mkdir};
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
        _ => unimplemented!("sys.{}", x),
    }
}
