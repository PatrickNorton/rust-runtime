use crate::custom_types::exceptions::{index_error, value_error};
use crate::custom_var::CustomVar;
use crate::int_tools::{bytes_index, bytes_index_le};
use crate::int_var::{normalize, IntVar};
use crate::looping;
use crate::looping::{IterResult, NativeIterator};
use crate::method::StdMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use num::ToPrimitive;
use std::cell::{Cell, RefCell};
use std::char;
use std::mem::take;
use std::rc::Rc;

#[derive(Debug)]
pub struct LangBytes {
    value: RefCell<Vec<u8>>,
}

#[derive(Debug)]
struct BytesIter {
    current: Cell<usize>,
    value: Rc<LangBytes>,
}

impl LangBytes {
    pub fn new(val: Vec<u8>) -> LangBytes {
        LangBytes {
            value: RefCell::new(val),
        }
    }

    fn get_op(self: Rc<Self>, op: Operator) -> Variable {
        let func = match op {
            Operator::GetAttr => Self::index,
            Operator::SetAttr => Self::set_index,
            Operator::Str => Self::str,
            Operator::Iter => Self::iter,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self, func))
    }

    fn get_attribute(self: Rc<Self>, attr: StringVar) -> Variable {
        let func = match attr.as_str() {
            "length" => return IntVar::from(self.value.borrow().len()).into(),
            "encode" => Self::encode,
            "join" => Self::join,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self, func))
    }

    fn index(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        match normalize(self.value.borrow().len(), take(&mut args[0]).into()) {
            Result::Ok(index) => runtime.return_1(IntVar::from(self.value.borrow()[index]).into()),
            Result::Err(index) => self.index_err(index, runtime),
        }
    }

    fn set_index(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        match normalize(self.value.borrow().len(), take(&mut args[0]).into()) {
            Result::Ok(index) => {
                let value = IntVar::from(take(&mut args[1]));
                self.value.borrow_mut()[index] = match value.to_u8() {
                    Option::Some(i) => i,
                    Option::None => return Self::shrink_err(value, runtime)?,
                };
                runtime.return_0()
            }
            Result::Err(index) => self.index_err(index, runtime),
        }
    }

    fn str(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.encode(args, runtime)
    }

    fn encode(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let encoding_type = if args.is_empty() {
            "utf-8".into()
        } else {
            StringVar::from(take(&mut args[1]))
        };
        let result = StringVar::from(match encoding_type.as_str() {
            "utf-8" => String::from_utf8(self.value.borrow().clone())
                .or_else(|_| self.utf8_err(runtime))?,
            "utf-16" => self.convert_utf16(runtime)?,
            "utf-16be" => self.convert_utf16_be(runtime)?,
            "utf-32" => self.convert_utf32(runtime)?,
            "utf-32be" => self.convert_utf32_be(runtime)?,
            _ => {
                return runtime.throw_quick(
                    value_error(),
                    format!("{} not a valid encoding", encoding_type).into(),
                )
            }
        })
        .into();
        runtime.return_1(result)
    }

    fn convert_utf16(&self, runtime: &mut Runtime) -> Result<String, ()> {
        String::from_utf16(
            &*self
                .value
                .borrow()
                .chunks(2)
                .map(|x| bytes_index_le(x, &mut 0))
                .collect::<Vec<_>>(),
        )
        .or_else(|_| {
            runtime
                .throw_quick(
                    value_error(),
                    "Invalid byte literal for big-endian utf-16 conversion".into(),
                )
                .and_then(|_| unreachable!())
        })
    }

    fn convert_utf16_be(&self, runtime: &mut Runtime) -> Result<String, ()> {
        String::from_utf16(
            &*self
                .value
                .borrow()
                .chunks(2)
                .map(|x| bytes_index(x, &mut 0))
                .collect::<Vec<_>>(),
        )
        .or_else(|_| {
            runtime
                .throw_quick(
                    value_error(),
                    "Invalid byte literal for big-endian utf-16 conversion".into(),
                )
                .and_then(|_| unreachable!())
        })
    }

    fn convert_utf32(&self, runtime: &mut Runtime) -> Result<String, ()> {
        self.value
            .borrow()
            .chunks(4)
            .map(|x| match char::from_u32(bytes_index_le(x, &mut 0)) {
                Option::Some(value) => Result::Ok(value),
                Option::None => runtime
                    .throw_quick(
                        value_error(),
                        "Invalid byte literal for utf-32 conversion".into(),
                    )
                    .and_then(|_| unreachable!()),
            })
            .collect::<Result<String, ()>>()
    }

    fn convert_utf32_be(&self, runtime: &mut Runtime) -> Result<String, ()> {
        self.value
            .borrow()
            .chunks(4)
            .map(|x| match char::from_u32(bytes_index(x, &mut 0)) {
                Option::Some(value) => Result::Ok(value),
                Option::None => runtime
                    .throw_quick(
                        value_error(),
                        "Invalid byte literal for big-endian utf-32 conversion".into(),
                    )
                    .and_then(|_| unreachable!()),
            })
            .collect::<Result<String, ()>>()
    }

    fn index_err(&self, index: IntVar, runtime: &mut Runtime) -> FnResult {
        runtime.throw_quick(
            index_error(),
            format!(
                "index {} out of range for list of length {}",
                index,
                self.value.borrow().len()
            )
            .into(),
        )
    }

    fn utf8_err<T>(&self, runtime: &mut Runtime) -> Result<T, ()> {
        runtime.throw_quick(
            value_error(),
            "Invalid byte literal for utf-8 conversion".into(),
        )?;
        unreachable!()
    }

    fn shrink_err<T>(index: IntVar, runtime: &mut Runtime) -> Result<T, ()> {
        runtime.throw_quick(
            value_error(),
            format!(
                "{} is too big to fit in a byte (must be in [-255:255])",
                index
            )
            .into(),
        )?;
        unreachable!()
    }

    fn iter(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(BytesIter::new(self.clone())).into())
    }

    fn join(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let mut is_first = true;
        let mut result = Vec::new();
        let iter = take(&mut args[0]).iter(runtime)?;
        while let Option::Some(val) = iter.next(runtime)? {
            if !is_first {
                result.extend_from_slice(&self.value.borrow());
            }
            is_first = false;
            result.extend_from_slice(val.str(runtime)?.as_bytes());
        }
        runtime.return_1(Rc::new(LangBytes::new(result)).into())
    }

    fn create(mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let mut result = Vec::new();
        let iter = take(&mut args[0]).iter(runtime)?;
        while let Option::Some(val) = iter.next(runtime)? {
            result.push(IntVar::from(val).to_u8().unwrap());
        }
        runtime.return_1(Rc::new(LangBytes::new(result)).into())
    }

    pub fn bytes_type() -> Type {
        custom_class!(LangBytes, create, "bytes")
    }
}

