use crate::custom_types::exceptions::index_error;
use crate::int_var::IntVar;
use crate::method::{InnerMethod, NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use num::ToPrimitive;
use std::mem::replace;
use std::str::FromStr;

pub fn op_fn(o: Operator) -> NativeMethod<StringVar> {
    match o {
        Operator::Add => add,
        Operator::Multiply => multiply,
        Operator::Bool => bool,
        Operator::Int => int,
        Operator::Str => str,
        Operator::Repr => repr,
        Operator::GetAttr => index,
        _ => unimplemented!("Operator::{:?} unimplemented", o),
    }
}

pub fn get_operator(this: StringVar, o: Operator) -> Variable {
    let func = op_fn(o);
    Variable::Method(Box::new(StdMethod::new(this, InnerMethod::Native(func))))
}

pub fn get_attr(this: StringVar, s: StringVar) -> Variable {
    let func = match s.as_str() {
        "length" => return Variable::Bigint(this.len().into()),
        "upper" => upper,
        "lower" => lower,
        _ => unimplemented!(),
    };
    Variable::Method(StdMethod::new_native(this, func))
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
            IntVar::from(arg)
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
    match IntVar::from_str(this) {
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
    let index = IntVar::from(replace(&mut args[0], Variable::Null()))
        .to_usize()
        .unwrap();
    match this.chars().nth(index) {
        Option::None => runtime.throw_quick(index_error(), "Index out of bounds".into()),
        Option::Some(value) => {
            runtime.push(value.into());
            FnResult::Ok(())
        }
    }
}

fn upper(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(this.to_uppercase().into());
    FnResult::Ok(())
}

fn lower(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.push(this.to_lowercase().into());
    FnResult::Ok(())
}
