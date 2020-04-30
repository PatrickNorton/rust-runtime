use crate::method::{InnerMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::variable::{FnResult, Variable};

pub fn get_operator(this: char, o: Operator) -> Variable {
    let func = match o {
        Operator::Equals => eq,
        _ => unimplemented!(),
    };
    Variable::Method(Box::new(StdMethod::new(this, InnerMethod::Native(func))))
}

fn eq(this: &char, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    for arg in args {
        if char::from(arg) != *this {
            runtime.push(Variable::Bool(false));
            return FnResult::Ok(());
        }
    }
    runtime.push(Variable::Bool(true));
    FnResult::Ok(())
}
