use crate::custom_var::CustomVarWrapper;
use crate::function::Function;
use crate::int_var::IntVar;
use crate::lang_union::LangUnion;
use crate::method::{Method, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::rational_var::RationalVar;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::std_variable::StdVariable;
use crate::string_var::StringVar;
use crate::tuple::LangTuple;
use crate::variable::{FnResult, Variable};
use std::convert::TryFrom;
use std::hash::Hash;
use std::hash::Hasher;
use std::mem::take;
use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LangOption {
    depth: usize,
    value: Option<InnerOption>,
}

#[derive(Debug, Clone)]
pub enum InnerOption {
    Null(),
    Bool(bool),
    Bigint(IntVar),
    String(StringVar),
    Decimal(RationalVar),
    Char(char),
    Type(Type),
    Standard(StdVariable),
    Tuple(LangTuple),
    Method(Box<dyn Method>),
    Function(Function),
    Custom(CustomVarWrapper),
    Union(LangUnion),
}

impl LangOption {
    pub fn new(value: Option<Variable>) -> LangOption {
        value
            .map(|x| match InnerOption::try_from(x) {
                Result::Ok(x) => LangOption {
                    depth: 1,
                    value: Option::Some(x),
                },
                Result::Err(x) => LangOption {
                    depth: x.depth + 1,
                    value: x.value,
                },
            })
            .unwrap_or_else(|| LangOption {
                depth: 1,
                value: Option::None,
            })
    }

    pub fn map<U, F: FnOnce(Variable) -> U>(self, f: F) -> Option<U> {
        match self.value {
            Some(x) => Some(f(x.into())),
            None => None,
        }
    }

    pub fn str(self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(match self.take() {
            Option::Some(val) => format!("Some({})", val.str(runtime)?).into(),
            Option::None => "null".into(),
        })
    }

    pub fn repr(self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(match self.take() {
            Option::Some(val) => format!("Some({})", val.repr(runtime)?).into(),
            Option::None => "null".into(),
        })
    }

    pub fn get_attr(&self, attr: StringVar) -> Variable {
        let func = match attr.as_str() {
            "map" => Self::map_fn,
            "flatMap" => Self::flat_map,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    pub fn get_op(&self, op: Operator) -> Variable {
        let func = match op {
            Operator::Str => Self::to_str,
            Operator::Repr => Self::to_repr,
            _ => unimplemented!("Option.{}", op.name()),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    pub fn call_op(&self, op: Operator, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.get_op(op).call((args, runtime))
    }

    pub fn index(&self, name: Name) -> Variable {
        name.do_each(|o| self.get_op(o), |a| self.get_attr(a))
    }

    pub fn take(self) -> Option<Variable> {
        self.value.map(Variable::from)
    }

    pub fn get_type(&self) -> Type {
        self.value
            .as_ref()
            .map(|x| x.get_type())
            .unwrap_or(Type::Object)
            .make_option_n(self.depth)
    }

    fn to_str(&self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let result = match &**self {
            Option::Some(val) => {
                format!("Some({})", Variable::from(val.clone()).str(runtime)?).into()
            }
            Option::None => Variable::String("null".into()),
        };
        runtime.return_1(result)
    }

    fn to_repr(&self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let result = match &**self {
            Option::Some(val) => {
                format!("Some({})", Variable::from(val.clone()).repr(runtime)?).into()
            }
            Option::None => Variable::String("null".into()),
        };
        runtime.return_1(result)
    }

    fn map_fn(&self, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let result = match &**self {
            Option::Some(val) => {
                take(&mut args[0]).call((vec![Variable::from((*val).clone())], runtime))?;
                Option::Some(runtime.pop_return()).into()
            }
            Option::None => Option::None.into(),
        };
        runtime.return_1(result)
    }

    fn flat_map(&self, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let result = match &**self {
            Option::Some(val) => {
                take(&mut args[0]).call((vec![Variable::from((*val).clone())], runtime))?;
                runtime.pop_return()
            }
            Option::None => Option::None.into(),
        };
        runtime.return_1(result)
    }
}

impl InnerOption {
    pub fn get_type(&self) -> Type {
        match self {
            InnerOption::Null() => Type::Null,
            InnerOption::Bool(_) => Type::Bool,
            InnerOption::Bigint(_) => Type::Bigint,
            InnerOption::String(_) => Type::String,
            InnerOption::Decimal(_) => Type::Decimal,
            InnerOption::Char(_) => Type::Char,
            InnerOption::Type(_) => Type::Type,
            InnerOption::Standard(s) => s.get_type(),
            InnerOption::Tuple(_) => Type::Tuple,
            InnerOption::Method(_) => unimplemented!(),
            InnerOption::Function(_) => unimplemented!(),
            InnerOption::Custom(c) => (**c).clone().get_type(),
            InnerOption::Union(u) => u.get_type(),
        }
    }
}

impl Deref for LangOption {
    type Target = Option<InnerOption>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl From<LangOption> for Variable {
    fn from(x: LangOption) -> Self {
        Variable::Option(x)
    }
}

impl From<LangOption> for Option<Variable> {
    fn from(x: LangOption) -> Self {
        x.value.map(|a| a.into())
    }
}

impl From<Option<Variable>> for LangOption {
    fn from(x: Option<Variable>) -> Self {
        Self::new(x)
    }
}

impl PartialEq for InnerOption {
    fn eq(&self, other: &Self) -> bool {
        Variable::from(self.clone()).eq(&other.clone().into())
    }
}

impl Eq for InnerOption {}

impl Hash for InnerOption {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&Variable::from(self.clone()), state)
    }
}

impl From<InnerOption> for Variable {
    fn from(x: InnerOption) -> Self {
        match x {
            InnerOption::Null() => Variable::Null(),
            InnerOption::Bool(b) => Variable::Bool(b),
            InnerOption::Bigint(b) => Variable::Bigint(b),
            InnerOption::String(s) => Variable::String(s),
            InnerOption::Decimal(d) => Variable::Decimal(d),
            InnerOption::Char(c) => Variable::Char(c),
            InnerOption::Type(t) => Variable::Type(t),
            InnerOption::Standard(s) => Variable::Standard(s),
            InnerOption::Tuple(t) => Variable::Tuple(t),
            InnerOption::Method(m) => Variable::Method(m),
            InnerOption::Function(f) => Variable::Function(f),
            InnerOption::Custom(c) => Variable::Custom(c),
            InnerOption::Union(u) => Variable::Union(u),
        }
    }
}

impl TryFrom<Variable> for InnerOption {
    type Error = LangOption;

    fn try_from(value: Variable) -> Result<Self, Self::Error> {
        Result::Ok(match value {
            Variable::Null() => InnerOption::Null(),
            Variable::Bool(b) => InnerOption::Bool(b),
            Variable::Bigint(b) => InnerOption::Bigint(b),
            Variable::String(s) => InnerOption::String(s),
            Variable::Decimal(d) => InnerOption::Decimal(d),
            Variable::Char(c) => InnerOption::Char(c),
            Variable::Type(t) => InnerOption::Type(t),
            Variable::Standard(s) => InnerOption::Standard(s),
            Variable::Tuple(t) => InnerOption::Tuple(t),
            Variable::Method(m) => InnerOption::Method(m),
            Variable::Function(f) => InnerOption::Function(f),
            Variable::Custom(c) => InnerOption::Custom(c),
            Variable::Union(u) => InnerOption::Union(u),
            Variable::Option(o) => return Result::Err(o),
        })
    }
}
