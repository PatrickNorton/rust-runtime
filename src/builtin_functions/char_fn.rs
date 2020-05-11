use crate::method::{InnerMethod, NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};

pub fn op_fn(o: Operator) -> NativeMethod<char> {
    match o {
        Operator::Equals => eq,
        Operator::Str => str,
        Operator::Repr => str,
        _ => unimplemented!(),
    }
}

pub fn get_operator(this: char, o: Operator) -> Variable {
    let func = op_fn(o);
    Variable::Method(Box::new(StdMethod::new(this, InnerMethod::Native(func))))
}

fn eq(this: &char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if char::from(arg) != *this {
            return runtime.return_1(Variable::Bool(false));
        }
    }
    runtime.return_1(Variable::Bool(true))
}

fn str(this: &char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::String(this.to_string().into()))
}
