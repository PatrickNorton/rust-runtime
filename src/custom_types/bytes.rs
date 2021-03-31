use crate::custom_types::exceptions::{arithmetic_error, index_error, value_error};
use crate::custom_var::{downcast_var, CustomVar};
use crate::int_tools::FromBytes;
use crate::int_var::{normalize, IntVar};
use crate::looping::{self, TypicalIterator};
use crate::method::{NativeMethod, StdMethod};
use crate::name::Name;
use crate::operator::Operator;
use crate::runtime::Runtime;
use crate::std_type::Type;
use crate::string_var::StringVar;
use crate::variable::{FnResult, Variable};
use crate::{first, first_n};
use ascii::{AsciiChar, AsciiString, IntoAsciiString};
use num::{BigInt, ToPrimitive};
use std::array::IntoIter;
use std::cell::{Cell, Ref, RefCell};
use std::char;
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

#[derive(Debug)]
struct BytesRevIter {
    current: Cell<usize>,
    value: Rc<LangBytes>,
}

impl LangBytes {
    pub fn new(val: Vec<u8>) -> LangBytes {
        LangBytes {
            value: RefCell::new(val),
        }
    }

    pub fn get_value(&self) -> Ref<'_, Vec<u8>> {
        self.value.borrow()
    }

    fn op_fn(op: Operator) -> NativeMethod<Rc<LangBytes>> {
        match op {
            Operator::GetAttr => Self::index,
            Operator::SetAttr => Self::set_index,
            Operator::Str => Self::str,
            Operator::Repr => Self::repr,
            Operator::Iter => Self::iter,
            Operator::Bool => Self::bool,
            Operator::Add => Self::plus,
            Operator::Multiply => Self::mul,
            _ => unimplemented!("bytes.{}", op.name()),
        }
    }

    fn attr_fn(attr: &str) -> NativeMethod<Rc<LangBytes>> {
        match attr {
            "encode" => Self::encode,
            "join" => Self::join,
            "indexOf" => Self::index_of,
            "get" => Self::get,
            "add" => Self::add,
            "addChar" => Self::add_char,
            "startsWith" => Self::starts_with,
            "endsWith" => Self::ends_with,
            "lastIndexOf" => Self::last_index_of,
            "hex" => Self::hex,
            _ => unimplemented!("bytes.{}", attr),
        }
    }

    fn index(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        match normalize(self.value.borrow().len(), first(args).into()) {
            Result::Ok(index) => runtime.return_1(IntVar::from(self.value.borrow()[index]).into()),
            Result::Err(index) => self.index_err(index, runtime),
        }
    }

    fn set_index(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let len = self.value.borrow().len();
        let [signed_index, value] = first_n(args);
        match normalize(len, signed_index.into()) {
            Result::Ok(index) => {
                let value = IntVar::from(value);
                self.value.borrow_mut()[index] = match value.to_u8() {
                    Option::Some(i) => i,
                    Option::None => return Self::shrink_err(value, runtime),
                };
                runtime.return_0()
            }
            Result::Err(index) => self.index_err(index, runtime),
        }
    }

    fn str(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        self.encode(args, runtime)
    }

    fn repr(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(self.repr_value().into())
    }

    fn repr_value(&self) -> StringVar {
        let value = self.value.borrow();
        let mut result = AsciiString::with_capacity(value.len());
        for chr in &*value {
            match AsciiChar::from_ascii(*chr) {
                Ok(ascii) => result.push(ascii),
                Err(_) => {
                    // Not quite confident enough in this to use from_ascii_unchecked here
                    result += &*AsciiString::from_ascii(format!("{:x}", chr))
                        .expect("Hex value of a u8 should always be valid ASCII")
                }
            }
        }
        result.into()
    }

    fn encode(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let encoding_type = if args.is_empty() {
            "utf-8".into()
        } else {
            StringVar::from(first(args))
        };
        let result = match encoding_type.to_lowercase().as_str() {
            "ascii" => StringVar::from(self.convert_ascii(runtime)?),
            "utf-8" => StringVar::from(
                String::from_utf8(self.value.borrow().clone())
                    .or_else(|_| self.utf8_err(runtime))?,
            ),
            "utf-16" => StringVar::from(self.convert_utf16(runtime)?),
            "utf-16be" => StringVar::from(self.convert_utf16_be(runtime)?),
            "utf-32" => StringVar::from(self.convert_utf32(runtime)?),
            "utf-32be" => StringVar::from(self.convert_utf32_be(runtime)?),
            _ => {
                return runtime.throw_quick(
                    value_error(),
                    format!("{} not a valid encoding", encoding_type),
                )
            }
        }
        .into();
        runtime.return_1(result)
    }

    fn convert_ascii(&self, runtime: &mut Runtime) -> Result<AsciiString, ()> {
        let value = self.value.borrow();
        match AsciiString::from_ascii(&**value) {
            Result::Ok(x) => Result::Ok(x),
            Result::Err(x) => {
                let error = x.ascii_error();
                // valid_up_to is perfectly fine to use as codepoints, b/c each ASCII character is
                // exactly one byte, and all the characters up to valid_up_to() are ASCII
                runtime.throw_quick_native(
                    value_error(),
                    format!(
                        "Cannot convert to ascii: byte at position {} (value {}) is not in the range [0:128]", 
                        error.valid_up_to(), value[error.valid_up_to()]
                    )
                )
            }
        }
    }

    // #![feature(array_chunks)] will make this much nicer (#74985)

    fn convert_utf16(&self, runtime: &mut Runtime) -> Result<String, ()> {
        String::from_utf16(
            &*self
                .value
                .borrow()
                .chunks(2)
                .map(FromBytes::from_le)
                .collect::<Vec<_>>(),
        )
        .or_else(|_| {
            runtime.throw_quick_native(
                value_error(),
                "Invalid byte literal for little-endian utf-16 conversion",
            )
        })
    }

    fn convert_utf16_be(&self, runtime: &mut Runtime) -> Result<String, ()> {
        String::from_utf16(
            &*self
                .value
                .borrow()
                .chunks(2)
                .map(FromBytes::from_le)
                .collect::<Vec<_>>(),
        )
        .or_else(|_| {
            runtime.throw_quick_native(
                value_error(),
                "Invalid byte literal for big-endian utf-16 conversion",
            )
        })
    }

    fn convert_utf32(&self, runtime: &mut Runtime) -> Result<String, ()> {
        self.value
            .borrow()
            .chunks(4)
            .map(|x| match char::from_u32(FromBytes::from_le(x)) {
                Option::Some(value) => Result::Ok(value),
                Option::None => runtime.throw_quick_native(
                    value_error(),
                    "Invalid byte literal for utf-32 conversion",
                ),
            })
            .collect()
    }

    fn convert_utf32_be(&self, runtime: &mut Runtime) -> Result<String, ()> {
        self.value
            .borrow()
            .chunks(4)
            .map(|x| match char::from_u32(FromBytes::from_be(x)) {
                Option::Some(value) => Result::Ok(value),
                Option::None => runtime.throw_quick_native(
                    value_error(),
                    "Invalid byte literal for big-endian utf-32 conversion",
                ),
            })
            .collect()
    }

    fn get(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let value = IntVar::from(first(args))
            .to_usize()
            .and_then(|x| self.value.borrow().get(x).copied())
            .map(|x| IntVar::Small(x as isize).into())
            .into();
        runtime.return_1(value)
    }

    fn add(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let int_val = IntVar::from(first(args));
        if let Option::Some(value) = int_val.to_u8() {
            self.value.borrow_mut().push(value);
            runtime.return_0()
        } else {
            Self::shrink_err(int_val, runtime)
        }
    }

    fn add_char(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 2);
        let [char_val, encoding] = first_n(args);
        let char_val = char_val.into();
        match encoding.str(runtime)?.to_lowercase().as_str() {
            "ascii" => self.add_ascii(char_val, runtime)?,
            "utf-8" => self.add_utf8(char_val),
            "utf-16" => self.add_utf16(char_val),
            "utf-16be" => self.add_utf16be(char_val),
            "utf-32" => self.add_utf32(char_val),
            "utf-32be" => self.add_utf32be(char_val),
            x => {
                return runtime.throw_quick(value_error(), format!("{} is not a valid encoding", x))
            }
        };
        runtime.return_0()
    }

    fn starts_with(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let variable = first(args);
        let value = downcast_var::<LangBytes>(variable).expect("Expected bytes");
        let needle = value.value.borrow();
        runtime.return_1(self.value.borrow().starts_with(&needle).into())
    }

    fn ends_with(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let variable = first(args);
        let value = downcast_var::<LangBytes>(variable).expect("Expected bytes");
        let needle = value.value.borrow();
        runtime.return_1(self.value.borrow().ends_with(&needle).into())
    }

    fn add_ascii(&self, value: char, runtime: &mut Runtime) -> FnResult {
        match AsciiChar::from_ascii(value) {
            Result::Ok(ch) => self.value.borrow_mut().push(ch as u8),
            Result::Err(_) => runtime.throw_quick(
                value_error(),
                format!(
                    "Invalid ASCII character (value {} is not in [0:128])",
                    value as u32
                ),
            )?,
        }
        FnResult::Ok(())
    }

    fn add_utf8(&self, value: char) {
        self.value
            .borrow_mut()
            .extend_from_slice(value.encode_utf8(&mut [0; 4]).as_bytes())
    }

    fn add_utf16(&self, value: char) {
        self.value.borrow_mut().extend(
            value
                .encode_utf16(&mut [0; 2])
                .iter()
                .flat_map(|x| IntoIter::new(x.to_le_bytes())),
        );
    }

    fn add_utf16be(&self, value: char) {
        self.value.borrow_mut().extend(
            value
                .encode_utf16(&mut [0; 2])
                .iter()
                .flat_map(|x| IntoIter::new(x.to_be_bytes())),
        );
    }

    fn add_utf32(&self, value: char) {
        self.value
            .borrow_mut()
            .extend(&(value as u32).to_le_bytes())
    }

    fn add_utf32be(&self, value: char) {
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
            ),
        )
    }

    fn utf8_err<T>(&self, runtime: &mut Runtime) -> Result<T, ()> {
        runtime.throw_quick_native(value_error(), "Invalid byte literal for utf-8 conversion")
    }

    fn shrink_err(index: IntVar, runtime: &mut Runtime) -> FnResult {
        runtime.throw_quick(
            value_error(),
            format!("{} is too big to fit in a byte (must be in [0:256])", index),
        )
    }

    fn iter(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1(Rc::new(BytesIter::new(self)).into())
    }

    fn bool(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        runtime.return_1((!self.value.borrow().is_empty()).into())
    }

    fn plus(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let mut result = self.value.borrow().clone();
        for value in args {
            let val = downcast_var::<LangBytes>(value).expect("Invalid type to bytes.operator +");
            result.extend(&*val.value.borrow());
        }
        runtime.return_1(Rc::new(LangBytes::new(result)).into())
    }

    fn mul(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
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
                Option::None => return runtime.throw_quick(arithmetic_error(), "Cannot multiply"),
            }
        }
        runtime.return_1(Rc::new(LangBytes::new(result)).into())
    }

    fn join(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let mut is_first = true;
        let mut result = Vec::new();
        let iter = first(args).iter(runtime)?;
        while let Option::Some(val) = iter.next(runtime)?.take_first() {
            if !is_first {
                result.extend_from_slice(&self.value.borrow());
            }
            is_first = false;
            result.extend_from_slice(val.str(runtime)?.as_bytes());
        }
        runtime.return_1(Rc::new(LangBytes::new(result)).into())
    }

    fn index_of(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let search_int = IntVar::from(first(args));
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

    fn last_index_of(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.len() == 1);
        let search_int = IntVar::from(first(args));
        runtime.return_1(Variable::from(match search_int.to_u8() {
            Option::Some(i) => self
                .value
                .borrow()
                .iter()
                .rev()
                .enumerate()
                .find(|x| *x.1 == i)
                .map(|x| x.0)
                .map(IntVar::from)
                .map(Variable::from),
            Option::None => Option::None,
        }))
    }

    fn hex(self: Rc<Self>, args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert!(args.is_empty());
        let result = self
            .value
            .borrow()
            .iter()
            .map(|x| format!("{:x}", x).into_ascii_string().unwrap())
            .collect::<StringVar>();
        runtime.return_1(result.into())
    }

    fn from_hex(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        debug_assert_eq!(args.len(), 1);
        let str = StringVar::from(first(args));
        let mut result = Vec::with_capacity(str.len() / 2);
        for slice in str.chunks(2) {
            if slice.char_len() == 2 {
                match u8::from_str_radix(slice.as_str(), 16) {
                    Result::Ok(u) => result.push(u),
                    Result::Err(_) => {
                        return runtime.throw_quick(
                            value_error(),
                            format!("Cannot parse hex value of {}", slice),
                        )
                    }
                }
            } else {
                return runtime.throw_quick(value_error(), from_hex_exc(str.char_len()));
            }
        }
        runtime.return_1(Rc::new(LangBytes::new(result)).into())
    }

    fn create(args: Vec<Variable>, runtime: &mut Runtime) -> FnResult {
        let mut result = Vec::new();
        let iter = first(args).iter(runtime)?;
        while let Option::Some(val) = iter.next(runtime)?.take_first() {
            result.push(IntVar::from(val).to_u8().unwrap());
        }
        runtime.return_1(Rc::new(LangBytes::new(result)).into())
    }

    pub fn bytes_type() -> Type {
        custom_class!(LangBytes, create, "bytes", "fromHex" => from_hex)
    }
}

