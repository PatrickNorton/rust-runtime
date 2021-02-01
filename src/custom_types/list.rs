use crate::custom_types::exceptions::{index_error, value_error};
use crate::custom_types::join_values;
use crate::custom_types::range::Range;
use crate::custom_var::{downcast_var, CustomVar};
use crate::int_var::{normalize, IntVar};
use crate::looping::{self, IterResult, NativeIterator};
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use crate::{first, first_two};
use num::{One, Signed, ToPrimitive, Zero};
use std::cell::{Cell, RefCell};
use std::cmp::min;
use std::iter::repeat_with;
use std::rc::Rc;

#[derive(Debug)]
pub struct List {
    generic: Type,
    value: RefCell<Vec<Variable>>,
}

impl List {
    pub fn from_values(generic: Type, values: Vec<Variable>) -> Rc<List> {
        Rc::new(List {
            generic,
            value: RefCell::new(values),
        })
    }

    pub fn len(&self) -> usize {
        self.value.borrow().len()
    }

    fn op_fn(name: Operator) -> NativeMethod<Rc<List>> {
        match name {
            Operator::Bool => List::list_bool,
            Operator::Str => List::list_str,
            Operator::Repr => List::list_repr,
            Operator::GetAttr => List::list_index,
            Operator::SetAttr => List::set_index,
            Operator::Equals => List::eq,
            Operator::Iter => List::iter,
            Operator::In => List::contains,
            Operator::Reversed => List::reversed,
            Operator::GetSlice => List::get_slice,
            Operator::SetSlice => List::set_slice,
            Operator::DelSlice => List::del_slice,
            Operator::IterSlice => List::iter_slice,
            Operator::Add => List::plus,
            Operator::Multiply => List::times,
            _ => unimplemented!("List.{}", name.name()),
        }
    }

    fn get_operator(self: Rc<Self>, name: Operator) -> Variable {
        let value = List::op_fn(name);
        StdMethod::new_native(self, value).into()
    }

    fn get_attribute(self: Rc<Self>, name: &str) -> Variable {
        let value = match name {
            "length" => return IntVar::from(self.len()).into(),
            "containsAll" => Self::contains_all,
            "get" => Self::list_get,
            "reverse" => Self::reverse,
            "count" => Self::count,
            "clear" => Self::clear,
            "add" => Self::add,
            "addAll" => Self::add_all,
            "insert" => Self::insert,
            "indexOf" => Self::index_of,
            "pop" => Self::pop,
            "popFirst" => Self::pop_first,
            "swap" => Self::swap,
            x => unimplemented!("List.{}", x),
        };
        StdMethod::new_native(self, value).into()
    }

    fn list_bool(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1((!self.value.borrow().is_empty()).into())
    }

    fn list_str(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let value = self.str(runtime)?;
        runtime.return_1(value.into())
    }

    fn list_repr(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let value = self.repr(runtime)?;
        runtime.return_1(value.into())
    }

    fn list_index(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let val = self.value.borrow();
        match normalize(val.len(), first(args).into()) {
            Result::Ok(index) => runtime.return_1(val[index].clone()),
            Result::Err(index) => self.index_error(index, runtime),
        }
    }

    fn set_index(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let (signed_index, value) = first_two(args);
        let len = self.value.borrow().len(); // Keep out of match to prevent double-borrow error
        match normalize(len, signed_index.into()) {
            Result::Ok(index) => {
                if value.get_type().is_subclass(&self.generic, runtime) {
                    self.value.borrow_mut()[index] = value;
                } else {
                    panic!("Bad type for list.operator []=")
                }
                runtime.return_0()
            }
            Result::Err(index) => self.index_error(index, runtime),
        }
    }

    fn list_get(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let val = self.value.borrow();
        if args.len() == 1 {
            runtime.return_1(match normalize(val.len(), first(args).into()) {
                Result::Ok(index) => Option::Some(val[index].clone()).into(),
                Result::Err(_) => Option::None.into(),
            })
        } else {
            debug_assert_eq!(args.len(), 2);
            let (signed_index, default) = first_two(args);
            runtime.return_1(match normalize(val.len(), signed_index.into()) {
                Result::Ok(index) => val[index].clone(),
                Result::Err(_) => default,
            })
        }
    }

