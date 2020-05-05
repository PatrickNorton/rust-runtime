macro_rules! custom_class {
    ($type_value:ident, $create_fn:ident, $str_name:tt) => {{
        lazy_static! {
            static ref TYPE: CustomType<$type_value> = CustomType::new(
                $str_name.into(),
                Vec::new(),
                Function::Native($type_value::$create_fn),
                HashMap::new()
            );
        }
        Type::Custom(&*TYPE)
    }};
}

pub mod dict;
pub mod exceptions;
pub mod list;
pub mod range;
pub mod set;
pub mod types;
