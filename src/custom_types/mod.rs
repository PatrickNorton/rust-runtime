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
