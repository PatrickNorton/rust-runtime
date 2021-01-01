use crate::custom_var::CustomVar;
use crate::name::Name;
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
            fn get_attr(self: Rc<Self>, _name: Name) -> Variable {
                unimplemented!()
            }

            fn set(self: Rc<Self>, _name: Name, _object: Variable) {
                unimplemented!()
            }

            fn get_type(&self) -> Type {
                Self::cls()
            }
        }
    };
}

std_interface!(Callable, "Callable");
std_interface!(Throwable, "Throwable");
