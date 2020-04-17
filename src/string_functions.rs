use crate::method::{InnerMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::Variable;

pub fn get_operator(this: &StringVar, o: Operator) -> Variable {
    let func: fn(&StringVar, Vec<Variable>, &mut Runtime) = match o {
        Operator::Add => add,
        _ => unimplemented!("Operator::{:?} unimplemented", o),
    };
    Variable::Method(Box::new(StdMethod::new(
        this.clone(),
        InnerMethod::Native(func),
    )))
}

fn add(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) {
    let mut result: String = this.parse().unwrap();
    for arg in args {
        result += arg.str(runtime).as_ref();
    }
    runtime.push(Variable::String(result.into()));
}