    fn plus(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let iter = first(args).iter(runtime)?;
        let mut new = Vec::new();
        while let Option::Some(val) = iter.next(runtime)?.take_first() {
            if !val.get_type().is_subclass(&self.generic, runtime) {
                panic!(
                    "Bad type for list[{}].addAll: {}\n{}",
                    self.generic.str(),
                    val.get_type().str(),
                    runtime.frame_strings(),
                )
            } else {
                new.push(val)
            }
        }
        runtime.return_1(List::from_values(self.generic, new).into())
    }

    fn times(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let times = IntVar::from(first(args));
        if times.is_negative() {
            return runtime.throw_quick(
                value_error(),
                format!(
                    "Cannot multiply list: Expected non-negative number, got {}",
                    times
                ),
            );
        }
        let values = self.value.borrow();
        if values.is_empty() || times.is_zero() {
            runtime.return_1(List::from_values(self.generic, Vec::new()).into())
        } else if times.is_one() {
            runtime.return_1(List::from_values(self.generic, values.clone()).into())
        } else {
            match times.to_usize() {
                Option::Some(x) => {
                    if x.checked_mul(values.len()).is_none() {
                        runtime.throw_quick(
                            value_error(),
                            format!("List repetition {} times too big to fit in memory", times),
                        )
                    } else {
                        let new = repeat_with(|| values.clone()).take(x).flatten().collect();
                        runtime.return_1(List::from_values(self.generic, new).into())
                    }
                }
                Option::None => runtime.throw_quick(
                    value_error(),
                    format!("List repetition {} too big to fit in memory", times),
                ),
            }
        }
    }

