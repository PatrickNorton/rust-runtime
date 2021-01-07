use crate::custom_types::exceptions::{index_error, value_error};
use crate::custom_types::range::Range;
use crate::custom_var::{downcast_var, CustomVar};
use crate::int_var::IntVar;
use crate::looping;
use crate::looping::{IterResult, NativeIterator};
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::variable::{FnResult, Variable};
use num::{Signed, ToPrimitive};
use std::cell::{Cell, RefCell};
use std::cmp::min;
use std::mem::take;
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

    fn get_operator(self: Rc<Self>, name: Operator) -> Variable {
        let func = match name {
            Operator::GetAttr => Self::index,
            Operator::SetAttr => Self::set_index,
            Operator::Bool => Self::bool,
            Operator::Str => Self::str,
            Operator::Equals => Self::eq,
            Operator::In => Self::contains,
            Operator::GetSlice => Self::get_slice,
            Operator::Iter => Self::iter,
            Operator::IterSlice => Self::iter_slice,
            _ => unimplemented!("Array.{}", name.name()),
        };
        StdMethod::new_native(self, func).into()
    }

    fn index(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let signed_index = IntVar::from(args[0].clone());
        let index = if signed_index.is_negative() {
            signed_index + self.vars.borrow().len().into()
        } else {
            signed_index
        };
        if index >= self.vars.borrow().len().into() || index.is_negative() {
            runtime.throw_quick(
                index_error(),
                format!(
                    "index {} out of range for array of length {}",
                    index,
                    self.vars.borrow().len()
                )
                .into(),
            )
        } else {
            runtime.return_1(self.vars.borrow()[index.to_usize().unwrap()].clone())
        }
    }

    fn set_index(self: Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let index = self.normalize_index(IntVar::from(take(&mut args[0])));
        let value = take(&mut args[1]);
        match index {
            Option::Some(val) => self.vars.borrow_mut()[val] = value,
            Option::None => {
                return runtime.throw_quick(index_error(), "Array index out of bounds".into())
            }
        }
        runtime.return_0()
    }

    fn bool(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1((!self.vars.borrow().is_empty()).into())
    }

    fn str(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let mut value = String::new();
        value += "Array[";
        for arg in self.vars.borrow().iter().enumerate() {
            value += arg.1.clone().str(runtime)?.as_str();
            if arg.0 != self.vars.borrow().len() - 1 {
                value += ", ";
            }
        }
        value += "]";
        runtime.return_1(value.into())
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

    fn contains(self: Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let arg = take(&mut args[0]);
        for val in self.vars.borrow().iter() {
            if arg.equals(val.clone(), runtime)? {
                return runtime.return_1(true.into());
            }
        }
        runtime.return_1(false.into())
    }

    fn get_slice(self: Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let range = Range::from_slice(self.vars.borrow().len(), runtime, take(&mut args[0]))?;
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

    fn iter_slice(self: Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let range = Range::from_slice(self.vars.borrow().len(), runtime, take(&mut args[0]))?;
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
            if !a.equals(b.clone(), runtime)? {
                return Result::Ok(false);
            }
        }
        Result::Ok(true)
    }

    fn create(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let len = IntVar::from(take(&mut args[0]));
        let usize_len = match len.to_usize() {
            Option::Some(val) => val,
            Option::None => {
                return runtime.throw_quick(value_error(), "Array init too large to store".into())
            }
        };
        let fill = take(&mut args[1]);
        let vars = RefCell::new(vec![fill; usize_len].into_boxed_slice());
        runtime.return_1(Rc::new(Array { vars }).into())
    }

    fn normalize_index(&self, signed_index: IntVar) -> Option<usize> {
        let len = self.vars.borrow().len();
        let index = if signed_index.is_negative() {
            signed_index + len.into()
        } else {
            signed_index
        };
        index.to_usize().and_then(|a| {
            if a < len {
                Option::Some(a)
            } else {
                Option::None
            }
        })
    }

    pub fn array_type() -> Type {
        custom_class!(Array, create, "Array")
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

impl CustomVar for Array {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Operator(o) => self.get_operator(o),
            _ => unimplemented!(),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::array_type()
    }
}

impl ArrayIter {
    pub fn new(value: Rc<Array>) -> ArrayIter {
        ArrayIter {
            value,
            current: Cell::new(0),
        }
    }

    iter_internals!();

    fn inner_next(&self) -> Option<Variable> {
        if self.current.get() != self.value.vars.borrow().len() {
            let result = self.value.vars.borrow()[self.current.get()].clone();
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
        custom_class!(ArrayIter, create, "ArrayIter")
    }
}

impl CustomVar for ArrayIter {
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

impl NativeIterator for ArrayIter {
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.inner_next().into())
    }
}
