use crate::custom_types::exceptions::{arithmetic_error, index_error, value_error};
use crate::custom_var::{downcast_var, CustomVar};
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
use num::{BigInt, ToPrimitive};
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
            Operator::Repr => Self::repr,
            Operator::Iter => Self::iter,
            Operator::Bool => Self::bool,
            Operator::Add => Self::plus,
            Operator::Multiply => Self::mul,
            _ => unimplemented!("bytes.{}", op.name()),
        };
        StdMethod::new_native(self, func).into()
    }

    fn get_attribute(self: Rc<Self>, attr: StringVar) -> Variable {
        let func = match attr.as_str() {
            "length" => return IntVar::from(self.value.borrow().len()).into(),
            "encode" => Self::encode,
            "join" => Self::join,
            "indexOf" => Self::index_of,
            "get" => Self::get,
            "add" => Self::add,
            "addChar" => Self::add_char,
            _ => unimplemented!(),
        };
        StdMethod::new_native(self, func).into()
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
        let len = self.value.borrow().len();
        match normalize(len, take(&mut args[0]).into()) {
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

    fn repr(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let val = format!(
            "{:?}",
            String::from_utf8(self.value.borrow().clone()).or_else(|_| self.utf8_err(runtime))?
        );
        runtime.return_1(StringVar::from(val).into())
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
                    "Invalid byte literal for little-endian utf-16 conversion".into(),
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
            .collect()
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
            .collect()
    }

    fn get(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let value = IntVar::from(take(&mut args[0]))
            .to_usize()
            .and_then(|x| self.value.borrow().get(x).copied())
            .map(|x| IntVar::Small(x as isize).into())
            .into();
        runtime.return_1(value)
    }

    fn add(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let int_val = IntVar::from(take(&mut args[0]));
        if let Option::Some(value) = int_val.to_u8() {
            self.value.borrow_mut().push(value);
            runtime.return_0()
        } else {
            runtime.throw_quick(
                value_error(),
                format!(
                    "Value added to bytes must be in [-255:256], not {}",
                    int_val
                )
                .into(),
            )
        }
    }

    fn add_char(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let char_val = take(&mut args[0]).into();
        match take(&mut args[0]).str(runtime)?.as_str() {
            "utf-8" => self.add_utf8(char_val),
            "utf-16" => self.add_utf16(char_val),
            "utf-16be" => self.add_utf16be(char_val),
            "utf-32" => self.add_utf32(char_val),
            "utf-32be" => self.add_utf32be(char_val),
            x => {
                return runtime.throw_quick(
                    value_error(),
                    format!("{} is not a valid encoding", x).into(),
                )
            }
        };
        runtime.return_0()
    }

    fn add_utf8(self: &Rc<Self>, value: char) {
        self.value
            .borrow_mut()
            .extend_from_slice(value.encode_utf8(&mut [0; 4]).as_bytes())
    }

    // These two can probably be improved with #![feature(array_value_iter)]
    // val.extend(value.encode_utf16(&mut [0; 2]).flat_map(u16::to_le_bytes))

    fn add_utf16(self: &Rc<Self>, value: char) {
        let mut val = self.value.borrow_mut();
        value
            .encode_utf16(&mut [0; 2])
            .iter()
            .for_each(|x| val.extend_from_slice(&x.to_le_bytes()));
    }

    fn add_utf16be(self: &Rc<Self>, value: char) {
        let mut val = self.value.borrow_mut();
        value
            .encode_utf16(&mut [0; 2])
            .iter()
            .for_each(|x| val.extend_from_slice(&x.to_be_bytes()));
    }

    fn add_utf32(self: &Rc<Self>, value: char) {
        self.value
            .borrow_mut()
            .extend(&(value as u32).to_le_bytes())
    }

    fn add_utf32be(self: &Rc<Self>, value: char) {
        self.value
            .borrow_mut()
            .extend(&(value as u32).to_be_bytes())
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

    fn bool(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1((!self.value.borrow().is_empty()).into())
    }

    fn plus(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let mut result = self.value.borrow().clone();
        for value in args {
            let val = downcast_var::<LangBytes>(value).expect("Invalid type to bytes.+");
            result.extend(&*val.value.borrow());
        }
        runtime.return_1(Rc::new(LangBytes::new(result)).into())
    }

    fn mul(self: &Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        if self.value.borrow().is_empty() {
            return runtime.return_1(Rc::new(LangBytes::new(Vec::new())).into());
        }
        let mut result = self.value.borrow().clone();
        for arg in args {
            let big_val = IntVar::from(arg);
            match big_val.to_usize() {
                Option::Some(val) => match val.checked_mul(result.len()) {
                    Option::Some(_) => result = result.repeat(val),
                    Option::None => {
                        return runtime
                            .throw_quick(arithmetic_error(), overflow_exc(val, result.len()))
                    }
                },
                Option::None => {
                    return runtime.throw_quick(arithmetic_error(), "Cannot multiply".into())
                }
            }
        }
        runtime.return_1(Rc::new(LangBytes::new(result)).into())
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

    fn index_of(self: &Rc<Self>, mut args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let search_int = IntVar::from(take(&mut args[0]));
        runtime.return_1(Variable::from(match search_int.to_u8() {
            Option::Some(i) => self
                .value
                .borrow()
                .iter()
                .enumerate()
                .find(|x| *x.1 == i)
                .map(|x| x.0)
                .map(IntVar::from)
                .map(Variable::from),
            Option::None => Option::None,
        }))
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

    fn get_type(&self) -> Type {
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
        StdMethod::new_native(self.clone(), func).into()
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

fn overflow_exc(val: usize, len: usize) -> StringVar {
    format!(
        "Too many string repetitions: maximum bytes length is {}, \
        but repetition would produce bytes of length {}",
        usize::MAX,
        BigInt::from(val) * len
    )
    .into()
}

impl CustomVar for BytesIter {
    fn get_attr(self: Rc<Self>, name: Name) -> Variable {
        name.do_each(|_| unimplemented!(), |s| self.get_attribute(s))
    }

    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
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
