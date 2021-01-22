use crate::custom_types::types::CustomType;
use crate::custom_var::CustomVar;
use crate::first;
use crate::function::Function;
use crate::method::StdMethod;
use crate::name::Name;
use crate::name_map::NameMap;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::rc::Rc;

#[derive(Debug)]
struct StdException {
    msg: StringVar,
    exc_type: Type,
}

impl StdException {
    pub fn new(msg: StringVar, exc_type: Type) -> StdException {
        StdException { msg, exc_type }
    }

    pub fn get_op(self: Rc<Self>, o: Operator) -> Variable {
        let func = match o {
            Operator::Str => Self::str,
            _ => unimplemented!(),
        };
        StdMethod::new_native(self, func).into()
    }

    pub fn get_attribute(self: Rc<Self>, name: &str) -> Variable {
        match name {
            "message" => self.msg.clone().into(),
            _ => unimplemented!(),
        }
    }

    fn str(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.msg.clone().into())
    }
}

impl CustomVar for StdException {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        default_attr!(self, name)
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        self.exc_type
    }

    fn str(self: Rc<Self>, _runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(self.msg.clone())
    }
}

macro_rules! create_exc {
    ($fn_name:ident, $type_name:tt) => {
        pub fn $fn_name() -> Type {
            fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                let msg = match args.len() {
                    0 => format!("{}\n{}", stringify!($type_name), runtime.frame_strings()),
                    1 => format!(
                        "{}:\n{}\n{}",
                        stringify!($type_name),
                        StringVar::from(first(args)),
                        runtime.frame_strings()
                    ),
                    x => panic!(
                        "Expected 0 or 1 args, got {}\n{}",
                        x,
                        runtime.frame_strings()
                    ),
                }
                .into();
                runtime.return_1(Rc::new(StdException::new(msg, $fn_name())).into())
            }
            lazy_static! {
                static ref TYPE: CustomType = CustomType::new(
                    $type_name.into(),
                    Vec::new(),
                    Function::Native(create),
                    NameMap::new()
                );
            }
            Type::Custom(&*TYPE)
        }
    };
}

create_exc!(arithmetic_error, "ArithmeticError");
create_exc!(assertion_error, "AssertionError");
create_exc!(index_error, "IndexError");
create_exc!(invalid_state, "InvalidState");
create_exc!(io_error, "IOError");
create_exc!(key_error, "KeyError");
create_exc!(not_implemented, "NotImplemented");
create_exc!(null_error, "NullError");
// create_exc!(stop_iteration, "StopIteration");
create_exc!(value_error, "ValueError");
