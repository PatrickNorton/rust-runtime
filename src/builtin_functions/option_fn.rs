use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, InnerVar, Variable};
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

pub fn get_attr(this: (usize, Option<InnerVar>), attr: StringVar) -> Variable {
    let func = match attr.as_str() {
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
    let result = if this.0 == 1 {
        match &this.1 {
            Option::Some(val) => {
                take(&mut args[0]).call((vec![Variable::Normal((*val).clone())], runtime))?;
                Option::Some(runtime.pop_return()).into()
            }
            Option::None => Option::None.into(),
        }
    } else {
        take(&mut args[0]).call((vec![Variable::Option(this.0 - 1, this.1.clone())], runtime))?;
        Option::Some(runtime.pop_return()).into()
    };
    runtime.return_1(result)
}

fn flat_map(
    this: &(usize, Option<InnerVar>),
    mut args: Vec<Variable>,
    runtime: &mut Runtime,
) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let result = if this.0 == 1 {
        match &this.1 {
            Option::Some(val) => {
                take(&mut args[0]).call((vec![Variable::from((*val).clone())], runtime))?;
                runtime.pop_return()
            }
            Option::None => Option::None.into(),
        }
    } else {
        take(&mut args[0]).call((vec![Variable::Option(this.0 - 1, this.1.clone())], runtime))?;
        runtime.pop_return()
    };
    runtime.return_1(result)
}

fn to_str(
    this: &(usize, Option<InnerVar>),
    args: Vec<Variable>,
    runtime: &mut Runtime,
) -> FnResult {
    debug_assert!(args.is_empty());
    let result = match &this.1 {
        Option::Some(val) => format!("Some({})", Variable::from(val.clone()).str(runtime)?).into(),
        Option::None => StringVar::from("null").into(),
    };
    runtime.return_1(result)
}

fn to_repr(
    this: &(usize, Option<InnerVar>),
    args: Vec<Variable>,
    runtime: &mut Runtime,
) -> FnResult {
    debug_assert!(args.is_empty());
    let result = match &this.1 {
        Option::Some(val) => format!("Some({})", Variable::from(val.clone()).repr(runtime)?).into(),
        Option::None => StringVar::from("null").into(),
    };
    runtime.return_1(result)
}
