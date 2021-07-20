use crate::base_fn::BaseFunction;
use crate::builtins::builtin_of;
use crate::custom_types::bytes::LangBytes;
use crate::custom_types::range::Range;
use crate::fmt::FormatArgs;
use crate::int_tools::bytes_index;
use crate::int_var::IntVar;
use crate::method::InnerMethod;
use crate::name_map::NameMap;
use crate::operator::Operator;
use crate::property::{Property, StdProperty};
use crate::rational_var::RationalVar;
use crate::std_type::Type;
use crate::std_variable::StdVarMethod;
use crate::string_var::StringVar;
use crate::variable::Variable;
use ascii::AsciiChar;
use num::bigint::Sign;
use num::traits::pow::pow;
use num::traits::{One, Zero};
use num::{BigInt, BigRational, FromPrimitive};
use std::char;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;

pub fn load_std_str(data: &[u8], index: &mut usize) -> String {
    let size = bytes_index::<u32>(data, index);
    let mut value: Vec<u8> = Vec::with_capacity(size as usize);
    for _ in 0..size {
        let mut char = data[*index];
        *index += 1;
        value.push(char);
        while char >= 0b1100_0000 {
            char = data[*index];
            *index += 1;
            value.push(char);
        }
    }
    String::from_utf8(value).expect("UTF-8 error")
}

pub fn load_ascii_str(data: &[u8], index: &mut usize) -> Box<[AsciiChar]> {
    let size = bytes_index::<u32>(data, index);
    let mut value = Vec::with_capacity(size as usize);
    for _ in 0..size {
        let char = AsciiChar::from_ascii(data[*index])
            .unwrap_or_else(|_| panic!("Character value {} is invalid ASCII", data[*index]));
        *index += 1;
        value.push(char);
    }
    value.into_boxed_slice()
}

pub fn load_str(data: &[u8], index: &mut usize) -> Variable {
    StringVar::from_leak(load_std_str(data, index)).into()
}

pub fn load_builtin(data: &[u8], index: &mut usize) -> Variable {
    builtin_of(bytes_index::<u32>(data, index) as usize)
}

pub fn load_int(data: &[u8], index: &mut usize) -> Variable {
    let value = bytes_index::<i32>(data, index);
    value.into()
}

pub fn load_bigint(data: &[u8], index: &mut usize) -> Variable {
    inner_bigint(data, index).into()
}

fn inner_bigint(data: &[u8], index: &mut usize) -> BigInt {
    let count = bytes_index::<u32>(data, index);
    let mut values: Vec<u32> = Vec::with_capacity(count as usize);
    for _ in 0..count {
        values.push(bytes_index::<u32>(data, index));
    }
    values.reverse(); // Comes in big-endian, little-endian needed
    BigInt::new(Sign::Plus, values)
}

pub fn load_decimal(data: &[u8], index: &mut usize) -> Variable {
    let count = bytes_index::<u32>(data, index);
    let scale = bytes_index::<u32>(data, index);
    let mut values: Vec<u32> = Vec::with_capacity(count as usize);
    for _ in 0..count {
        values.push(bytes_index::<u32>(data, index));
    }
    RationalVar::new(BigRational::new(
        BigInt::new(Sign::Plus, values),
        pow(BigInt::from_u64(10).unwrap(), scale as usize),
    ))
    .into()
}

pub fn function_index(data: &[u8], index: &mut usize) -> u32 {
    bytes_index::<u32>(data, index)
}

pub fn class_index(data: &[u8], index: &mut usize) -> u32 {
    bytes_index::<u32>(data, index)
}

pub fn load_bool(data: &[u8], index: &mut usize) -> Variable {
    let value = data[*index];
    *index += 1;
    (value != 0).into()
}

pub fn option_index(data: &[u8], index: &mut usize) -> u16 {
    bytes_index::<u16>(data, index)
}

pub fn load_bytes(data: &[u8], index: &mut usize) -> Variable {
    let len = bytes_index::<u32>(data, index) as usize;
    let byte_arr = &data[*index..*index + len];
    *index += len;
    Rc::new(LangBytes::new(byte_arr.to_vec())).into()
}

