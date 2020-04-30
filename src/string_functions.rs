use crate::method::{InnerMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use num::{BigInt, ToPrimitive};
use std::mem::replace;
use std::str::FromStr;

pub fn get_operator(this: StringVar, o: Operator) -> Variable {
    let func = match o {
        Operator::Add => add,
        Operator::Multiply => multiply,
        Operator::Bool => bool,
        Operator::Int => int,
        Operator::Str => str,
        Operator::Repr => repr,
        Operator::GetAttr => index,
        _ => unimplemented!("Operator::{:?} unimplemented", o),
    };
    Variable::Method(Box::new(StdMethod::new(this, InnerMethod::Native(func))))
}

fn add(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut result: String = this.parse().unwrap();
    for arg in args {
        result += StringVar::from(arg).as_ref();
    }
    runtime.push(Variable::String(result.into()));
    FnResult::Ok(())
}

fn multiply(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut result: String = this.parse().unwrap();
    for arg in args {
        result = result.repeat(
            BigInt::from(arg)
                .to_usize()
                .expect("Too many string repetitions"),
        );
    }
    runtime.push(Variable::String(result.into()));
    FnResult::Ok(())
}

fn bool(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::Bool(this.is_empty()));
    FnResult::Ok(())
}

fn int(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    match BigInt::from_str(this) {
        Ok(val) => runtime.push(Variable::Bigint(val)),
        Err(_) => runtime.throw(Variable::String("Error in string conversion".into()))?,
    }
    FnResult::Ok(())
}

fn str(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::String(this.clone()));
    FnResult::Ok(())
}

fn repr(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(Variable::String(format!("{:?}", this.as_str()).into()));
    FnResult::Ok(())
}

fn index(this: &StringVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let index = BigInt::from(replace(&mut args[0], Variable::Null()))
        .to_usize()
        .unwrap();
    match this.chars().nth(index) {
        Option::None => runtime.throw_quick(Type::String, "Index out of bounds".into()),
        Option::Some(value) => {
            runtime.push(value.into());
            FnResult::Ok(())
        }
    }
}
