extern crate num;
#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate downcast_rs;

use crate::executor::execute;
use crate::file_info::FileInfo;
use crate::file_parsing::parse_file;
use crate::runtime::Runtime;

mod base_fn;
mod builtin_functions;
mod builtins;
mod bytecode;
mod constant_loaders;
mod custom_types;
mod custom_var;
mod executor;
mod file_info;
mod file_parsing;
mod function;
mod int_tools;
mod int_var;
mod method;
mod operator;
mod quick_functions;
mod rational_var;
mod runtime;
mod stack_frame;
mod std_type;
mod std_variable;
mod string_var;
mod variable;

#[macro_use]
mod macros;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut files: Vec<FileInfo> = Vec::new();
    let index = parse_file(args[1].clone(), &mut files);
    let mut runtime = Runtime::new(files, index);
    let result = execute(&mut runtime);
    if let Result::Err(_) = result {
        panic!("Too many errors!")
    }
}