pub fn load_range(data: &[u8], index: &mut usize) -> Variable {
    let start = get_range_index(data, index);
    let stop = get_range_index(data, index);
    let step = get_range_index(data, index);
    Rc::new(Range::new(
        start.unwrap_or_else(Zero::zero),
        stop.unwrap_or_else(Zero::zero),
        step.unwrap_or_else(One::one),
    ))
    .into()
}

pub fn load_char(data: &[u8], index: &mut usize) -> Variable {
    bytes_index::<char>(data, index).into()
}

pub fn load_ascii(data: &[u8], index: &mut usize) -> Variable {
    StringVar::from_leak_ascii(load_ascii_str(data, index)).into()
}

pub fn load_fmt_args(data: &[u8], index: &mut usize) -> Variable {
    Rc::new(FormatArgs::parse(data, index)).into()
}

pub fn tuple_indices(data: &[u8], index: &mut usize) -> Vec<u16> {
    let len = bytes_index::<u32>(data, index);
    (0..len).map(|_| bytes_index(data, index)).collect()
}

fn get_range_index(data: &[u8], index: &mut usize) -> Option<IntVar> {
    *index += 1;
    match data[*index - 1] {
        0 => Option::None,
        1 => {
            let value = bytes_index::<u32>(data, index);
            Option::Some(IntVar::from(value))
        }
        2 => Option::Some(inner_bigint(data, index).into()),
        _ => panic!(),
    }
}

fn get_variables(data: &[u8], index: &mut usize) -> HashSet<Arc<str>> {
    let byte_size = bytes_index::<u32>(data, index);
    let mut variables = HashSet::with_capacity(byte_size as usize);
    for _ in 0..byte_size {
        let name = load_std_str(data, index);
        bytes_index::<u16>(data, index); // TODO: Get classes properly
        variables.insert(name.into());
    }
    variables
}

fn get_operators(
    cls_name: &str,
    data: &[u8],
    file_no: usize,
    index: &mut usize,
    functions: &mut Vec<BaseFunction>,
) -> HashMap<Operator, StdVarMethod> {
    let mut operators: HashMap<Operator, StdVarMethod> = HashMap::new();
    let byte_size = bytes_index::<u32>(data, index);
    for _ in 0..byte_size {
        let op: Operator = FromPrimitive::from_u8(data[*index]).expect("Invalid operator");
        *index += 1;
        let is_gen = data[*index];
        *index += 1;
        let method_size = bytes_index::<u32>(data, index);
        let values = data[*index..*index + method_size as usize].to_vec();
        *index += method_size as usize;
        let full_name = format!("{}.{}", cls_name, op.name());
        operators.insert(op, StdVarMethod::Standard(file_no, functions.len() as u32));
        let base_fn = if is_gen != 0 {
            BaseFunction::new_gen(full_name, 0, values)
        } else {
            BaseFunction::new(full_name, 0, values)
        };
        functions.push(base_fn);
    }
    operators
}

fn get_methods(
    cls_name: &str,
    data: &[u8],
    file_no: usize,
    index: &mut usize,
    functions: &mut Vec<BaseFunction>,
) -> HashMap<String, StdVarMethod> {
    let mut methods: HashMap<String, StdVarMethod> = HashMap::new();
    let byte_size = bytes_index::<u32>(data, index);
    for _ in 0..byte_size {
        let name = load_std_str(data, index);
        let is_gen = data[*index];
        *index += 1;
        let method_size = bytes_index::<u32>(data, index);
        let values = data[*index..*index + method_size as usize].to_vec();
        *index += method_size as usize;
        let full_name = format!("{}.{}", cls_name, name);
        methods.insert(
            name,
            StdVarMethod::Standard(file_no, functions.len() as u32),
        );
        let base_fn = if is_gen != 0 {
            BaseFunction::new_gen(full_name, 0, values)
        } else {
            BaseFunction::new(full_name, 0, values)
        };
        functions.push(base_fn);
    }
    methods
}