impl CustomVar for LangBytes {
    fn set(self: Rc<Self>, _name: Name, _object: Variable) {
        unimplemented!()
    }

    fn get_type(&self) -> Type {
        Self::bytes_type()
    }

    fn get_operator(self: Rc<Self>, op: Operator) -> Variable {
        let func = LangBytes::op_fn(op);
        StdMethod::new_native(self, func).into()
    }

    fn get_attribute(self: Rc<Self>, attr: &str) -> Variable {
        let func = match attr {
            "length" => return IntVar::from(self.value.borrow().len()).into(),
            _ => Self::attr_fn(attr),
        };
        StdMethod::new_native(self, func).into()
    }

    fn call_op(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(LangBytes::op_fn(operator), self, args)
    }

    fn call_op_or_goto(
        self: Rc<Self>,
        operator: Operator,
        args: Vec<Variable>,
        runtime: &mut Runtime,
    ) -> FnResult {
        runtime.call_native_method(LangBytes::op_fn(operator), self, args)
    }

    fn str(self: Rc<Self>, runtime: &mut Runtime) -> Result<StringVar, ()> {
        let string =
            String::from_utf8(self.value.borrow().clone()).or_else(|_| self.utf8_err(runtime))?;
        Result::Ok(StringVar::from(string))
    }

