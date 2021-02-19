use crate::custom_types::exceptions::{index_error, value_error};
use crate::custom_types::join_values;
use crate::custom_types::range::Range;
use crate::custom_var::{downcast_var, CustomVar};
use crate::int_var::{normalize, IntVar};
use crate::looping::{self, TypicalIterator};
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::{MaybeString, StringVar};
use crate::variable::{FnResult, Variable};
use crate::{first, first_two};
use ascii::{AsciiChar, AsciiStr};
use num::ToPrimitive;
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};
use std::cmp::min;
use std::rc::Rc;

#[derive(Debug)]
pub struct Array {
    vars: RefCell<Box<[Variable]>>,
}

#[derive(Debug)]
struct ArrayIter {
    current: Cell<usize>,
    value: Rc<Array>,
}

impl Array {
    fn new(args: Box<[Variable]>) -> Rc<Array> {
        Rc::new(Array {
            vars: RefCell::new(args),
        })
    }

    fn op_fn(o: Operator) -> NativeMethod<Rc<Array>> {
        match o {
            Operator::GetAttr => Self::index,
            Operator::SetAttr => Self::set_index,
            Operator::Bool => Self::bool,
            Operator::Str => Self::str,
            Operator::Repr => Self::repr,
            Operator::Equals => Self::eq,
            Operator::In => Self::contains,
            Operator::GetSlice => Self::get_slice,
            Operator::Iter => Self::iter,
            Operator::IterSlice => Self::iter_slice,
            _ => unimplemented!("Array.{}", o.name()),
        }
    }

    fn index(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let values = self.vars.borrow();
        match normalize(values.len(), first(args).into()) {
            Result::Ok(i) => runtime.return_1(values[i].clone()),
            Result::Err(index) => Self::index_err(runtime, values.len(), &index),
        }
    }

    fn set_index(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let (index, value) = first_two(args);
        let mut vars = self.vars.borrow_mut();
        match normalize(vars.len(), IntVar::from(index)) {
            Result::Ok(val) => vars[val] = value,
            Result::Err(val) => return Self::index_err(runtime, vars.len(), &val),
        }
        runtime.return_0()
    }

    fn bool(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1((!self.vars.borrow().is_empty()).into())
    }

    fn str(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let value = self.str_value(runtime)?;
        runtime.return_1(value.into())
    }

    fn str_value(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        let value = join_values(&**self.vars.borrow(), |x| x.str(runtime))?;
        Result::Ok(Self::surround(value).into())
    }

    fn repr(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let value = self.repr_value(runtime)?;
        runtime.return_1(value.into())
    }

    fn repr_value(&self, runtime: &mut Runtime) -> Result<StringVar, ()> {
        let value = join_values(&**self.vars.borrow(), |x| x.repr(runtime))?;
        Result::Ok(Self::surround(value).into())
    }

    fn surround(mut str: MaybeString) -> MaybeString {
        static ARRAY: Lazy<&AsciiStr> = Lazy::new(|| AsciiStr::from_ascii("Array[").unwrap());
        str.insert_ascii_str(0, *ARRAY);
        str.push_ascii(AsciiChar::BracketClose);
        str
    }

    fn eq(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        for arg in args {
            if !match downcast_var::<Array>(arg) {
                Option::None => false,
                Option::Some(other) => {
                    let self_val = self.vars.borrow();
                    let other_val = other.vars.borrow();
                    self_val.len() == other_val.len()
                        && Self::arr_eq(&*self_val, &*other_val, runtime)?
                }
            } {
                return runtime.return_1(false.into());
            }
        }
        runtime.return_1(true.into())
    }

    fn contains(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let arg = first(args);
        for val in self.vars.borrow().iter() {
            if arg.clone().equals(val.clone(), runtime)? {
                return runtime.return_1(true.into());
            }
        }
        runtime.return_1(false.into())
    }

    fn get_slice(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let range = Range::from_slice(self.vars.borrow().len(), runtime, first(args))?;
        let mut raw_vec = Vec::new();
        let self_val = self.vars.borrow();
        for i in range.values() {
            raw_vec.push(self_val[i.to_usize().expect("Conversion error")].clone());
        }
        runtime.return_1(Self::new(raw_vec.into_boxed_slice()).into())
    }

    fn iter(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(ArrayIter::new(self)).into())
    }

    fn iter_slice(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let range = Range::from_slice(self.vars.borrow().len(), runtime, first(args))?;
        let value = self.vars.borrow();
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
        let new_vec = Array::new(value[start..stop].iter().step_by(step).cloned().collect());
        runtime.return_1(Rc::new(ArrayIter::new(new_vec)).into())
    }

    fn arr_eq(first: &[Variable], second: &[Variable], runtime: &mut Runtime) -> Result<bool, ()> {
        for (a, b) in first.iter().zip(second.iter()) {
            if !a.clone().equals(b.clone(), runtime)? {
                return Result::Ok(false);
            }
        }
        Result::Ok(true)
    }

    fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let (len, fill) = first_two(args);
        let len = IntVar::from(len);
        let usize_len = match len.to_usize() {
            Option::Some(val) => val,
            Option::None => {
                return runtime.throw_quick(value_error(), "Array init too large to store")
            }
        };
        runtime.return_1(Array::new(vec![fill; usize_len].into_boxed_slice()).into())
    }

    pub fn array_type() -> Type {
        custom_class!(Array, create, "Array")
    }

    fn index_err(runtime: &mut Runtime, len: usize, size: &IntVar) -> FnResult {
        runtime.throw_quick(
            index_error(),
            format!("index {} out of range for array of length {}", size, len),
        )
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

impl CustomVar for Array {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::array_type()
    }

    fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
        let func = Array::op_fn(op);
        StdMethod::new_native(self, func).into()
    }

    fn get_attribute(self: Rc<Self>, _name: &str) -> Variable {
        unimplemented!()
    }

    fn call_op(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(Array::op_fn(operator), self, args)
    }

    fn call_op_or_goto(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(Array::op_fn(operator), self, args)
    }

    fn str(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.str_value(runtime)
    }

    fn repr(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        self.repr_value(runtime)
    }

    fn bool(self: Rc<Self>, _runtime: &mut Runtime) -> Result<bool, ()> {
        Result::Ok(!self.vars.borrow().is_empty())
    }

    fn iter(self: Rc<Self>, _runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        Result::Ok(Rc::new(ArrayIter::new(self)).into())
    }
}

impl ArrayIter {
    pub fn new(value: Rc<Array>) -> ArrayIter {
        ArrayIter {
            value,
            current: Cell::new(0),
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }
}

impl TypicalIterator for ArrayIter {
    fn inner_next(&self) -> Option<Variable> {
        if self.current.get() != self.value.vars.borrow().len() {
            let result = self.value.vars.borrow()[self.current.get()].clone();
            self.current.set(self.current.get() + 1);
            Option::Some(result)
        } else {
            Option::None
        }
    }

    fn get_type() -> Type {
        custom_class!(ArrayIter, create, "ArrayIter")
    }
}