    fn pop(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.value.borrow_mut().pop().into())
    }

    fn pop_first(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        if self.value.borrow().is_empty() {
            runtime.return_1(Option::None.into())
        } else {
            runtime.return_1(Option::Some(self.value.borrow_mut().remove(0)).into())
        }
    }

    fn swap(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let len = self.value.borrow().len();
        let (index_1, index_2) = first_two(args);
        match normalize(len, index_1.into()) {
            Result::Ok(i1) => match normalize(len, index_2.into()) {
                Result::Ok(i2) => {
                    self.value.borrow_mut().swap(i1, i2);
                    runtime.return_0()
                }
                Result::Err(i2) => self.index_error(i2, runtime),
            },
            Result::Err(i1) => self.index_error(i1, runtime),
        }
    }

    fn index_error(&self, index: IntVar, runtime: &mut Runtime) -> FnResult {
        runtime.throw_quick(
            index_error(),
            format!(
                "Index {} out of bounds for list of length {}",
                index,
                self.value.borrow().len()
            ),
        )
    }

    fn contains(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        runtime.return_1(self.value.borrow().contains(&args[0]).into())
    }

    fn contains_all(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let checked_var = first(args);
        let this_iter = checked_var.iter(runtime)?;
        while let Option::Some(val) = this_iter.next(runtime)?.take_first() {
            if !self.value.borrow().contains(&val) {
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn index_of(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let searcher = first(args);
        for (i, var) in self.value.borrow().iter().enumerate() {
            if searcher.clone().equals(var.clone(), runtime)? {
                return runtime.return_1(Option::Some(IntVar::from(i).into()).into());
            }
        }
        runtime.return_1(Option::None.into())
    }

    fn reversed(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(ListRevIter::new(self)).into())
    }

    fn reverse(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.value.borrow_mut().reverse();
        runtime.return_0()
    }

    fn count(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let mut count: usize = 0;
        for x in &*self.value.borrow() {
            if x.clone().equals(args[0].clone(), runtime)? {
                count += 1;
            }
        }
        runtime.return_1(IntVar::from(count).into())
    }

    fn clear(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.value.borrow_mut().clear();
        runtime.return_0()
    }

    fn add(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        if !args[0].get_type().is_subclass(&self.generic, runtime) {
            panic!(
                "Bad type for list.add: got {}, expected {}\n{}",
                args[0].get_type().str(),
                self.generic.str(),
                runtime.frame_strings()
            )
        }
        self.value.borrow_mut().push(first(args));
        runtime.return_0()
    }

    fn add_all(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let iterator = first(args).iter(runtime)?;
        let mut value = self.value.borrow_mut();
        while let Option::Some(val) = iterator.next(runtime)?.take_first() {
            if !val.get_type().is_subclass(&self.generic, runtime) {
                panic!(
                    "Bad type for list[{}].addAll: {}\n{}",
                    self.generic.str(),
                    val.get_type().str(),
                    runtime.frame_strings(),
                )
            }
            value.push(val);
        }
        runtime.return_0()
    }

    fn eq(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        for arg in args {
            if !match downcast_var::<List>(arg) {
                Option::None => false,
                Option::Some(other) => {
                    let self_val = self.value.borrow();
                    let other_val = other.value.borrow();
                    self_val.len() == other_val.len()
                        && Self::vec_eq(&*self_val, &*other_val, runtime)?
                }
            } {
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn vec_eq(first: &[Variable], second: &[Variable], runtime: &mut Runtime) -> Result<bool, ()> {
        let mut is_eq = true;
        for (a, b) in first.iter().zip(second.iter()) {
            if !a.clone().equals(b.clone(), runtime)? {
                is_eq = false;
                break;
            }
        }
        Result::Ok(is_eq)
    }

    fn get_slice(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let range = self.slice_to_range(runtime, first(args))?;
        if range.get_step().is_one() {
            let value = self.value.borrow();
            let start = range.get_start().to_usize().unwrap();
            let stop = range.get_stop().to_usize().unwrap_or(usize::MAX);
            runtime.return_1(List::from_values(self.generic, value[start..stop].to_vec()).into())
        } else {
            let mut raw_vec = Vec::new();
            let self_val = self.value.borrow();
            for i in range.values() {
                raw_vec.push(self_val[i.to_usize().expect("Conversion error")].clone());
            }
            runtime.return_1(List::from_values(self.generic, raw_vec).into())
        }
    }

    fn set_slice(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let (range, values) = first_two(args);
        let range = self.slice_to_range(runtime, range)?;
        if !range.get_step().is_one() {
            return runtime.throw_quick(
                value_error(),
                format!(
                    "list.operator [:]= is only valid with a slice step of 1, not {}",
                    range.get_step()
                ),
            );
        }
        let range_end = range.get_stop().to_usize().unwrap_or(usize::MAX);
        let value_iter = values.iter(runtime)?;
        let mut array = self.value.borrow_mut();
        for next_index in range.values() {
            let index = match next_index.to_usize() {
                Option::None => return Self::size_error(runtime, &next_index),
                Option::Some(val) => val,
            };
            let next_value = match value_iter.next(runtime)?.take_first() {
                Option::Some(v) => v,
                Option::None => {
                    // If there are extra values left on the range after the iterator has been
                    // iterated, drain the rest of the array
                    let end = min(range_end, array.len());
                    array.drain(index..end);
                    return runtime.return_0();
                }
            };
            if index >= array.len() {
                array.push(next_value);
            } else {
                array[index] = next_value;
            }
        }
        // If there are values left on the iterable after the range has been iterated, put them in
        while let Option::Some(val) = value_iter.next(runtime)?.take_first() {
            let arr_len = array.len();
            let end = min(range_end, arr_len);
            if end >= arr_len {
                array.push(val);
            } else {
                array.insert(end, val);
            }
        }
        runtime.return_0()
    }

    fn del_slice(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let range = self.slice_to_range(runtime, first(args))?;
        if !range.get_step().is_one() {
            return runtime.throw_quick(
                value_error(),
                format!(
                    "list.operator del[:] is only valid with a slice step of 1, not {}",
                    range.get_step()
                ),
            );
        }
        let range_start = range.get_start().to_usize().unwrap();
        let range_end = range.get_stop().to_usize().unwrap_or(usize::MAX);
        let len = self.len();
        self.value
            .borrow_mut()
            .drain(range_start..min(range_end, len));

        runtime.return_0()
    }

    fn iter(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(ListIter::new(self)).into())
    }

    fn iter_slice(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let range = self.slice_to_range(runtime, first(args))?;
        let value = self.value.borrow();
        let len = value.len();
        let start = match range.get_start().to_usize() {
            Option::Some(v) => v,
            Option::None => return Self::size_error(runtime, range.get_start()),
        };
        let stop = min(range.get_stop().to_usize().unwrap_or(len), len);
        let step = match range.get_step().to_usize() {
            Option::Some(v) => v,
            Option::None => return Self::size_error(runtime, range.get_step()),
        };
        let new_vec = List::from_values(
            self.generic,
            value[start..stop].iter().step_by(step).cloned().collect(),
        );
        runtime.return_1(Rc::new(ListIter::new(new_vec)).into())
    }

    pub fn insert(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let (index, param) = first_two(args);
        let index = index.int(runtime)?;
        let mut value = self.value.borrow_mut();
        if index == value.len().into() {
            value.push(param);
            runtime.return_0()
        } else {
            match normalize(value.len(), index) {
                Result::Ok(i) => {
                    if i <= value.len() {
                        value.insert(i, param);
                        runtime.return_0()
                    } else {
                        runtime.throw_quick(
                            index_error(),
                            format!(
                                "Index {} out of bounds for insert on list of length {}",
                                i,
                                value.len()
                            ),
                        )
                    }
                }
                Result::Err(i) => runtime.throw_quick(
                    index_error(),
                    format!(
                        "Index {} out of bounds for insert on list of length {}",
                        i,
                        value.len()
                    ),
                ),
            }
        }
    }

    pub fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty()); // TODO: List of a value
        runtime.return_1(List::from_values(Type::Object, vec![]).into())
    }

    pub fn list_type() -> Type {
        custom_class!(List, create, "list")
    }

    fn slice_to_range(&self, runtime: &mut Runtime, arg: Variable) -> Result<Rc<Range>, ()> {
        Range::from_slice(self.len(), runtime, arg)
    }

    fn size_error(runtime: &mut Runtime, size: &IntVar) -> FnResult {
        runtime.throw_quick(
            value_error(),
            format!(
                "Index {} too large (must be less than {})",
                size,
                usize::MAX
            ),
        )
    }
}

impl CustomVar for List {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Operator(o) => self.get_operator(o),
            Name::Attribute(s) => self.get_attribute(s),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        List::list_type()
    }

    fn call_op(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(List::op_fn(operator), self, args)
    }

    fn call_op_or_goto(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(List::op_fn(operator), self, args)
    }

    fn str(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        let value = join_values(&**self.value.borrow(), |x| x.str(runtime))?;
        Result::Ok(format!("[{}]", value).into())
    }

    fn repr(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        let value = join_values(&**self.value.borrow(), |x| x.repr(runtime))?;
        Result::Ok(format!("[{}]", value).into())
    }

    fn bool(self: Rc<Self>, _runtime: &mut Runtime) -> Result<bool, ()> {
        Result::Ok(!self.value.borrow().is_empty())
    }

    fn iter(self: Rc<Self>, _runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        Result::Ok(Rc::new(ListIter::new(self)).into())
    }
}

