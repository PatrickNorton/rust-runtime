use crate::base_fn::BaseFunction;
use crate::builtins::builtin_of;
use crate::custom_types::bytes::LangBytes;
use crate::custom_types::range::Range;
use crate::int_tools::bytes_index;
use crate::int_var::IntVar;
use crate::lang_union::UnionMethod;
use crate::name::Name;
use crate::operator::Operator;
use crate::property::{Property, StdProperty};
use crate::rational_var::RationalVar;
use crate::std_type::Type;
use crate::std_variable::StdVarMethod;
use crate::string_var::StringVar;
use crate::variable::Variable;
use num::bigint::Sign;
use num::traits::pow::pow;
use num::traits::{One, Zero};
use num::{BigInt, BigRational, FromPrimitive};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

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

pub fn load_str(data: &[u8], index: &mut usize) -> Variable {
    Variable::String(StringVar::from_leak(load_std_str(data, index)))
}

pub fn load_builtin(data: &[u8], index: &mut usize) -> Variable {
    builtin_of(bytes_index::<u32>(data, index) as usize)
}

pub fn load_int(data: &[u8], index: &mut usize) -> Variable {
    let value = bytes_index::<i32>(data, index);
    Variable::Bigint(IntVar::from_i32(value).unwrap())
}

pub fn load_bigint(data: &[u8], index: &mut usize) -> Variable {
    Variable::Bigint(inner_bigint(data, index).into())
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
    Variable::Decimal(RationalVar::new(BigRational::new(
        BigInt::new(Sign::Plus, values),
        pow(BigInt::from_u64(10).unwrap(), scale as usize),
    )))
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
    Variable::Bool(value != 0)
}

pub fn option_index(data: &[u8], index: &mut usize) -> u16 {
    bytes_index::<u16>(data, index)
}

pub fn load_bytes(data: &[u8], index: &mut usize) -> Variable {
    let len = bytes_index::<u32>(data, index) as usize;
    let byte_arr = &data[*index..*index + len];
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

fn get_variables(data: &[u8], index: &mut usize) -> HashSet<StringVar> {
    let mut variables: HashSet<StringVar> = HashSet::new();
    let byte_size = bytes_index::<u32>(data, index);
    for _ in 0..byte_size {
        let name = StringVar::from_leak(load_std_str(data, index));
        bytes_index::<u16>(data, index); // TODO: Get classes properly
        variables.insert(name);
    }
    variables
}

fn get_operators(
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
        let method_size = bytes_index::<u32>(data, index);
        let values = data[*index..*index + method_size as usize].to_vec();
        *index += method_size as usize;
        operators.insert(op, StdVarMethod::Standard(file_no, functions.len() as u32));
        functions.push(BaseFunction::new(String::new(), 0, values));
    }
    operators
}

fn get_methods(
    data: &[u8],
    file_no: usize,
    index: &mut usize,
    functions: &mut Vec<BaseFunction>,
) -> HashMap<String, StdVarMethod> {
    let mut methods: HashMap<String, StdVarMethod> = HashMap::new();
    let byte_size = bytes_index::<u32>(data, index);
    for _ in 0..byte_size {
        let name = load_std_str(data, index);
        let method_size = bytes_index::<u32>(data, index);
        let values = data[*index..*index + method_size as usize].to_vec();
        *index += method_size as usize;
        methods.insert(
            name,
            StdVarMethod::Standard(file_no, functions.len() as u32),
        );
        functions.push(BaseFunction::new(String::new(), 0, values));
    }
    methods
}

fn get_properties(
    data: &[u8],
    file_no: usize,
    index: &mut usize,
    functions: &mut Vec<BaseFunction>,
) -> HashMap<StringVar, Property> {
    let mut properties: HashMap<StringVar, Property> = HashMap::new();
    let byte_size = bytes_index::<u32>(data, index);
    for _ in 0..byte_size {
        let name = load_std_str(data, index);

        let getter_size = bytes_index::<u32>(data, index);
        let getter = data[*index..*index + getter_size as usize].to_vec();
        *index += getter_size as usize;
        let getter_index = functions.len();
        functions.push(BaseFunction::new(name.clone(), 0, getter));

        let setter_size = bytes_index::<u32>(data, index);
        let setter = data[*index..*index + setter_size as usize].to_vec();
        *index += setter_size as usize;
        let setter_index = functions.len();
        functions.push(BaseFunction::new(name.clone(), 0, setter));

        properties.insert(
            StringVar::from_leak(name),
            Property::Standard(StdProperty::new(
                file_no,
                getter_index as u32,
                setter_index as u32,
            )),
        );
    }
    properties
}

fn merge_maps(
    ops: HashMap<Operator, StdVarMethod>,
    strings: HashMap<String, StdVarMethod>,
) -> HashMap<Name, StdVarMethod> {
    let mut result: HashMap<Name, StdVarMethod> = HashMap::with_capacity(ops.len() + strings.len());
    for i in ops {
        result.insert(Name::Operator(i.0), i.1);
    }
    for i in strings {
        result.insert(Name::Attribute(StringVar::from_leak(i.0)), i.1);
    }
    result
}

fn merge_maps_union(
    ops: HashMap<Operator, StdVarMethod>,
    strings: HashMap<String, StdVarMethod>,
) -> HashMap<Name, UnionMethod> {
    let mut result: HashMap<Name, UnionMethod> = HashMap::with_capacity(ops.len() + strings.len());
    for i in ops {
        result.insert(Name::Operator(i.0), std_to_union(i.1));
    }
    for i in strings {
        result.insert(
            Name::Attribute(StringVar::from_leak(i.0)),
            std_to_union(i.1),
        );
    }
    result
}

fn std_to_union(val: StdVarMethod) -> UnionMethod {
    match val {
        StdVarMethod::Standard(a, b) => UnionMethod::Standard(a, b),
        _ => panic!("Cannot convert method"),
    }
}

fn get_names(data: &[u8], index: &mut usize) -> Option<Vec<StringVar>> {
    let is_union = data[*index] != 0;
    *index += 1;
    if is_union {
        let vec_size = bytes_index::<u32>(data, index);
        Option::Some(
            (0..vec_size)
                .map(|_| StringVar::from_leak(load_std_str(data, index)))
                .collect(),
        )
    } else {
        Option::None
    }
}

pub fn load_class(
    file_no: usize,
    data: &[u8],
    index: &mut usize,
    functions: &mut Vec<BaseFunction>,
) -> Variable {
    let name = load_std_str(data, index);
    if bytes_index::<u32>(data, index) != 0 {
        panic!("Supers not allowed yet")
    }
    let _generic_size = bytes_index::<u16>(data, index);
    assert_eq!(_generic_size, 0);
    let names = get_names(data, index);
    let variables = get_variables(data, index);
    get_variables(data, index);
    let operators = get_operators(data, file_no, index, functions);
    let static_operators = get_operators(data, file_no, index, functions);
    let methods = get_methods(data, file_no, index, functions);
    let static_methods = get_methods(data, file_no, index, functions);
    let properties = get_properties(data, file_no, index, functions);

    match names {
        Option::None => Variable::Type(Type::new_std(
            StringVar::from_leak(name),
            file_no,
            variables,
            merge_maps(operators, methods),
            merge_maps(static_operators, static_methods),
            properties,
        )),
        Option::Some(variants) => Variable::Type(Type::new_union(
            StringVar::from_leak(name),
            file_no,
            variants,
            variables,
            merge_maps_union(operators, methods),
            merge_maps_union(static_operators, static_methods),
            properties,
        )),
    }
}
