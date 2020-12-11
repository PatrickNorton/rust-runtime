use crate::method::{NativeCopyMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};

pub fn op_fn(o: Operator) -> NativeCopyMethod<char> {
    match o {
        Operator::Equals => eq,
        Operator::Str => str,
        Operator::Repr => str,
        x => unimplemented!("char.{}", x.name()),
    }
}

pub fn get_operator(this: char, o: Operator) -> Variable {
    let func = op_fn(o);
    Variable::Method(StdMethod::new_move(this, func))
}

fn eq(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    runtime.return_1(args.into_iter().any(|arg| char::from(arg) != this).into())
}

fn str(this: char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::String(this.to_string().into()))
}