impl CustomVar for LangBytes {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        match name {
            Name::Operator(op) => self.get_op(op),
            Name::Attribute(val) => self.get_attribute(val),
        }
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Self::bytes_type()
    }
}

impl BytesIter {
    pub fn new(value: Rc<LangBytes>) -> BytesIter {
        BytesIter {
            current: Cell::new(0),
            value,
        }
    }

    fn get_attribute(self: &Rc<Self>, val: StringVar) -> Variable {
        let func = match val.as_str() {
            "next" => Self::next_fn,
            _ => unimplemented!(),
        };
        Variable::Method(StdMethod::new_native(self.clone(), func))
    }

    fn next_fn(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.inner_next().into())
    }

    fn inner_next(&self) -> Option<Variable> {
        if self.current.get() != self.value.value.borrow().len() {
            let result = self.value.value.borrow()[self.current.get()];
            self.current.set(self.current.get() + 1);
            Option::Some(IntVar::from(result).into())
        } else {
            Option::None
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }

    fn bytes_iter_type() -> Type {
        custom_class!(BytesIter, create, "BytesIter")
    }
}

impl CustomVar for BytesIter {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        name.do_each(|_| unimplemented!(), |s| self.get_attribute(s))
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(self: Rc<Self>) -> Type {
        Self::bytes_iter_type()
    }

    fn into_iter(self: Rc<Self>) -> looping::Iterator {
        looping::Iterator::Native(self)
    }
}

impl NativeIterator for BytesIter {
    fn next(self: Rc<Self>, _runtime: &mut Runtime) -> IterResult {
        IterResult::Ok(self.inner_next())
    }
}
