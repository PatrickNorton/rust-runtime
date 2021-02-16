use crate::function::Function;
use crate::sys::files::{chdir, getcwd, mkdir};
use crate::variable::Variable;

mod files;

pub fn get_value(x: &str) -> Variable {
    let func = match x {
        "mkdir" => mkdir,
        "chdir" => chdir,
        "getcwd" => getcwd,
        _ => unimplemented!("sys.{}", x),
    };
    Function::Native(func).into()
}
