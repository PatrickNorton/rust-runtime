use crate::method::{InnerMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::variable::Variable;

pub fn get_operator(this: &String, o: Operator) -> Variable {
    let func: fn(&String, Vec<Variable>, &mut Runtime) = match o {
        Operator::Add => add,
        _ => unimplemented!("Operator::{:?} unimplemented", o),
    };
    Variable::Method(Box::new(StdMethod::new(
        this.clone(),
        InnerMethod::Native(func),
    )))
}

fn add(this: &String, args: Vec<Variable>, runtime: &mut Runtime) {
    let mut result = this.clone();
    for arg in args {
        result += arg.str(runtime).as_ref();
    }
    runtime.push(Variable::String(result));
}
