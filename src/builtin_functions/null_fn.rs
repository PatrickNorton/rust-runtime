use crate::method::{NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};

pub fn op_fn(o: Operator) -> NativeMethod<()> {
    match o {
        Operator::Equals => eq,
        Operator::Str => str,
        Operator::Repr => str,
        Operator::Bool => bool,
        x => unimplemented!("null.{}", x.name()),
    }
}

pub fn get_operator(o: Operator) -> Variable {
    let func = op_fn(o);
    StdMethod::new_native((), func).into()
}

fn eq(_this: (), args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if arg.is_null() {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn str(_this: (), args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(StringVar::from("null").into())
}

fn bool(_this: (), args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(false.into())
}
