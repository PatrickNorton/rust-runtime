use crate::custom_types::exceptions::{index_error, stop_iteration};
use crate::custom_var::CustomVar;
use crate::int_var::IntVar;
use crate::looping::{IterResult, NativeIterator};
use crate::method::{InnerMethod, NativeMethod, StdMethod};
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Name, Variable};
use num::ToPrimitive;
use std::cell::Cell;
use std::mem::replace;
use std::rc::Rc;
use std::str::{from_utf8_unchecked, FromStr};

pub fn op_fn(o: Operator) -> NativeMethod<StringVar> {
    match o {
        Operator::Add => add,
        Operator::Multiply => multiply,
        Operator::Bool => bool,
        Operator::Int => int,
        Operator::Str => str,
        Operator::Repr => repr,
        Operator::GetAttr => index,
        Operator::Iter => iter,
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
        "join" => join,
        "joinAll" => join_all,
        _ => unimplemented!(),
    };
    Variable::Method(StdMethod::new_native(this, func))
}

fn add(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut result: String = this.parse().unwrap();
    for arg in args {
        result += StringVar::from(arg).as_ref();
    }
    runtime.return_1(Variable::String(result.into()))
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
    runtime.return_1(Variable::String(result.into()))
}

fn bool(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::Bool(this.is_empty()))
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
    runtime.return_1(Variable::String(this.clone()))
}

fn repr(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Variable::String(format!("{:?}", this.as_str()).into()))
}

fn index(this: &StringVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert_eq!(args.len(), 1);
    let index = IntVar::from(replace(&mut args[0], Variable::Null()))
        .to_usize()
        .unwrap();
    match this.chars().nth(index) {
        Option::None => runtime.throw_quick(index_error(), "Index out of bounds".into()),
        Option::Some(value) => runtime.return_1(value.into()),
    }
}

fn iter(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(Rc::new(StringIter::new(this.clone())).into())
}

fn upper(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.to_uppercase().into())
}

fn lower(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.is_empty());
    runtime.return_1(this.to_lowercase().into())
}

fn join(this: &StringVar, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    debug_assert!(args.len() == 1);
    let mut is_first = true;
    let mut result = String::new();
    let iter = replace(&mut args[0], Variable::Null()).iter(runtime)?;
    while let Option::Some(val) = iter.next(runtime)? {
        if !is_first {
            result += this;
        }
        is_first = false;
        result += val.str(runtime)?.as_str();
    }
    runtime.return_1(result.into())
}

fn join_all(this: &StringVar, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
    let mut result = String::with_capacity(this.len() * args.len());
    let len = args.len();
    for (i, val) in args.into_iter().enumerate() {
        result += val.str(runtime)?.as_str();
        if i + 1 < len {
            result += this;
        }
    }
    runtime.return_1(result.into())
}

#[derive(Debug)]
pub struct StringIter {
    index: Cell<usize>,
    val: StringVar,
}

impl StringIter {
    fn new(val: StringVar) -> StringIter {
        StringIter {
            val,
            index: Cell::new(0),
        }
    }

    fn next_func(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        match self.next_fn() {
            Option::Some(ret) => runtime.return_1(ret),
            Option::None => runtime.throw_quick(stop_iteration(), "".into()),
        }
    }

    fn next_fn(&self) -> Option<Variable> {
        if self.index.get() != self.val.len() {
            let mut indices = unsafe {
                // We know this is safe b/c:
                // * The slice comes from a valid str, therefore, no invalid UTF-8 can be entered
                // * self.index is always on a valid char boundary, as received by char_indices
                from_utf8_unchecked(&self.val.as_bytes()[self.index.get()..])
            }
            .char_indices();
            match indices.next() {
                Option::Some((_, c)) => {
                    self.index.set(
                        indices
                            .next()
                            .map(|a| self.index.get() + a.0)
                            .unwrap_or_else(|| self.val.len()),
                    );
                    Option::Some(c.into())
                }
                Option::None => Option::None,
            }
        } else {
            Option::None
        }
    }
}

impl CustomVar for StringIter {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        if let Name::Attribute(n) = name {
            if &*n == "next" {
                Variable::Method(StdMethod::new_native(self, Self::next_func))
            } else {
                unimplemented!()
            }
        } else {
            unimplemented!()
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        unimplemented!()
    }
}

impl NativeIterator for StringIter {
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.next_fn())
    }
}
