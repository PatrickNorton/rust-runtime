use crate::custom_types::list::List;
use crate::std_type::Type;
use crate::variable::Variable;
use std::env::args;

mod files;

pub fn get_value(x: &str) -> Variable {
    match x {
        _ => unimplemented!("sys.{}", x),
    }
}
