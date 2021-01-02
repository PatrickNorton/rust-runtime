macro_rules! custom_class {
    ($type_value:ty, $create_fn:ident, $str_name:tt) => {{
        lazy_static! {
            static ref TYPE: $crate::custom_types::types::CustomType<$type_value> =
                $crate::custom_types::types::CustomType::new(
                    $str_name.into(),
                    ::std::vec::Vec::new(),
                    $crate::function::Function::Native(<$type_value>::$create_fn),
                    $crate::name_map::NameMap::new()
                );
        }
        Type::Custom(&*TYPE)
    }};
}

macro_rules! iter_internals {
    () => {
        iter_no_next!();

        fn next_fn(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            debug_assert!(args.is_empty());
            runtime.return_1(self.inner_next().into())
        }
    };
}

macro_rules! iter_no_next {
    () => {
        fn get_attribute(self: &Rc<Self>, val: &str) -> Variable {
            let func = match val {
                "next" => Self::next_fn,
                _ => unimplemented!("{}", val),
            };
            StdMethod::new_native(self.clone(), func).into()
        }

        fn get_op(self: &Rc<Self>, val: Operator) -> Variable {
            let func = match val {
                Operator::Iter => Self::ret_self,
                _ => unimplemented!("{}", val.name()),
            };
            StdMethod::new_native(self.clone(), func).into()
        }

        fn ret_self(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            debug_assert!(args.is_empty());
            runtime.return_1(self.clone().into())
        }
    };
}

pub mod array;
pub mod bytes;
pub mod coroutine;
pub mod dict;
pub mod enumerate;
pub mod exceptions;
pub mod file;
pub mod interfaces;
pub mod lambda;
pub mod list;
pub mod range;
pub mod set;
pub mod slice;
pub mod types;