    fn repr(self: Rc<Self>, _runtime: &mut Runtime) -> Result<StringVar, ()> {
        Result::Ok(self.repr_value())
    }

    fn bool(self: Rc<Self>, _runtime: &mut Runtime) -> Result<bool, ()> {
        Result::Ok(!self.value.borrow().is_empty())
    }

    fn iter(self: Rc<Self>, _runtime: &mut Runtime) -> Result<looping::Iterator, ()> {
        Result::Ok(Rc::new(BytesIter::new(self)).into())
    }
}

impl BytesIter {
    pub fn new(value: Rc<LangBytes>) -> BytesIter {
        BytesIter {
            current: Cell::new(0),
            value,
        }
    }

    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }
}

impl TypicalIterator for BytesIter {
    fn inner_next(&self) -> Option<Variable> {
        if self.current.get() != self.value.value.borrow().len() {
            let result = self.value.value.borrow()[self.current.get()];
            self.current.set(self.current.get() + 1);
            Option::Some(IntVar::from(result).into())
        } else {
            Option::None
        }
    }

    fn get_type() -> Type {
        custom_class!(BytesIter, create, "BytesIter")
    }
}

impl BytesRevIter {
    fn create(_args: Vec<Variable>, _runtime: &mut Runtime) -> FnResult {
        unimplemented!()
    }
}

impl TypicalIterator for BytesRevIter {
    fn inner_next(&self) -> Option<Variable> {
        if self.current.get() != 0 {
            let result = self.value.value.borrow()[self.current.replace(self.current.get() - 1)];
            Option::Some(IntVar::from(result).into())
        } else {
            Option::None
        }
    }

    fn get_type() -> Type {
        custom_class!(BytesRevIter, create, "BytesRevIter")
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

fn from_hex_exc(len: usize) -> StringVar {
    format!(
        "bytes.fromHex requires a string of even length, not {}",
        len
    )
    .into()
}
