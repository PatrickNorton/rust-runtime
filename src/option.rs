use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use std::mem::take;
use std::ops::Deref;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LangOption {
    value: Option<Box<Variable>>,
}

impl LangOption {
    pub fn new(value: Option<Variable>) -> LangOption {
        LangOption {
            value: value.map(Box::new),
        }
    }

    pub fn map<U, F: FnOnce(Variable) -> U>(self, f: F) -> Option<U> {
        match self.value {
            Some(x) => Some(f(*x)),
            None => None,
        }
    }

    pub fn str(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(match &**self {
            Option::Some(val) => format!("Some({})", val.str(runtime)?).into(),
            Option::None => "null".into(),
        })
    }

    pub fn repr(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(match &**self {
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

    pub fn take(self) -> Option<Box<Variable>> {
        self.value
    }

    fn to_str(&self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let result = match &**self {
            Option::Some(val) => format!("Some({})", val.str(runtime)?).into(),
            Option::None => Variable::String("null".into()),
        };
        runtime.return_1(result)
    }

    fn to_repr(&self, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let result = match &**self {
            Option::Some(val) => format!("Some({})", val.repr(runtime)?).into(),
            Option::None => Variable::String("null".into()),
        };
        runtime.return_1(result)
    }

    fn map_fn(&self, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let result = match &**self {
            Option::Some(val) => {
                take(&mut args[0]).call((vec![(**val).clone()], runtime))?;
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
                take(&mut args[0]).call((vec![(**val).clone()], runtime))?;
                runtime.pop_return()
            }
            Option::None => Option::None.into(),
        };
        runtime.return_1(result)
    }
}

impl Deref for LangOption {
    type Target = Option<Box<Variable>>;

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
        x.value.map(|a| *a)
    }
}

impl From<Option<Variable>> for LangOption {
    fn from(x: Option<Variable>) -> Self {
        Self::new(x)
    }
}
