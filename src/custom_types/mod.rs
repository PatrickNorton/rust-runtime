macro_rules! custom_class {
    ($type_value:ty, $create_fn:ident, $str_name:tt) => {{
        lazy_static! {
            static ref TYPE: $crate::custom_types::types::CustomType<$type_value> =
                $crate::custom_types::types::CustomType::new(
                    $str_name.into(),
                    ::std::vec::Vec::new(),
                    $crate::function::Function::Native(<$type_value>::$create_fn),
                    ::std::collections::HashMap::new()
                );
        }
        Type::Custom(&*TYPE)
    }};
}

pub mod array;
pub mod bytes;
pub mod coroutine;
pub mod dict;
pub mod exceptions;
pub mod file;
pub mod lambda;
pub mod list;
pub mod range;
pub mod set;
pub mod slice;
pub mod types;
