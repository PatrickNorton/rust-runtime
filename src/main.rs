extern crate num;
#[macro_use]
extern crate num_derive;

use crate::executor::execute;
use crate::file_info::FileInfo;
use crate::file_parsing::parse_file;
use crate::runtime::Runtime;
use std::rc::Rc;

mod base_fn;
mod builtins;
mod bytecode;
mod constant_loaders;
mod custom_types;
mod custom_var;
mod executor;
mod file_info;
mod file_parsing;
mod int_functions;
mod int_tools;
mod method;
mod operator;
mod quick_functions;
mod runtime;
mod stack_frame;
mod std_type;
mod std_variable;
mod string_functions;
mod string_var;
mod variable;

fn main() {
    let _args: Vec<String> = std::env::args().collect();
    let mut files: Vec<Rc<FileInfo>> = vec![];
    // let index = parse_file(args[1].clone(), &mut files);
    const FILE_NAME: &str =
        "/Users/patricknorton/Projects/Python files/__ncache__/HelloWorld.nbyte";
    let index = parse_file(FILE_NAME.to_string(), &mut files);
    let mut runtime = Runtime::new(files, index);
    let result = execute(&mut runtime);
    if let Result::Err(_) = result {
        panic!("Too many errors!")
    }
}
