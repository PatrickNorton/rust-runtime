use crate::name::Name;
use crate::operator::Operator;
use std::collections::HashMap;
use std::ops::Index;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NameMap<T> {
    operators: HashMap<Operator, T>,
    values: HashMap<String, T>,
}

macro_rules! impl_method {
    ($name:ident -> $value:ty) => {
        pub fn $name(&self, value: Name) -> $value {
            match value {
                Name::Operator(o) => self.operators.$name(&o),
                Name::Attribute(a) => self.values.$name(a),
            }
        }
    };
}

impl<T> NameMap<T> {
    pub fn from_values(operators: HashMap<Operator, T>, values: HashMap<String, T>) -> NameMap<T> {
        NameMap { operators, values }
    }

    pub fn new() -> NameMap<T> {
        NameMap {
            operators: HashMap::new(),
            values: HashMap::new(),
        }
    }

    impl_method!(get -> Option<&T>);
    impl_method!(contains_key -> bool);

    pub fn get_mut(&mut self, value: Name) -> Option<&mut T> {
        match value {
            Name::Operator(o) => self.operators.get_mut(&o),
            Name::Attribute(a) => self.values.get_mut(a),
        }
    }

    pub fn insert(&mut self, name: Name, value: T) -> Option<T> {
        match name {
            Name::Attribute(a) => self.values.insert(a.to_owned(), value),
            Name::Operator(o) => self.operators.insert(o, value),
        }
    }
}

impl<T> Default for NameMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<Name<'_>> for NameMap<T> {
    type Output = T;

    fn index(&self, index: Name) -> &Self::Output {
        match index {
            Name::Attribute(a) => &self.values[a],
            Name::Operator(o) => &self.operators[&o],
        }
    }
}
