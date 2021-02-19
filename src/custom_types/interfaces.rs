use crate::custom_var::CustomVar;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::variable::{FnResult, Variable};
use std::rc::Rc;

macro_rules! std_interface {
    ($name:ident, $str_name:tt) => {
        #[derive(Debug, Copy, Clone)]
        pub struct $name {}

        impl $name {
            pub fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
                panic!("Created interface {}", $str_name)
            }

            pub fn cls() -> Type {
                custom_class!($name, create, $str_name)
            }
        }

        impl CustomVar for $name {
            fn set(self: Rc<Self>, _name: Name, _object: Variable) {
                unimplemented!()
            }

            fn get_type(&self) -> Type {
                Self::cls()
            }

            fn get_operator(self: Rc<Self>, _op: Operator) -> Variable {
                unimplemented!()
            }

            fn get_attribute(self: Rc<Self>, _name: &str) -> Variable {
                unimplemented!()
            }
        }
    };
}

std_interface!(Callable, "Callable");
std_interface!(Iterable, "Iterable");
std_interface!(Throwable, "Throwable");