fn get_properties(
    cls_name: &str,
    data: &[u8],
    file_no: usize,
    index: &mut usize,
    functions: &mut Vec<BaseFunction>,
) -> HashMap<String, Property> {
    let mut properties: HashMap<String, Property> = HashMap::new();
    let byte_size = bytes_index::<u32>(data, index);
    for _ in 0..byte_size {
        let name = load_std_str(data, index);

        assert_eq!(data[*index], 0);
        *index += 1;
        let getter_size = bytes_index::<u32>(data, index);
        let getter = data[*index..*index + getter_size as usize].to_vec();
        *index += getter_size as usize;
        let getter_index = functions.len();
        functions.push(BaseFunction::new(
            format!("{}.{}$get", cls_name, name),
            0,
            getter,
        ));

        assert_eq!(data[*index], 0);
        *index += 1;
        let setter_size = bytes_index::<u32>(data, index);
        let setter = data[*index..*index + setter_size as usize].to_vec();
        *index += setter_size as usize;
        let setter_index = functions.len();
        functions.push(BaseFunction::new(
            format!("{}.{}$set", cls_name, name),
            0,
            setter,
        ));

        properties.insert(
            name,
            Property::Standard(StdProperty::new(
                file_no,
                getter_index as u32,
                setter_index as u32,
            )),
        );
    }
    properties
}

fn merge_maps_union<T>(
    ops: HashMap<Operator, StdVarMethod>,
    strings: HashMap<String, StdVarMethod>,
) -> NameMap<InnerMethod<T>> {
    let new_ops = unionize_map(ops);
    let new_strings = unionize_map(strings);
    NameMap::from_values(new_ops, new_strings)
}

fn unionize_map<T: Eq + Hash, U>(value: HashMap<T, StdVarMethod>) -> HashMap<T, InnerMethod<U>> {
    value
        .into_iter()
        .map(|(k, v)| (k, std_to_union(v)))
        .collect()
}

fn std_to_union<T>(val: StdVarMethod) -> InnerMethod<T> {
    match val {
        StdVarMethod::Standard(a, b) => InnerMethod::Standard(a, b),
        _ => panic!("Cannot convert method"),
    }
}

fn get_names(data: &[u8], index: &mut usize) -> Option<Vec<String>> {
    let is_union = data[*index] != 0;
    *index += 1;
    if is_union {
        let vec_size = bytes_index::<u32>(data, index);
        Option::Some((0..vec_size).map(|_| load_std_str(data, index)).collect())
    } else {
        Option::None
    }
}

pub fn load_class(
    file_no: usize,
    data: &[u8],
    index: &mut usize,
    functions: &mut Vec<BaseFunction>,
) -> Type {
    let name = load_std_str(data, index);
    let supers = (0..bytes_index::<u32>(data, index))
        .map(|_| bytes_index::<u32>(data, index))
        .collect();
    let _generic_size = bytes_index::<u16>(data, index);
    // assert_eq!(_generic_size, 0);
    let names = get_names(data, index);
    let variables = get_variables(data, index);
    get_variables(data, index);
    let operators = get_operators(&*name, data, file_no, index, functions);
    let static_operators = get_operators(&*name, data, file_no, index, functions);
    let methods = get_methods(&*name, data, file_no, index, functions);
    let static_methods = get_methods(&*name, data, file_no, index, functions);
    let properties = get_properties(&*name, data, file_no, index, functions);

    match names {
        Option::None => Type::new_std(
            StringVar::from_leak(name),
            file_no,
            supers,
            variables,
            NameMap::from_values(operators, methods),
            NameMap::from_values(static_operators, static_methods),
            properties,
        ),
        Option::Some(variants) => Type::new_union(
            StringVar::from_leak(name),
            file_no,
            supers,
            variants,
            variables,
            merge_maps_union(operators, methods),
            merge_maps_union(static_operators, static_methods),
            properties,
        ),
    }
}
