use crate::custom_types::types::CustomType;
use crate::custom_var::CustomVar;
use crate::function::Function;
use crate::method::StdMethod;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use std::collections::HashMap;
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

    pub fn get_op(self: &Rc<Self>, o: Operator) -> Variable {
        let func = match o {
            Operator::Str => Self::str,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    pub fn get_attribute(self: &Rc<Self>, name: StringVar) -> Variable {
        match name.as_str() {
            "message" => self.msg.clone().into(),
            _ => unimplemented!(),
        }
    }

    fn str(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.push(self.msg.clone().into());
        FnResult::Ok(())
    }
}

impl CustomVar for StdException {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Operator(o) => self.get_op(o),
            Name::Attribute(s) => self.get_attribute(s),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        self.exc_type
    }
}

macro_rules! create_exc {
    ($fn_name:ident, $type_name:tt) => {
        pub fn $fn_name() -> Type {
            fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
                debug_assert!(args.len() == 1);
                let msg = StringVar::from(args[0].clone());
                runtime.push(Rc::new(StdException::new(msg, $fn_name())).into());
                FnResult::Ok(())
            }
            lazy_static! {
                static ref TYPE: CustomType<StdException> = CustomType::new(
                    $type_name.into(),
                    Vec::new(),
                    Function::Native(create),
                    HashMap::new()
                );
            }
            Type::Custom(&*TYPE)
        }
    };
}

create_exc!(arithmetic_error, "ArithmeticError");
create_exc!(index_error, "IndexError");
create_exc!(key_error, "KeyError");
create_exc!(stop_iteration, "StopIteration");