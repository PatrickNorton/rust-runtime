use crate::method::InnerMethod;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::Name;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait CustomTypeImpl: Debug + Sync {}

#[derive(Debug)]
pub struct CustomType<T> {
    name: StringVar,
    supers: Vec<Type>,
    constructor: InnerMethod<T>,
    static_methods: HashMap<Name, InnerMethod<T>>,
}

impl<T> CustomType<T> {
    pub fn new(
        name: StringVar,
        supers: Vec<Type>,
        constructor: InnerMethod<T>,
        static_methods: HashMap<Name, InnerMethod<T>>,
    ) -> CustomType<T> {
        CustomType {
            name,
            supers,
            constructor,
            static_methods,
        }
    }
}

impl<T> PartialEq for CustomType<T> {
    fn eq(&self, other: &Self) -> bool {
        unimplemented!()
    }
}

impl<T> Eq for CustomType<T> {}

impl<T> CustomTypeImpl for CustomType<T> where T: Debug {}
