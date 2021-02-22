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
use once_cell::sync::Lazy;
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

    fn str(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.msg.clone().into())
    }

    fn msg(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.msg.clone().into())
    }
}

impl CustomVar for StdException {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        self.exc_type
    }

    fn get_operator(self: Rc<Self>, o: Operator) -> Variable {
        let func = match o {
            Operator::Str => Self::str,
            _ => unimplemented!("{}{}", self.exc_type.str(), o.name()),
        };
        StdMethod::new_native(self, func).into()
    }

    fn get_attribute(self: Rc<Self>, name: &str) -> Variable {
        match name {
            "message" => self.msg.clone().into(),
            "msg" => StdMethod::new_native(self, Self::msg).into(),
            _ => unimplemented!("{}{}", self.exc_type.str(), name),
        }
    }

    fn str(self: Rc<Self>, _runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(self.msg.clone())
    }
}

fn get_message(type_name: &str, args: Vec<Variable>, runtime: &mut Runtime) -> String {
    match args.len() {
        0 => format!("{}\n{}", type_name, runtime.frame_strings()),
        1 => format!(
            "{}:\n{}\n{}",
            type_name,
            StringVar::from(first(args)),
            runtime.frame_strings()
        ),
        x => panic!(
            "Expected 0 or 1 args, got {}\n{}",
            x,
            runtime.frame_strings()
        ),
    }
}

macro_rules! create_exc {
    ($fn_name:ident, $type_name:tt) => {
        pub fn $fn_name() -> Type {
            fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                let msg = get_message(stringify!($type_name), args, runtime).into();
                runtime.return_1(Rc::new(StdException::new(msg, $fn_name())).into())
            }
            static TYPE: Lazy<CustomType> = Lazy::new(|| {
                CustomType::new(
                    $type_name.into(),
                    Vec::new(),
                    Function::Native(create),
                    NameMap::new(),
                )
            });
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
create_exc!(value_error, "ValueError");
