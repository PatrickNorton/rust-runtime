use crate::custom_var::CustomVar;
use crate::function::Function;
use crate::name::Name;
use crate::name_map::NameMap;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug)]
pub struct CustomType {
    name: StringVar,
    supers: Vec<Type>,
    constructor: Function,
    static_methods: NameMap<Function>,
}

impl CustomType {
    pub fn new(
        name: StringVar,
        supers: Vec<Type>,
        constructor: Function,
        static_methods: NameMap<Function>,
    ) -> CustomType {
        CustomType {
            name,
            supers,
            constructor,
            static_methods,
        }
    }
}

impl CustomType {
    pub fn get_name(&self) -> &StringVar {
        &self.name
    }

    pub fn create(&self, args: Vec<Variable>, runtime: &mut Runtime) -> Result<Variable, ()> {
        self.constructor.call((args, runtime))?;
        Result::Ok(runtime.pop_return())
    }

    pub fn is_subclass(&self, other: &Type, runtime: &Runtime) -> bool {
        for s in &self.supers {
            if s.is_subclass(other, runtime) {
                return true;
            }
        }
        false
    }

    pub fn index(&self, name: Name) -> Variable {
        self.static_methods
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("{}.{} does not exist", &self.name, name.as_str()))
            .into()
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
