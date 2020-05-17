use crate::method::{NativeCopyMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};

pub fn op_fn(o: Operator) -> NativeCopyMethod<()> {
    match o {
        Operator::Equals => eq,
        Operator::Str => str,
        Operator::Repr => str,
        Operator::Bool => bool,
        _ => unimplemented!(),
    }
}

pub fn get_operator(o: Operator) -> Variable {
    let func = op_fn(o);
    Variable::Method(StdMethod::new_move((), func))
}

fn eq(_this: (), args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if let Variable::Null() = arg {
            return runtime.return_1(false.into());
        }
    }
    runtime.return_1(true.into())
}

fn str(_this: (), args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::String("null".into()))
}

fn bool(_this: (), args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(false.into())
}
