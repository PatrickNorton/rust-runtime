extern crate num;
#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate downcast_rs;

use crate::executor::execute;
use crate::file_info::FileInfo;
use crate::file_parsing::parse_file;
use crate::runtime::Runtime;

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

// Once #[feature(box_patterns)] is stabilized, we can remove the iterator
// let box [a, ..] = args.into_boxed_slice();
// How I wish there were a better way to do this

fn first<T>(args: Vec<T>) -> T {
    debug_assert!(
        !args.is_empty(),
        "Value passed to first must have at least 1 element"
    );
    args.into_iter().next().unwrap()
}

fn first_two<T>(args: Vec<T>) -> (T, T) {
    debug_assert!(
        args.len() >= 2,
        "Value passed to first_two must have at least 2 elements"
    );
    let mut iter = args.into_iter();
    (iter.next().unwrap(), iter.next().unwrap())
}

fn first_three<T>(args: Vec<T>) -> (T, T, T) {
    debug_assert!(
        args.len() >= 3,
        "Value passed to first_three must have at least 3 elements"
    );
    let mut iter = args.into_iter();
    (
        iter.next().unwrap(),
        iter.next().unwrap(),
        iter.next().unwrap(),
    )
}
