extern crate num;
#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate downcast_rs;

use crate::executor::execute;
use crate::file_info::FileInfo;
use crate::file_parsing::parse_file;
use crate::runtime::Runtime;
use std::convert::TryInto;

#[macro_use]
mod macros;

mod base_fn;
mod builtin_functions;
mod builtins;
mod bytecode;
mod character;
mod constant_loaders;
mod custom_types;
mod custom_var;
mod executor;
mod file_info;
mod file_parsing;
mod fmt;
mod from_bool;
mod function;
mod int_tools;
mod int_var;
mod jump_table;
mod lang_union;
mod looping;
mod method;
mod name;
mod name_map;
mod operator;
mod property;
mod quick_functions;
mod rational_var;
mod runtime;
mod stack_frame;
mod std_type;
mod std_variable;
mod string_var;
mod sys;
mod tuple;
mod var_impls;
mod variable;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut files: Vec<FileInfo> = Vec::new();
    let index = parse_file(args[1].clone(), &mut files);
    let mut runtime = Runtime::new(files, index);
    let result = execute(&mut runtime);
    if result.is_err() {
        panic!("Too many errors!")
    }
}

fn first<T>(args: Vec<T>) -> T {
    debug_assert!(
        !args.is_empty(),
        "Value passed to first must have at least 1 element"
    );
    args.into_iter().next().unwrap()
}

fn first_n<T, const N: usize>(args: Vec<T>) -> [T; N] {
    args.try_into()
        .unwrap_or_else(|x: Vec<T>| panic!("Value had length {}, expected length {}", x.len(), N))
}
