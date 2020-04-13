use std::string::{String, ToString};

#[derive(Clone)]
pub struct Type {
    name: String,
}

impl Type {
    fn is_subclass(_other: Type) -> bool {
        false
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}
