use crate::builtin_functions::string_fn;
use crate::builtins::default_methods;
use crate::custom_types::exceptions::value_error;
use crate::custom_types::types::{CustomTypeImpl, TypeIdentity};
use crate::lang_union::{UnionMethod, UnionType};
use crate::method::{InnerMethod, Method, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::property::Property;
use crate::runtime::Runtime;
use crate::std_variable::{StdVarMethod, StdVariable};
use crate::string_var::StringVar;
use crate::tuple::LangTuple;
use crate::variable::{FnResult, Variable};
use num::ToPrimitive;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::mem::take;
use std::ptr;
use std::string::{String, ToString};

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Standard(&'static StdType),
    Null,
    Bool,
    Bigint,
    String,
    Decimal,
    Char,
    Tuple,
    Type,
    Object,
    Custom(&'static dyn CustomTypeImpl),
    Union(&'static UnionType),
    Option(usize, OptionType),
}

#[derive(Debug, Clone, Copy)]
pub enum OptionType {
    Standard(&'static StdType),
    Null,
    Bool,
    Bigint,
    String,
    Decimal,
    Char,
    Tuple,
    Type,
    Object,
    Custom(&'static dyn CustomTypeImpl),
    Union(&'static UnionType),
}

#[derive(Debug)]
pub struct StdType {
    name: StringVar,
    file_no: usize,
    supers: Vec<u32>,
    variables: HashSet<StringVar>,
    methods: HashMap<Name, StdVarMethod>,
    static_methods: HashMap<Name, StdVarMethod>,
    properties: HashMap<StringVar, Property>,
}

impl Type {
    pub fn new_std(
        name: StringVar,
        file_no: usize,
        supers: Vec<u32>,
        variables: HashSet<StringVar>,
        methods: HashMap<Name, StdVarMethod>,
        static_methods: HashMap<Name, StdVarMethod>,
        properties: HashMap<StringVar, Property>,
    ) -> Type {
        let t = Box::new(StdType::new(
            name,
            file_no,
            supers,
            variables,
            methods,
            static_methods,
            properties,
        ));
        Type::Standard(Box::leak(t)) // Classes live forever, why worry about cleanup?
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_union(
        name: StringVar,
        file_no: usize,
        supers: Vec<u32>,
        variants: Vec<StringVar>,
        variables: HashSet<StringVar>,
        methods: HashMap<Name, UnionMethod>,
        static_methods: HashMap<Name, UnionMethod>,
        properties: HashMap<StringVar, Property>,
    ) -> Type {
        let t = Box::new(UnionType::new(
            name,
            file_no,
            supers,
            variants,
            variables,
            methods,
            static_methods,
            properties,
        ));
        Type::Union(Box::leak(t)) // Classes live forever, why worry about cleanup?
    }

    pub fn is_subclass(&self, other: &Type, runtime: &Runtime) -> bool {
        match (self, other) {
            (Type::Standard(t), _) => t.is_subclass(other, runtime),
            (Type::Null, Type::Null) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Bool, Type::Bigint) => true,
            (Type::Bigint, Type::Bigint) => true,
            (Type::String, Type::String) => true,
            (Type::Char, Type::Char) => true,
            (Type::Decimal, Type::Decimal) => true,
            (Type::Tuple, Type::Tuple) => true,
            (Type::Type, Type::Type) => true,
            (Type::Object, _) => true,
            (Type::Custom(t), _) => t.is_subclass(other, runtime),
            (Type::Union(t), Type::Union(u)) => ptr::eq(*t, *u),
            _ => false,
        }
    }

    pub fn is_type_of(&self, var: &Variable, runtime: &Runtime) -> bool {
        var.get_type().is_subclass(self, runtime)
    }

    pub fn create_inst(
        &self,
        mut args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> Result<Variable, ()> {
        Result::Ok(match self {
            Type::Standard(std_t) => std_t.create(args, runtime)?,
            Type::Null => Variable::default(),
            Type::Bool => take(&mut args[0]).into_bool(runtime)?.into(),
            Type::Bigint => take(&mut args[0]).int(runtime)?.into(),
            Type::String => take(&mut args[0]).str(runtime)?.into(),
            Type::Char => create_char(take(&mut args[0]), runtime)?,
            Type::Decimal => unimplemented!(),
            Type::Tuple => LangTuple::new(args.into()).into(),
            Type::Type => args[0].get_type().into(),
            Type::Object => unimplemented!(),
            Type::Custom(t) => t.create(args, runtime)?,
            Type::Union(_) => unimplemented!(),
            Type::Option(_, _) => unimplemented!(),
        })
    }

    pub fn push_create(&self, args: (Vec<Variable>, &mut Runtime)) -> FnResult {
        let runtime = args.1;
        let new = self.create_inst(args.0, runtime)?;
        runtime.return_1(new)
    }

    pub fn index(self, index: Name, runtime: &mut Runtime) -> Variable {
        match self {
            Type::Standard(std_t) => match std_t.index_method(&index) {
                Option::Some(index_pair) => {
                    let inner_m = InnerMethod::Standard(index_pair.0, index_pair.1);
                    let n = StdMethod::new(self, inner_m);
                    Box::new(n).into()
                }
                Option::None => {
                    if index == Name::Operator(Operator::GetAttr) {
                        // FIXME: This is used for type generification, but enum indexing won't work
                        TypeIdentity::new(self).into()
                    } else {
                        runtime.static_attr(&self, &index).unwrap_or_else(|| {
                            panic!(
                                "{}.{} not found\n{}",
                                self.str(),
                                index.as_str(),
                                runtime.stack_frames()
                            )
                        })
                    }
                }
            },
            Type::Union(union_t) => union_t.index(index),
            Type::String => match index {
                Name::Attribute(s) => string_fn::static_attr(s),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    }

    pub fn str(&self) -> StringVar {
        match self {
            Type::Standard(t) => t.name().clone(),
            Type::Null => "null".into(),
            Type::Bool => "bool".into(),
            Type::Bigint => "int".into(),
            Type::String => "str".into(),
            Type::Decimal => "dec".into(),
            Type::Char => "char".into(),
            Type::Tuple => "tuple".into(),
            Type::Type => "type".into(),
            Type::Object => "object".into(),
            Type::Custom(t) => t.get_name().clone(),
            Type::Union(u) => u.name().clone(),
            Type::Option(i, t) => format!("{}{}", Type::from(*t).str(), "?".repeat(*i)).into(),
        }
    }

    pub fn set(&self, index: StringVar, value: Variable, runtime: &mut Runtime) {
        match self {
            Type::Standard(_) | Type::Custom(_) => {
                runtime.set_static_attr(self, Name::Attribute(index), value)
            }
            _ => unimplemented!(),
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Type::Standard(t) => *t as *const _ as usize,
            Type::Custom(t) => *t as *const _ as *const () as usize,
            Type::Union(u) => *u as *const _ as usize,
            _ => todo!("Unique ids for special types"),
        }
    }

    pub fn make_option(self) -> Self {
        self.make_option_n(1)
    }

    pub fn make_option_n(self, n: usize) -> Self {
        Type::Option(
            n,
            match self {
                Type::Standard(s) => OptionType::Standard(s),
                Type::Null => OptionType::Null,
                Type::Bool => OptionType::Bool,
                Type::Bigint => OptionType::Bigint,
                Type::String => OptionType::String,
                Type::Decimal => OptionType::Decimal,
                Type::Char => OptionType::Char,
                Type::Tuple => OptionType::Tuple,
                Type::Type => OptionType::Type,
                Type::Object => OptionType::Object,
                Type::Custom(c) => OptionType::Custom(c),
                Type::Union(u) => OptionType::Union(u),
                Type::Option(i, o) => return Type::Option(i + n, o),
            },
        )
    }

    fn std_method(self, name: &Name, runtime: &Runtime) -> Option<StdVarMethod> {
        match self {
            Type::Standard(s) => s.try_method(name, runtime),
            _ => Option::None,
        }
    }
}

fn create_char(var: Variable, runtime: &mut Runtime) -> Result<Variable, ()> {
    let int_val = var.int(runtime)?;
    match int_val.to_u32().and_then(|i| char::try_from(i).ok()) {
        Option::Some(c) => Result::Ok(c.into()),
        Option::None => runtime
            .throw_quick(
                value_error(),
                format!("Cannot convert scalar value {} to a char", int_val).into(),
            )
            .and_then(|_| unreachable!("Should be in native function here")),
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        self.str().to_string()
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
        supers: Vec<u32>,
        variables: HashSet<StringVar>,
        methods: HashMap<Name, StdVarMethod>,
        static_methods: HashMap<Name, StdVarMethod>,
        properties: HashMap<StringVar, Property>,
    ) -> StdType {
        StdType {
            name,
            file_no,
            supers,
            variables,
            methods,
            static_methods,
            properties,
        }
    }

    pub fn get_property(&self, name: &Name) -> Option<Property> {
        name.do_each_ref(|_| Option::None, |str| self.properties.get(&str).cloned())
    }

    fn is_subclass(&self, other: &Type, runtime: &Runtime) -> bool {
        if let Type::Standard(o) = other {
            if self == *o {
                return true;
            }
        }
        for sup in &self.supers {
            if runtime.class_no(*sup).is_subclass(other, runtime) {
                return true;
            }
        }
        false
    }

    pub fn name(&self) -> &StringVar {
        &self.name
    }

    fn index_method(&self, name: &Name) -> Option<(usize, u32)> {
        if let StdVarMethod::Standard(f, a) = self.static_methods.get(name)? {
            Some((*f, *a))
        } else {
            panic!();
        }
    }

    fn convert_variables(&self) -> HashMap<Name, Variable> {
        self.variables
            .iter()
            .map(|x| (Name::Attribute(x.clone()), Variable::default()))
            .collect()
    }

    fn create(&'static self, args: Vec<Variable>, runtime: &mut Runtime) -> Result<Variable, ()> {
        let instance = StdVariable::new(self, self.convert_variables());
        let method = self.methods.get(&Name::Operator(Operator::New)).unwrap();
        StdMethod::new(instance.clone(), *method).call((args, runtime))?;
        Result::Ok(instance.into())
    }

    pub(crate) fn get_method(&self, name: &Name, runtime: &Runtime) -> StdVarMethod {
        self.try_method(&name, runtime).unwrap_or_else(|| {
            panic!(
                "{}.{} does not exist\n{}",
                self.name,
                name.as_str(),
                runtime.stack_frames()
            )
        })
    }

    fn try_method(&self, name: &Name, runtime: &Runtime) -> Option<StdVarMethod> {
        match self.methods.get(&name) {
            Option::Some(t) => Option::Some(*t),
            Option::None => {
                for sup in &self.supers {
                    match runtime.class_no(*sup).std_method(name, runtime) {
                        Option::Some(t) => return Option::Some(t),
                        Option::None => {}
                    }
                }
                default_methods(name)
            }
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Type::Standard(a), Type::Standard(b)) => ptr::eq(*a, *b),
            (Type::Null, Type::Null) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Bigint, Type::Bigint) => true,
            (Type::String, Type::String) => true,
            (Type::Decimal, Type::Decimal) => true,
            (Type::Char, Type::Char) => true,
            (Type::Tuple, Type::Tuple) => true,
            (Type::Type, Type::Type) => true,
            (Type::Custom(a), Type::Custom(b)) => {
                ptr::eq(*a as *const _ as *const (), *b as *const _ as *const ())
            }
            (Type::Union(t), Type::Union(u)) => ptr::eq(*t, *u),
            _ => false,
        }
    }
}

impl Eq for Type {}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Type::Standard(a) => ptr::hash(*a, state),
            Type::Null => 0.hash(state),
            Type::Bool => 1.hash(state),
            Type::Bigint => 2.hash(state),
            Type::String => 3.hash(state),
            Type::Decimal => 4.hash(state),
            Type::Char => 5.hash(state),
            Type::Tuple => 6.hash(state),
            Type::Type => 7.hash(state),
            Type::Object => 8.hash(state),
            Type::Custom(b) => ptr::hash(*b, state),
            Type::Union(c) => ptr::hash(*c, state),
            Type::Option(_, t) => ptr::hash(t, state),
        }
    }
}

impl From<OptionType> for Type {
    fn from(value: OptionType) -> Self {
        match value {
            OptionType::Standard(s) => Type::Standard(s),
            OptionType::Null => Type::Null,
            OptionType::Bool => Type::Bool,
            OptionType::Bigint => Type::Bigint,
            OptionType::String => Type::String,
            OptionType::Decimal => Type::Decimal,
            OptionType::Char => Type::Char,
            OptionType::Tuple => Type::Tuple,
            OptionType::Type => Type::Type,
            OptionType::Object => Type::Object,
            OptionType::Custom(c) => Type::Custom(c),
            OptionType::Union(u) => Type::Union(u),
        }
    }
}
