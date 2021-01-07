use crate::int_var::IntVar;
use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::tuple::LangTuple;
use crate::variable::{FnResult, InnerVar, Variable};

pub fn op_fn(o: Operator) -> NativeMethod<LangTuple> {
    match o {
        Operator::Equals => equals,
        Operator::Bool => bool,
        Operator::Str => str,
        Operator::Repr => repr,
        Operator::Hash => hash,
        _ => unimplemented!("tuple.{} unimplemented", o.name()),
    }
}

pub fn get_operator(this: LangTuple, o: Operator) -> Variable {
    StdMethod::new_native(this, op_fn(o)).into()
}

pub fn get_attr(this: LangTuple, s: &str) -> Variable {
    if s == "length" {
        return IntVar::from(this.len()).into();
    }
    match s.parse() {
        Result::Ok(x) => this[x].clone(),
        Result::Err(_) => unimplemented!("tuple.{}", s),
    }
}

pub fn equals(this: LangTuple, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        match arg {
            Variable::Normal(InnerVar::Tuple(other)) => {
                if this.len() != other.len() {
                    return runtime.return_1(false.into());
                }
                for (x, y) in this.iter().zip(&other) {
                    if !x.equals(y.clone(), runtime)? {
                        return runtime.return_1(false.into());
                    }
                }
            }
            _ => {
                if !arg.equals(this.clone().into(), runtime)? {
                    return runtime.return_1(false.into());
                }
            }
        }
    }
    runtime.return_1(true.into())
}

pub fn bool(this: LangTuple, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1((!this.is_empty()).into())
}

pub fn str(this: LangTuple, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let result = this.str(runtime)?.into();
    runtime.return_1(result)
}

pub fn repr(this: LangTuple, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let result = this.repr(runtime)?.into();
    runtime.return_1(result)
}

pub fn hash(this: LangTuple, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let result = IntVar::from(this.lang_hash(runtime)?).into();
    runtime.return_1(result)
}
