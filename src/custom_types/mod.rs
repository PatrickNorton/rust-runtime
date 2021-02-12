use crate::string_var::{MaybeString, StringVar};
use crate::variable::Variable;
use ascii::AsciiStr;
use once_cell::sync::Lazy;
macro_rules! custom_class {
    ($type_value:ty, $create_fn:ident, $str_name:tt) => {{
        custom_class!($type_value, $create_fn, $str_name,)
    }};

    ($type_value:ty, $create_fn:ident, $str_name:tt, $($name:expr => $value:ident),* $(,)?) => {{
        use ::once_cell::sync::Lazy;
        use $crate::custom_types::types::CustomType;
        use $crate::function::Function;
        static TYPE: Lazy<CustomType> = Lazy::new(
            || CustomType::new(
                $str_name.into(),
                ::std::vec::Vec::new(),
                Function::Native(<$type_value>::$create_fn),
                name_map!(
                    $(
                        $name.into() => Function::Native(<$type_value>::$value),
                    ),*
                )
            )
        );
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

/// A static reference to prevent multiple-checking of ascii
static ASCII_COMMA: Lazy<&AsciiStr> = Lazy::new(|| AsciiStr::from_ascii(", ").unwrap());

pub fn join_values(
    values: &[Variable],
    mut func: impl FnMut(Variable) -> Result<StringVar, ()>,
) -> Result<MaybeString, ()> {
    let mut result = MaybeString::new();
    for (i, value) in values.iter().enumerate() {
        result += &func(value.clone())?;
        if i != values.len() - 1 {
            result += *ASCII_COMMA;
        }
    }
    Result::Ok(result)
}
