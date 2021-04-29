use crate::first;
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::string_var::StringVar;
use crate::variable::{FnResult, OptionVar, Variable};

pub fn str(this: OptionVar, runtime: &mut Runtime) -> Result<StringVar, ()> {
    Result::Ok(if this.depth == 1 {
        match this.value {
            Option::Some(x) => format!("Some({})", x.str(runtime)?).into(),
            Option::None => "null".into(),
        }
    } else {
        match this.value {
            Option::Some(x) => fold_some(this.depth, &*x.str(runtime)?),
            Option::None => fold_some(this.depth, "null"),
        }
    })
}

pub fn repr(this: OptionVar, runtime: &mut Runtime) -> Result<StringVar, ()> {
    Result::Ok(if this.depth == 1 {
        match this.value {
            Option::Some(x) => format!("Some({})", x.repr(runtime)?).into(),
            Option::None => "null".into(),
        }
    } else {
        match this.value {
            Option::Some(x) => fold_some(this.depth, &*x.repr(runtime)?),
            Option::None => fold_some(this.depth, "null"),
        }
    })
}

pub fn hash(this: OptionVar, runtime: &mut Runtime) -> Result<usize, ()> {
    match Option::<Variable>::from(this) {
        Option::Some(x) => x.hash(runtime),
        Option::None => Result::Ok(0),
    }
}

fn fold_some(i: usize, x: &str) -> StringVar {
    let prefix = "Some(".repeat(i);
    let suffix = ")".repeat(i);
    (prefix + x + &*suffix).into()
}

pub fn get_attr(this: OptionVar, attr: &str) -> Variable {
    let func = match attr {
        "map" => map_fn,
        "flatMap" => flat_map,
        _ => unimplemented!(),
    };
    StdMethod::new_native(this, func).into()
}

pub fn get_op(this: OptionVar, op: Operator) -> Variable {
    let func = match op {
        Operator::Str => to_str,
        Operator::Repr => to_repr,
        Operator::Hash => to_hash,
        _ => unimplemented!("Option.{}", op.name()),
    };
    StdMethod::new_native(this, func).into()
}

pub fn index(this: OptionVar, name: Name) -> Variable {
    match name {
        Name::Attribute(a) => get_attr(this, a),
        Name::Operator(o) => get_op(this, o),
    }
}

pub fn call_op(
    this: OptionVar,
    op: Operator,
    args: Vec<Variable>,
    runtime: &mut Runtime,
) -> FnResult {
    get_op(this, op).call((args, runtime))
}

fn map_fn(this: OptionVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let result = match this.into() {
        Option::Some(val) => {
            first(args).call((vec![val], runtime))?;
            Option::Some(runtime.pop_return())
        }
        Option::None => Option::None,
    };
    runtime.return_1(result.into())
}

fn flat_map(this: OptionVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    match this.into() {
        Option::Some(val) => {
            first(args).call((vec![val], runtime))?;
            let val = runtime.pop_return();
            runtime.return_1(val)
        }
        Option::None => runtime.return_1(Option::None.into()),
    }
}

fn to_str(this: OptionVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let val = str(this, runtime)?;
    runtime.return_1(val.into())
}

fn to_repr(this: OptionVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let val = repr(this, runtime)?;
    runtime.return_1(val.into())
}

fn to_hash(this: OptionVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    let val = hash(this, runtime)?;
    runtime.return_1(val.into())
}
