use crate::method::{InnerMethod, NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::tuple::LangTuple;
use crate::variable::{FnResult, Variable};

pub fn op_fn(o: Operator) -> NativeMethod<LangTuple> {
    match o {
        Operator::Bool => bool,
        Operator::Str => str,
        Operator::Repr => repr,
        _ => unimplemented!("Operator {:?} unimplemented", o),
    }
}

pub fn get_operator(this: LangTuple, o: Operator) -> Variable {
    let func = op_fn(o);
    Variable::Method(Box::new(StdMethod::new(this, InnerMethod::Native(func))))
}

pub fn get_attr(this: LangTuple, s: StringVar) -> Variable {
    match s.as_str().parse() {
        Result::Ok(x) => this[x].clone(),
        Result::Err(_) => unimplemented!(),
    }
}

pub fn bool(this: &LangTuple, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1((!this.is_empty()).into())
}

pub fn str(this: &LangTuple, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let result = this.str(runtime)?.into();
    runtime.return_1(result)
}

pub fn repr(this: &LangTuple, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let result = this.repr(runtime)?.into();
    runtime.return_1(result)
}
