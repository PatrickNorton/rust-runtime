use crate::string_var::StringVar;
use crate::variable::Variable;
macro_rules! custom_class {
    ($type_value:ty, $create_fn:ident, $str_name:tt) => {{
        custom_class!($type_value, $create_fn, $str_name,)
    }};

    ($type_value:ty, $create_fn:ident, $str_name:tt, $($name:expr => $value:ident),* $(,)?) => {{
        lazy_static! {
            static ref TYPE: $crate::custom_types::types::CustomType =
                $crate::custom_types::types::CustomType::new(
                    $str_name.into(),
                    ::std::vec::Vec::new(),
                    $crate::function::Function::Native(<$type_value>::$create_fn),
                    name_map!(
                        $(
                            $name.into() => $crate::function::Function::Native(<$type_value>::$value),
                        ),*
                    )
                );
        }
        Type::Custom(&*TYPE)
    }};
}

macro_rules! iter_internals {
    () => {
        iter_no_next!();

        fn next_fn(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            debug_assert!(args.is_empty());
            runtime.return_1(self.inner_next().into())
        }
    };
}

macro_rules! iter_no_next {
    () => {
        fn get_attribute(self: Rc<Self>, val: &str) -> Variable {
            let func = match val {
                "next" => Self::next_fn,
                _ => unimplemented!("{}", val),
            };
            StdMethod::new_native(self, func).into()
        }

        fn get_op(self: Rc<Self>, val: Operator) -> Variable {
            let func = match val {
                Operator::Iter => Self::ret_self,
                _ => unimplemented!("{}", val.name()),
            };
            StdMethod::new_native(self, func).into()
        }

        fn ret_self(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
            debug_assert!(args.is_empty());
            runtime.return_1(self.into())
        }
    };
}

macro_rules! default_attr {
    ($self:expr, $name:expr) => {
        match $name {
            Name::Operator(o) => $self.get_op(o),
            Name::Attribute(s) => $self.get_attribute(s),
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

fn join_values(
    values: &[Variable],
    mut func: impl FnMut(Variable) -> Result<StringVar, ()>,
) -> Result<String, ()> {
    let mut result = String::new();
    for (i, value) in values.iter().enumerate() {
        result += func(value.clone())?.as_str();
        if i != values.len() - 1 {
            result += ", ";
        }
    }
    Result::Ok(result)
}
