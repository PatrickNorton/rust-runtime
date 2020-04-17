use crate::base_fn::BaseFunction;
use crate::builtins::builtin_of;
use crate::int_tools::bytes_index;
use crate::operator::Operator;
use crate::std_type::Type;
use crate::std_variable::StdVarMethod;
use crate::variable::{Name, Variable};
use num::bigint::Sign;
use num::traits::pow::pow;
use num::{BigInt, BigRational, FromPrimitive};
use std::collections::{HashMap, HashSet};

pub fn load_std_str(data: &Vec<u8>, index: &mut usize) -> String {
    let size = bytes_index::<u32>(data, index);
    let mut value: Vec<u8> = Vec::with_capacity(size as usize);
    for _ in 0..size {
        let mut char = data[*index];
        *index += 1;
        value.push(char);
        while char >= 0b11000000 {
            char = data[*index];
            *index += 1;
            value.push(char);
        }
    }
    String::from_utf8(value).expect("UTF-8 error")
}

pub fn load_str(data: &Vec<u8>, index: &mut usize) -> Variable {
    Variable::String(load_std_str(data, index))
}

pub fn load_builtin(data: &Vec<u8>, index: &mut usize) -> Variable {
    builtin_of(bytes_index::<u32>(data, index) as usize)
}

pub fn load_int(data: &Vec<u8>, index: &mut usize) -> Variable {
    let value = bytes_index::<u32>(data, index);
    Variable::Bigint(BigInt::from_u32(value).unwrap())
}

pub fn load_bigint(data: &Vec<u8>, index: &mut usize) -> Variable {
    let count = bytes_index::<u32>(data, index);
    let mut values: Vec<u32> = Vec::with_capacity(count as usize);
    for _ in 0..count {
        values.push(bytes_index::<u32>(data, index));
    }
    Variable::Bigint(BigInt::new(Sign::Plus, values))
}

pub fn load_decimal(data: &Vec<u8>, index: &mut usize) -> Variable {
    let count = bytes_index::<u32>(data, index);
    let scale = bytes_index::<u32>(data, index);
    let mut values: Vec<u32> = Vec::with_capacity(count as usize);
    for _ in 0..count {
        values.push(bytes_index::<u32>(data, index));
    }
    Variable::Decimal(BigRational::new(
        BigInt::new(Sign::Plus, values),
        pow(BigInt::from_u64(10).unwrap(), scale as usize),
    ))
}

pub fn function_index(data: &Vec<u8>, index: &mut usize) -> u32 {
    bytes_index::<u32>(data, index)
}

pub fn class_index(data: &Vec<u8>, index: &mut usize) -> u32 {
    bytes_index::<u32>(data, index)
}

pub fn load_bool(data: &Vec<u8>, index: &mut usize) -> Variable {
    let value = data[*index];
    *index += 1;
    Variable::Bool(value != 0)
}

fn get_variables(data: &Vec<u8>, index: &mut usize) -> HashSet<String> {
    let mut variables: HashSet<String> = HashSet::new();
    let byte_size = bytes_index::<u32>(data, index);
    for _ in 0..byte_size {
        let name = load_std_str(data, index);
        bytes_index::<u16>(data, index); // TODO: Get classes properly
        variables.insert(name);
    }
    variables
}

fn get_operators(
    data: &Vec<u8>,
    index: &mut usize,
    functions: &mut Vec<BaseFunction>,
) -> HashMap<Operator, StdVarMethod> {
    let mut operators: HashMap<Operator, StdVarMethod> = HashMap::new();
    let byte_size = bytes_index::<u32>(data, index);
    for _ in 0..byte_size {
        let op: Operator = FromPrimitive::from_u8(data[*index]).unwrap();
        *index += 1;
        let method_size = bytes_index::<u32>(data, index);
        let values = data[*index..*index + method_size as usize].to_vec();
        *index += method_size as usize;
        operators.insert(op, StdVarMethod::Standard(functions.len() as u32));
        functions.push(BaseFunction::new(String::new(), 0, values));
    }
    operators
}

fn get_methods(
    data: &Vec<u8>,
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
        methods.insert(name, StdVarMethod::Standard(functions.len() as u32));
        functions.push(BaseFunction::new(String::new(), 0, values));
    }
    methods
}

fn get_properties(
    data: &Vec<u8>,
    index: &mut usize,
    functions: &mut Vec<BaseFunction>,
) -> HashMap<String, (u32, u32)> {
    let mut properties: HashMap<String, (u32, u32)> = HashMap::new();
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

        properties.insert(name.clone(), (getter_index as u32, setter_index as u32));
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
        result.insert(Name::Attribute(i.0), i.1);
    }
    result
}

pub fn load_class(
    file_no: usize,
    data: &Vec<u8>,
    index: &mut usize,
    functions: &mut Vec<BaseFunction>,
) -> Variable {
    let name = load_std_str(data, index);
    if bytes_index::<u32>(data, index) != 0 {
        panic!("Supers not allowed yet")
    }
    let _generic_size = bytes_index::<u16>(data, index);
    get_variables(data, index);
    get_variables(data, index);
    let operators = get_operators(data, index, functions);
    let static_operators = get_operators(data, index, functions);
    let methods = get_methods(data, index, functions);
    let static_methods = get_methods(data, index, functions);
    let _properties = get_properties(data, index, functions);

    Variable::Type(Type::new_std(
        name,
        file_no,
        merge_maps(operators, methods),
        merge_maps(static_operators, static_methods),
    ))
}

/*
    Type loadClass(FileInfo* file, const std::vector<uint8_t>& data, size_t& index, std::vector<BaseFunction>& functions) {
        auto name = loadStdStr(data, index);
        if (IntTools::bytesTo<uint32_t>(data, index) != 0) { // No supers allowed yet
            throw std::runtime_error("Supers not allowed yet");
        }
        auto genericSize = IntTools::bytesTo<uint16_t>(data, index);
        getVariables(data, index);
        getVariables(data, index);
        auto operators = getOperators(data, index, functions);
        auto staticOperators = getOperators(data, index, functions);
        auto methods = getMethods(data, index, functions);
        auto staticMethods = getMethods(data, index, functions);
        auto properties = getProperties(data, index, functions);

        return std::make_shared<Constants::StdType>(name, file, genericSize, operators, staticOperators, methods, staticMethods, properties);
    }
*/
