use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};

pub fn op_fn(o: Operator) -> NativeMethod<()> {
    match o {
        Operator::Equals => eq,
        Operator::Str => str,
        Operator::Repr => str,
        _ => unimplemented!(),
    }
}

pub fn get_operator(o: Operator) -> Variable {
    let func = op_fn(o);
    Variable::Method(StdMethod::new_native((), func))
}

fn eq(_this: &(), args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if let Variable::Null() = arg {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn str(_this: &(), args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::String("null".into()))
}
