use crate::function::Function;
use crate::method::InnerMethod;
use crate::name::Name;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::Variable;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait CustomTypeImpl: Debug + Sync {
    fn get_name(&self) -> &StringVar;

    fn create(&self, args: Vec<Variable>, runtime: &mut Runtime) -> Result<Variable, ()>;

    fn is_subclass(&self, other: &Type, runtime: &Runtime) -> bool;
}

#[derive(Debug)]
pub struct CustomType<T> {
    name: StringVar,
    supers: Vec<Type>,
    constructor: Function,
    static_methods: HashMap<Name, InnerMethod<T>>,
}

impl<T> CustomType<T> {
    pub fn new(
        name: StringVar,
        supers: Vec<Type>,
        constructor: Function,
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

impl<T> CustomTypeImpl for CustomType<T>
where
    T: Debug,
{
    fn get_name(&self) -> &StringVar {
        &self.name
    }

    fn create(&self, args: Vec<Variable>, runtime: &mut Runtime) -> Result<Variable, ()> {
        self.constructor.call((args, runtime))?;
        Result::Ok(runtime.pop_return())
    }

    fn is_subclass(&self, other: &Type, runtime: &Runtime) -> bool {
        for s in &self.supers {
            if s.is_subclass(other, runtime) {
                return true;
            }
        }
        false
    }
}
