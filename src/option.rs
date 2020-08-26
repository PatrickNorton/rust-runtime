use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
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

    pub fn get_op(&self, op: Operator) -> Variable {
        let func = match op {
            Operator::Str => Self::to_str,
            Operator::Repr => Self::to_repr,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    pub fn call_op(&self, op: Operator, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        self.get_op(op).call((args, runtime))
    }

    pub fn index(&self, name: Name) -> Variable {
        name.do_each(|o| self.get_op(o), |_| unimplemented!())
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
