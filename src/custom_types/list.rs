use crate::custom_types::exceptions::{index_error, value_error};
use crate::custom_types::range::Range;
use crate::custom_var::{downcast_var, CustomVar};
use crate::int_var::{normalize, IntVar};
use crate::looping;
use crate::looping::{IterResult, NativeIterator};
use crate::method::{InnerMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use num::{One, Signed, ToPrimitive, Zero};
use std::cell::{Cell, RefCell};
use std::cmp::min;
use std::iter::repeat_with;
use std::mem::take;
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

    fn get_operator(self: Rc<Self>, name: Operator) -> Variable {
        let value = match name {
            Operator::Bool => List::list_bool,
            Operator::Str => List::list_str,
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
        };
        Box::new(StdMethod::new(self, InnerMethod::Native(value))).into()
    }

    fn get_attribute(self: Rc<Self>, name: StringVar) -> Variable {
        let value = match name.as_str() {
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

    fn list_bool(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1((!self.value.borrow().is_empty()).into())
    }

    fn list_str(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let mut value: String = String::new();
        value += "[";
        for arg in self.value.borrow().iter().enumerate() {
            value += arg.1.clone().str(runtime)?.as_str();
            if arg.0 != self.value.borrow().len() - 1 {
                value += ", ";
            }
        }
        value += "]";
        runtime.return_1(value.into())
    }

    fn list_index(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let val = self.value.borrow();
        match normalize(val.len(), take(&mut args[0]).into()) {
            Result::Ok(index) => runtime.return_1(val[index].clone()),
            Result::Err(index) => self.index_error(index, runtime),
        }
    }

    fn set_index(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        match normalize(self.value.borrow().len(), take(&mut args[0]).into()) {
            Result::Ok(index) => {
                if args[1].get_type().is_subclass(&self.generic, runtime) {
                    self.value.borrow_mut()[index] = take(&mut args[1]);
                } else {
                    panic!("Bad type for list.operator []=")
                }
                runtime.return_0()
            }
            Result::Err(index) => self.index_error(index, runtime),
        }
    }

    fn list_get(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let val = self.value.borrow();
        if args.len() == 1 {
            runtime.return_1(match normalize(val.len(), take(&mut args[0]).into()) {
                Result::Ok(index) => Option::Some(val[index].clone()).into(),
                Result::Err(_) => Option::None.into(),
            })
        } else {
            debug_assert_eq!(args.len(), 2);
            runtime.return_1(match normalize(val.len(), take(&mut args[0]).into()) {
                Result::Ok(index) => val[index].clone(),
                Result::Err(_) => take(&mut args[1]),
            })
        }
    }

    fn plus(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let iter = take(&mut args[0]).iter(runtime)?;
        let mut new = Vec::new();
        while let Option::Some(val) = iter.next(runtime)? {
            if !val.get_type().is_subclass(&self.generic, runtime) {
                panic!(
                    "Bad type for list[{}].addAll: {}\n{}",
                    self.generic.str(),
                    val.get_type().str(),
                    runtime.stack_frames(),
                )
            } else {
                new.push(val)
            }
        }
        runtime.return_1(List::from_values(self.generic, new).into())
    }

    fn times(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let times = IntVar::from(take(&mut args[0]));
        if times.is_negative() {
            return runtime.throw_quick(
                value_error(),
                format!(
                    "Cannot multiply list: Expected non-negative number, got {}",
                    times
                )
                .into(),
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
                    let new = repeat_with(|| values.clone()).take(x).flatten().collect();
                    runtime.return_1(List::from_values(self.generic, new).into())
                }
                Option::None => runtime.throw_quick(
                    value_error(),
                    format!("List repetition {} too big to fit in memory", times).into(),
                ),
            }
        }
    }

    fn pop(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.value.borrow_mut().pop().into())
    }

    fn pop_first(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        if self.value.borrow().is_empty() {
            runtime.return_1(Option::None.into())
        } else {
            runtime.return_1(Option::Some(self.value.borrow_mut().remove(0)).into())
        }
    }

    fn swap(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let len = self.value.borrow().len();
        match normalize(len, take(&mut args[0]).into()) {
            Result::Ok(i1) => match normalize(len, take(&mut args[1]).into()) {
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
            )
            .into(),
        )
    }

    fn contains(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        runtime.return_1(self.value.borrow().contains(&args[0]).into())
    }

    fn contains_all(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let checked_var = take(&mut args[0]);
        let this_iter = checked_var.iter(runtime)?;
        while let Option::Some(val) = this_iter.next(runtime)? {
            if !self.value.borrow().contains(&val) {
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn index_of(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let searcher = take(&mut args[0]);
        for (i, var) in self.value.borrow().iter().enumerate() {
            if searcher.equals(var.clone(), runtime)? {
                return runtime.return_1(Option::Some(IntVar::from(i).into()).into());
            }
        }
        runtime.return_1(Option::None.into())
    }

    fn reversed(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let mut val = self.value.borrow().clone();
        val.reverse();
        runtime.return_1(List::from_values(self.generic, val).into())
    }

    fn reverse(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.value.borrow_mut().reverse();
        runtime.return_0()
    }

    fn count(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let mut count: usize = 0;
        for x in &*self.value.borrow() {
            if x.equals(args[0].clone(), runtime)? {
                count += 1;
            }
        }
        runtime.return_1(IntVar::from(count).into())
    }

    fn clear(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.value.borrow_mut().clear();
        runtime.return_0()
    }

    fn add(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        if !args[0].get_type().is_subclass(&self.generic, runtime) {
            panic!("Bad type for list.add\n{}", runtime.stack_frames())
        }
        self.value.borrow_mut().push(take(&mut args[0]));
        runtime.return_0()
    }

    fn add_all(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let iterator = take(&mut args[0]).iter(runtime)?;
        let mut value = self.value.borrow_mut();
        while let Option::Some(val) = iterator.next(runtime)? {
            if !val.get_type().is_subclass(&self.generic, runtime) {
                panic!(
                    "Bad type for list[{}].addAll: {}\n{}",
                    self.generic.str(),
                    val.get_type().str(),
                    runtime.stack_frames(),
                )
            }
            value.push(val);
        }
        runtime.return_0()
    }

    fn eq(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
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
            if !a.equals(b.clone(), runtime)? {
                is_eq = false;
                break;
            }
        }
        Result::Ok(is_eq)
    }

    fn get_slice(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let range = self.slice_to_range(runtime, take(&mut args[0]))?;
        if range.get_step().is_one() {
            let value = self.value.borrow();
            let start = range.get_start().to_usize().unwrap();
            let stop = range.get_start().to_usize().unwrap_or(usize::MAX);
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

    fn set_slice(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let range = self.slice_to_range(runtime, take(&mut args[0]))?;
        if !range.get_step().is_one() {
            return runtime.throw_quick(
                value_error(),
                format!(
                    "list.operator [:]= is only valid with a slice step of 1, not {}",
                    range.get_step()
                )
                .into(),
            );
        }
        let range_end = range.get_stop().to_usize().unwrap_or(usize::MAX);
        let value_iter = take(&mut args[1]).iter(runtime)?;
        let mut array = self.value.borrow_mut();
        for next_index in range.values() {
            let index = match next_index.to_usize() {
                Option::None => return Self::size_error(runtime, &next_index),
                Option::Some(val) => val,
            };
            let next_value = match value_iter.next(runtime)? {
                Option::Some(v) => v,
                Option::None => {
                    // If there are extra values left on the range after the iterator has been
                    // iterated, drain the rest of the array
                    let end = if range_end > array.len() {
                        array.len()
                    } else {
                        range_end
                    };
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
        while let Option::Some(val) = value_iter.next(runtime)? {
            let arr_len = array.len();
            let end = if range_end > arr_len {
                arr_len
            } else {
                range_end
            };
            if end >= arr_len {
                array.push(val);
            } else {
                array.insert(end, val);
            }
        }
        runtime.return_0()
    }

    fn del_slice(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let range = self.slice_to_range(runtime, take(&mut args[0]))?;
        if !range.get_step().is_one() {
            return runtime.throw_quick(
                value_error(),
                format!(
                    "list.operator del[:] is only valid with a slice step of 1, not {}",
                    range.get_step()
                )
                .into(),
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

    fn iter(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(ListIter::new(self.clone())).into())
    }

    fn iter_slice(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let range = self.slice_to_range(runtime, take(&mut args[0]))?;
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

    pub fn insert(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let index = take(&mut args[0]).int(runtime)?;
        let mut value = self.value.borrow_mut();
        if index == value.len().into() {
            value.push(take(&mut args[1]));
            runtime.return_0()
        } else {
            match normalize(value.len(), index) {
                Result::Ok(i) => {
                    value.insert(i, take(&mut args[1]));
                    runtime.return_0()
                }
                Result::Err(i) => runtime.throw_quick(
                    index_error(),
                    format!(
                        "Index {} out of bounds for insert on list of length {}",
                        i,
                        value.len()
                    )
                    .into(),
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

    fn slice_to_range(
        self: &Rc<Self>,
        runtime: &mut Runtime,
        arg: Variable,
    ) -> Result<Rc<Range>, ()> {
        Range::from_slice(self.len(), runtime, arg)
    }

    fn size_error(runtime: &mut Runtime, size: &IntVar) -> FnResult {
        runtime.throw_quick(
            value_error(),
            format!(
                "Index {} too large (must be less than {})",
                size,
                usize::MAX
            )
            .into(),
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

    fn get_type(self: Rc<Self>) -> Type {
        List::list_type()
    }
}

#[derive(Debug)]
struct ListIter {
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

impl CustomVar for ListIter {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        name.do_each(|o| self.get_op(o), |s| self.get_attribute(s))
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Self::range_iter_type()
    }

    fn into_iter(self: Rc<Self>) -> looping::Iterator {
        looping::Iterator::Native(self)
    }
}

impl NativeIterator for ListIter {
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.inner_next())
    }
}
