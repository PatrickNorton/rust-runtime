use crate::custom_var::CustomVar;
use crate::function::Function;
use crate::method::InnerMethod;
use crate::name::Name;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use downcast_rs::__alloc::rc::Rc;
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

#[derive(Debug, Copy, Clone)]
pub struct TypeIdentity {
    value: Type,
}

impl TypeIdentity {
    pub fn new(value: Type) -> Rc<TypeIdentity> {
        Rc::new(TypeIdentity { value })
    }
}

impl CustomVar for TypeIdentity {
    fn get_attr(self: Rc<Self>, _name: Name) -> Variable {
        unimplemented!()
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        unimplemented!()
    }

    fn call(self: Rc<Self>, _args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        runtime.return_1(self.value.into())
    }

    fn call_or_goto(self: Rc<Self>, _args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        runtime.return_1(self.value.into())
    }
}
