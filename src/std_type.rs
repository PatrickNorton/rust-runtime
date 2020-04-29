use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::string::{String, ToString};

use crate::builtins::default_methods;
use crate::custom_types::types::CustomTypeImpl;
use crate::method::{InnerMethod, Method, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_variable::{StdVarMethod, StdVariable};
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use std::ptr;

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Standard(&'static StdType),
    Null,
    Bool,
    Bigint,
    String,
    Decimal,
    Type,
    Custom(&'static dyn CustomTypeImpl),
}

#[derive(Debug)]
pub struct StdType {
    name: StringVar,
    file_no: usize,
    supers: Vec<Type>,
    methods: HashMap<Name, StdVarMethod>,
    static_methods: HashMap<Name, StdVarMethod>,
}

impl Type {
    pub fn new_std(
        name: StringVar,
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
            (Type::Null, Type::Null) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Bool, Type::Bigint) => true,
            (Type::Bigint, Type::Bigint) => true,
            (Type::String, Type::String) => true,
            (Type::Decimal, Type::Decimal) => true,
            (Type::Type, Type::Type) => true,
            (Type::Custom(t), _) => t.is_subclass(other),
            _ => false,
        }
    }

    pub fn is_type_of(&self, var: &Variable) -> bool {
        var.get_type().is_subclass(self)
    }

    pub fn create_inst(&self, args: Vec<Variable>, runtime: &mut Runtime) -> Result<Variable, ()> {
        Result::Ok(match self {
            Type::Standard(std_t) => std_t.create(args, runtime)?,
            Type::Null => Variable::Null(),
            Type::Bool => Variable::Bool(args[0].to_bool(runtime)?),
            Type::Bigint => Variable::Bigint(args[0].int(runtime)?),
            Type::String => Variable::String(args[0].str(runtime)?),
            Type::Decimal => unimplemented!(),
            Type::Type => Variable::Type(args[0].get_type()),
            Type::Custom(t) => t.create(args, runtime)?,
        })
    }

    pub fn push_create(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        let runtime = args.1;
        let new = self.create_inst(args.0, runtime)?;
        runtime.push(new);
        FnResult::Ok(())
    }

    pub fn index(&self, index: Name) -> Variable {
        return match self {
            Type::Standard(std_t) => {
                let index_pair = std_t.index(&index);
                let inner_m = InnerMethod::Standard(index_pair.0, index_pair.1);
                let n = StdMethod::new(self.clone(), inner_m);
                Variable::Method(Box::new(n))
            }
            _ => unimplemented!(),
        };
    }

    pub fn str(&self) -> StringVar {
        return match self {
            Type::Standard(t) => t.name().clone(),
            Type::Null => "null".into(),
            Type::Bool => "bool".into(),
            Type::Bigint => "int".into(),
            Type::String => "str".into(),
            Type::Decimal => "dec".into(),
            Type::Type => "type".into(),
            Type::Custom(t) => t.get_name().clone(),
        };
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        return match self {
            Type::Standard(t) => t.name().to_string(),
            Type::Null => "null".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Bigint => "int".to_string(),
            Type::String => "str".to_string(),
            Type::Decimal => "dec".to_string(),
            Type::Type => "type".to_string(),
            Type::Custom(t) => t.get_name().to_string(),
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
    pub const fn new(
        name: StringVar,
        file_no: usize,
        methods: HashMap<Name, StdVarMethod>,
        static_methods: HashMap<Name, StdVarMethod>,
    ) -> StdType {
        StdType {
            name,
            file_no,
            supers: Vec::new(),
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

    fn name(&self) -> &StringVar {
        &self.name
    }

    fn index(&self, name: &Name) -> (usize, u32) {
        if let StdVarMethod::Standard(f, a) = self.static_methods[name] {
            (f, a)
        } else {
            panic!();
        }
    }

    fn create(&'static self, args: Vec<Variable>, runtime: &mut Runtime) -> Result<Variable, ()> {
        let instance = StdVariable::new(self, HashMap::new());
        let method = self.methods.get(&Name::Operator(Operator::New)).unwrap();
        StdMethod::new(instance.clone(), method.clone()).call((args, runtime))?;
        return Result::Ok(Variable::Standard(instance));
    }

    pub(crate) fn get_method(&self, name: Name) -> StdVarMethod {
        match self.methods.get(&name) {
            Option::Some(t) => t.clone(),
            Option::None => default_methods(name),
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Standard(a), Type::Standard(b)) => ptr::eq(a, b),
            (Type::Null, Type::Null) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Bigint, Type::Bigint) => true,
            (Type::String, Type::String) => true,
            (Type::Decimal, Type::Decimal) => true,
            (Type::Type, Type::Type) => true,
            (Type::Custom(a), Type::Custom(b)) => ptr::eq(a, b),
            _ => false,
        }
    }
}

impl Eq for Type {}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Type::Standard(a) => a.hash(state),
            Type::Null => 0.hash(state),
            Type::Bool => 1.hash(state),
            Type::Bigint => 2.hash(state),
            Type::String => 3.hash(state),
            Type::Decimal => 4.hash(state),
            Type::Type => 5.hash(state),
            Type::Custom(b) => ptr::hash(b, state),
        }
    }
}
