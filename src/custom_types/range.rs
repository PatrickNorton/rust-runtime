use crate::custom_types::exceptions::{index_error, value_error};
use crate::custom_var::{downcast_var, CustomVar};
use crate::int_var::IntVar;
use crate::looping::{self, TypicalIterator};
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use crate::{first, first_n};
use num::{One, Signed, Zero};
use std::cell::RefCell;
use std::mem::replace;
use std::ops::Neg;
use std::rc::Rc;

#[derive(Debug, Eq, PartialEq)]
pub struct Range {
    start: IntVar,
    stop: IntVar,
    step: IntVar,
}

impl Range {
    pub fn new(start: IntVar, stop: IntVar, step: IntVar) -> Range {
        Range { start, stop, step }
    }

    pub fn get_start(&self) -> &IntVar {
        &self.start
    }

    pub fn get_stop(&self) -> &IntVar {
        &self.stop
    }

    pub fn get_step(&self) -> &IntVar {
        &self.step
    }

    pub fn values(&self) -> impl Iterator<Item = IntVar> + '_ {
        RangeValueIter {
            current: self.start.clone(),
            value: self,
        }
    }

    pub fn from_slice(len: usize, runtime: &mut Runtime, arg: Variable) -> Result<Rc<Range>, ()> {
        runtime.call_attr(arg, "toRange", vec![len.into()])?;
        Result::Ok(downcast_var(runtime.pop_return()).expect("Expected a range"))
    }

    fn before_end(&self, value: &IntVar) -> bool {
        if self.step.is_positive() {
            value < &self.stop
        } else {
            value > &self.stop
        }
    }

    fn op_fn(o: Operator) -> NativeMethod<Rc<Range>> {
        match o {
            Operator::Str => Self::str,
            Operator::Repr => Self::str,
            Operator::Equals => Self::eq,
            Operator::Iter => Self::iter,
            Operator::GetAttr => Self::index,
            Operator::In => Self::contains,
            Operator::Reversed => Self::reversed,
            _ => unimplemented!("range.{}", o.name()),
        }
    }

    fn str(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.to_str().into())
    }

    fn eq(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let is_eq = match downcast_var::<Range>(first(args)) {
            Result::Err(_) => false,
            Result::Ok(other) => self == other,
        };
        runtime.return_1(is_eq.into())
    }

    fn iter(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(RangeIter::new(self)).into())
    }

    fn index(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let index = IntVar::from(first(args));
        let result = &self.start + &(&index * &self.step);
        if !self.before_end(&result) {
            let max_index = (&self.stop - &self.start) / self.step.clone();
            runtime.throw_quick(
                index_error(),
                format!(
                    "Index {} out of bounds for {} (max index is {})",
                    result,
                    self.to_str(),
                    max_index,
                ),
            )
        } else {
            runtime.return_1(result.into())
        }
    }

    fn contains(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let value = IntVar::from(first(args));
        let result = if self.step.is_positive() {
            value >= self.start && value < self.stop
        } else {
            value <= self.start && value > self.stop
        } && (&(&value - &self.start) % &self.step).is_zero();
        runtime.return_1(result.into())
    }

    fn reversed(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let new_stop = &self.start - &self.step;
        let new_start = &self.stop - &self.step;
        let new_step = (&self.step).neg(); // Turn into -(&self.step) when IDE stops making it an error
        runtime.return_1(Rc::new(Self::new(new_start, new_stop, new_step)).into())
    }

    fn get(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let index = IntVar::from(first(args));
        let result = &self.start + &(&index * &self.step);
        if !self.before_end(&result) {
            runtime.return_1(Option::None.into())
        } else {
            runtime.return_1(Option::Some(Variable::from(result)).into())
        }
    }

    fn to_str(&self) -> StringVar {
        if self.step.is_one() {
            format!("[{}:{}]", self.start, self.stop).into()
        } else {
            format!("[{}:{}:{}]", self.start, self.stop, self.step).into()
        }
    }

    fn len(&self) -> IntVar {
        let (start, stop) = if self.step.is_negative() {
            (&self.stop, &self.start)
        } else {
            (&self.start, &self.stop)
        };
        (stop - start) / self.step.abs()
    }

    fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 3);
        let [start, stop, step] = first_n(args);
        let range = Range::new(start.into(), stop.into(), step.into());
        if range.step.is_zero() {
            runtime.throw_quick(value_error(), "Step cannot be 0")
        } else {
            runtime.return_1(Rc::new(range).into())
        }
    }

    pub fn range_type() -> Type {
        custom_class!(Range, create, "range")
    }
}

impl CustomVar for Range {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::range_type()
    }

    fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
        let func = Range::op_fn(op);
        StdMethod::new_native(self, func).into()
    }

    fn get_attribute(self: Rc<Self>, attr: &str) -> Variable {
        let func = match attr {
            "length" => return self.len().into(),
            "get" => Self::get,
            x => unimplemented!("Range.{}", x),
        };
        StdMethod::new_native(self, func).into()
    }

    fn call_op(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(Range::op_fn(operator), self, args)
    }

    fn call_op_or_goto(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(Range::op_fn(operator), self, args)
    }

    fn str(self: Rc<Self>, _runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(self.to_str())
    }

    fn repr(self: Rc<Self>, _runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(self.to_str())
    }

    fn iter(self: Rc<Self>, _runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        Result::Ok(Rc::new(RangeIter::new(self)).into())
    }
}

#[derive(Debug)]
struct RangeIter {
    current: RefCell<IntVar>,
    value: Rc<Range>,
}

impl RangeIter {
    pub fn new(value: Rc<Range>) -> RangeIter {
        RangeIter {
            current: RefCell::new(value.start.clone()),
            value,
        }
    }

    fn true_next(&self) -> Option<IntVar> {
        if self.value.before_end(&*self.current.borrow()) {
            Option::Some(self.current.replace_with(|x| self.value.get_step() + x))
        } else {
            Option::None
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }

    fn range_iter_type() -> Type {
        custom_class!(RangeIter, create, "RangeIter")
    }
}

impl TypicalIterator for RangeIter {
    fn inner_next(&self) -> Option<Variable> {
        self.true_next().map(Into::into)
    }

    fn get_type() -> Type {
        Self::range_iter_type()
    }
}

#[derive(Debug, Clone)]
struct RangeValueIter<'a> {
    current: IntVar,
    value: &'a Range,
}

impl Iterator for RangeValueIter<'_> {
    type Item = IntVar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.value.before_end(&self.current) {
            let new = &self.current + self.value.get_step();
            Option::Some(replace(&mut self.current, new))
        } else {
            Option::None
        }
    }
}
