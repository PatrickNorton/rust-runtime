use crate::runtime::Runtime;
use crate::std_variable::StdVarMethod;
use crate::variable::{Name, Variable};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::string::{String, ToString};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Standard(&'static StdType),
    Null(),
    Bool(),
    Bigint(),
    String(),
    Decimal(),
    Type(),
}

pub struct StdType {
    name: String,
    file_no: usize,
    supers: Vec<Type>,
    methods: HashMap<Name, StdVarMethod>,
    static_methods: HashMap<Name, StdVarMethod>,
}

impl Type {
    pub fn new_std(
        name: String,
        file_no: usize,
        methods: HashMap<Name, StdVarMethod>,
        static_methods: HashMap<Name, StdVarMethod>,
    ) -> Type {
        let t = Box::new(StdType::new(name, file_no, methods, static_methods));
        Type::Standard(Box::leak(t)) // Classes live forever, why worry about cleanup?
    }

    pub fn is_subclass(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Standard(t), _) => t.is_subclass(other),
            (Type::Null(), Type::Null()) => true,
            (Type::Bool(), Type::Bool()) => true,
            (Type::Bool(), Type::Bigint()) => true,
            (Type::Bigint(), Type::Bigint()) => true,
            (Type::String(), Type::String()) => true,
            (Type::Decimal(), Type::Decimal()) => true,
            (Type::Type(), Type::Type()) => true,
            _ => false,
        }
    }

    pub fn is_type_of(&self, var: &Variable) -> bool {
        var.get_type().is_subclass(self)
    }

    pub fn create_inst(&self, args: &Vec<Variable>, runtime: &mut Runtime) -> Variable {
        return match self {
            Type::Standard(_) => todo!(),
            Type::Null() => Variable::Null(),
            Type::Bool() => Variable::Bool(args[0].to_bool(runtime)),
            Type::Bigint() => Variable::Bigint(args[0].int(runtime)),
            Type::String() => Variable::String(args[0].str(runtime)),
            Type::Decimal() => unimplemented!(),
            Type::Type() => Variable::Type(args[0].get_type()),
        };
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        return match self {
            Type::Standard(t) => t.name().clone(),
            Type::Null() => "null".to_string(),
            Type::Bool() => "bool".to_string(),
            Type::Bigint() => "int".to_string(),
            Type::String() => "str".to_string(),
            Type::Decimal() => "dec".to_string(),
            Type::Type() => "type".to_string(),
        };
    }
}

impl PartialEq for StdType {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.supers == other.supers
    }
}

impl Eq for StdType {}

impl Hash for StdType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.supers.hash(state);
    }
}

impl StdType {
    pub fn new(
        name: String,
        file_no: usize,
        methods: HashMap<Name, StdVarMethod>,
        static_methods: HashMap<Name, StdVarMethod>,
    ) -> StdType {
        StdType {
            name,
            file_no,
            supers: vec![],
            methods,
            static_methods,
        }
    }

    fn is_subclass(&self, other: &Type) -> bool {
        if let Type::Standard(o) = other {
            if self == *o {
                return true;
            }
        }
        for sup in &self.supers {
            if sup.is_subclass(other) {
                return true;
            }
        }
        false
    }

    fn name(&self) -> &String {
        &self.name
    }
}
