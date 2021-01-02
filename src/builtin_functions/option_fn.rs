use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, InnerVar, OptionVar, Variable};
use std::mem::take;

pub fn str(i: usize, val: Option<InnerVar>, runtime: &mut Runtime) -> Result<StringVar, ()> {
    Result::Ok(match val {
        Option::Some(x) => fold_some(i, x.str(runtime)?),
        Option::None => "null".into(),
    })
}

pub fn repr(i: usize, val: Option<InnerVar>, runtime: &mut Runtime) -> Result<StringVar, ()> {
    Result::Ok(match val {
        Option::Some(x) => fold_some(i, x.repr(runtime)?),
        Option::None => "null".into(),
    })
}

fn fold_some(i: usize, x: StringVar) -> StringVar {
    (0..i).fold(x, |x, _| format!("Some({})", x).into())
}

pub fn type_of(i: usize, val: Option<&InnerVar>) -> Type {
    val.as_ref()
        .map(|x| x.get_type())
        .unwrap_or(Type::Object)
        .make_option_n(i)
}

pub fn get_attr(this: (usize, Option<InnerVar>), attr: &str) -> Variable {
    let func = match attr {
        "map" => map_fn,
        "flatMap" => flat_map,
        _ => unimplemented!(),
    };
    StdMethod::new_native(this, func).into()
}

pub fn get_op(this: (usize, Option<InnerVar>), op: Operator) -> Variable {
    let func = match op {
        Operator::Str => to_str,
        Operator::Repr => to_repr,
        _ => unimplemented!("Option.{}", op.name()),
    };
    StdMethod::new_native(this, func).into()
}

pub fn index(i: usize, val: Option<InnerVar>, name: Name) -> Variable {
    match name {
        Name::Attribute(a) => get_attr((i, val), a),
        Name::Operator(o) => get_op((i, val), o),
    }
}

pub fn call_op(
    i: usize,
    val: Option<InnerVar>,
    op: Operator,
    args: Vec<Variable>,
    runtime: &mut Runtime,
) -> FnResult {
    get_op((i, val), op).call((args, runtime))
}

fn map_fn(
    this: &(usize, Option<InnerVar>),
    mut args: Vec<Variable>,
    runtime: &mut Runtime,
) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let result = match OptionVar(this.0, this.1.clone()).into() {
        Option::Some(val) => {
            take(&mut args[0]).call((vec![val], runtime))?;
            Option::Some(runtime.pop_return())
        }
        Option::None => Option::None,
    };
    runtime.return_1(result.into())
}

fn flat_map(
    this: &(usize, Option<InnerVar>),
    mut args: Vec<Variable>,
    runtime: &mut Runtime,
) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    match OptionVar(this.0, this.1.clone()).into() {
        Option::Some(val) => {
            take(&mut args[0]).call((vec![val], runtime))?;
            let val = runtime.pop_return();
            runtime.return_1(val)
        }
        Option::None => runtime.return_1(Option::None.into()),
    }
}

fn to_str(
    this: &(usize, Option<InnerVar>),
    args: Vec<Variable>,
    runtime: &mut Runtime,
) -> FnResult {
    debug_assert!(args.is_empty());
    let val = str(this.0, this.1.clone(), runtime)?;
    runtime.return_1(val.into())
}

fn to_repr(
    this: &(usize, Option<InnerVar>),
    args: Vec<Variable>,
    runtime: &mut Runtime,
) -> FnResult {
    debug_assert!(args.is_empty());
    let val = repr(this.0, this.1.clone(), runtime)?;
    runtime.return_1(val.into())
}