#[derive(Debug)]
struct ListIter {
    current: Cell<usize>,
    value: Rc<List>,
}

#[derive(Debug)]
struct ListRevIter {
    current: Cell<usize>,
    value: Rc<List>,
}

impl ListIter {
    pub fn new(value: Rc<List>) -> ListIter {
        ListIter {
            current: Cell::new(0),
            value,
        }
    }

    iter_internals!();

    fn inner_next(&self) -> Option<Variable> {
        if self.current.get() != self.value.value.borrow().len() {
            let result = self.value.value.borrow()[self.current.get()].clone();
            self.current.set(self.current.get() + 1);
            Option::Some(result)
        } else {
            Option::None
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }

    fn range_iter_type() -> Type {
        custom_class!(ListIter, create, "ListIter")
    }
}

impl ListRevIter {
    pub fn new(value: Rc<List>) -> ListRevIter {
        ListRevIter {
            current: Cell::new(value.len()),
            value,
        }
    }

    iter_internals!();

    fn inner_next(&self) -> Option<Variable> {
        if self.current.get() != 0 {
            self.current.set(self.current.get() + 1);
            let result = self.value.value.borrow()[self.current.get()].clone();
            Option::Some(result)
        } else {
            Option::None
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }

    fn range_iter_type() -> Type {
        custom_class!(ListRevIter, create, "ListRevIter")
    }
}

impl CustomVar for ListIter {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        default_attr!(self, name)
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::range_iter_type()
    }

    fn into_iter(self: Rc<Self>) -> looping::Iterator {
        looping::Iterator::Native(self)
    }
}

impl NativeIterator for ListIter {
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.inner_next().into())
    }
}

impl CustomVar for ListRevIter {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        default_attr!(self, name)
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::range_iter_type()
    }

    fn into_iter(self: Rc<Self>) -> looping::Iterator {
        looping::Iterator::Native(self)
    }
}

impl NativeIterator for ListRevIter {
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.inner_next().into())
    }
}
