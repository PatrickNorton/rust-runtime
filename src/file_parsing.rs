use crate::base_fn::BaseFunction;
use crate::constant_loaders::{
    class_index, function_index, load_bigint, load_bool, load_builtin, load_bytes, load_class,
    load_decimal, load_int, load_range, load_std_str, load_str, option_index, tuple_indices,
};
use crate::file_info::FileInfo;
use crate::function::Function;
use crate::int_tools::bytes_index;
use crate::jump_table::JumpTable;
use crate::option::LangOption;
use crate::std_type::Type;
use crate::tuple::LangTuple;
use crate::variable::Variable;
use std::collections::HashMap;
use std::fs::read;
use std::path::Path;

const FILE_EXTENSION: &str = ".nbyte";

#[derive(Debug, Clone)]
enum LoadType {
    Function(u32),
    Class(u32),
    Option(u16),
    Tuple(Vec<u16>),
    OptionType(u16),
}

fn load_constant(
    data: &[u8],
    index: &mut usize,
    load_later: &mut Vec<(usize, LoadType)>,
    imports: &[Variable],
    constant_no: usize,
) -> Variable {
    *index += 1;
    match data[*index - 1] {
        0 => load_str(data, index),
        1 => load_int(data, index),
        2 => load_bigint(data, index),
        3 => load_decimal(data, index),
        4 => imports[bytes_index::<u32>(data, index) as usize].clone(),
        5 => load_builtin(data, index),
        6 => {
            load_later.push((constant_no, LoadType::Function(function_index(data, index))));
            Variable::Null()
        }
        7 => load_bool(data, index),
        8 => {
            load_later.push((constant_no, LoadType::Class(class_index(data, index))));
            Variable::Null()
        }
        9 => {
            load_later.push((constant_no, LoadType::Option(option_index(data, index))));
            Variable::Null()
        }
        10 => load_bytes(data, index),
        11 => load_range(data, index),
        12 => {
            load_later.push((constant_no, LoadType::Tuple(tuple_indices(data, index))));
            Variable::Null()
        }
        13 => {
            load_later.push((constant_no, LoadType::OptionType(option_index(data, index))));
            Variable::Null()
        }
        _ => panic!("Invalid value for constant: {}", data[*index - 1]),
    }
}

pub fn parse_file(name: String, files: &mut Vec<FileInfo>) -> usize {
    let data = read(Path::new(&name)).unwrap_or_else(|_| panic!("File {} not found", &name));
    let file_no = files.len();
    files.push(FileInfo::temp());
    let mut index: usize = 0;

    let magic_number = bytes_index::<u32>(&data, &mut index);
    if magic_number != 0x0A_BAD_E66 {
        panic!("File does not start with the magic number")
    }

    let import_count = bytes_index::<u32>(&data, &mut index);
    let mut imports: Vec<Variable> = Vec::with_capacity(import_count as usize);
    for _ in 0..import_count {
        let _used_name = load_std_str(&data, &mut index);
        let full_name = load_std_str(&data, &mut index);
        let names: Vec<&str> = full_name.split('.').collect();
        let folder_split: Vec<&str> = name.rsplitn(2, '/').collect();
        let parent_folder = folder_split[1];
        let file_name = parent_folder.to_owned() + "/" + names[0] + FILE_EXTENSION;
        let file_index = files
            .iter()
            .position(|a| a.get_name() == &file_name)
            .unwrap_or_else(|| parse_file(file_name, files));
        // FIXME: Recursion fails
        let other_file = &files[file_index];
        // TODO: Get nested dots
        imports.push(other_file.get_export(names[1]).clone());
    }

    let export_count = bytes_index::<u32>(&data, &mut index);
    let mut exports: HashMap<String, u32> = HashMap::with_capacity(export_count as usize);
    for _ in 0..export_count {
        let export_name = load_std_str(&data, &mut index);
        let const_no = bytes_index::<u32>(&data, &mut index);
        exports.insert(export_name, const_no);
    }

    let constant_count = bytes_index::<u32>(&data, &mut index);
    let mut constants: Vec<Variable> = Vec::with_capacity(constant_count as usize);
    let mut loaded_later: Vec<(usize, LoadType)> = Vec::new();
    for i in 0..constant_count {
        constants.push(load_constant(
            &data,
            &mut index,
            &mut loaded_later,
            &imports,
            i as usize,
        ));
    }

    let fn_count = bytes_index::<u32>(&data, &mut index);
    let mut functions: Vec<BaseFunction> = Vec::with_capacity(fn_count as usize);
    for _ in 0..fn_count {
        functions.push(BaseFunction::parse(&data, &mut index))
    }

    let class_count = bytes_index::<u32>(&data, &mut index);
    let mut classes: Vec<Variable> = Vec::with_capacity(fn_count as usize);
    for _ in 0..class_count {
        classes.push(load_class(file_no, &data, &mut index, &mut functions));
    }

    let table_count = bytes_index::<u32>(&data, &mut index);
    let mut jump_tables: Vec<JumpTable> = Vec::with_capacity(table_count as usize);
    for _ in 0..table_count {
        jump_tables.push(JumpTable::parse(&data, &mut index));
    }

    debug_assert_eq!(data.len(), index);

    for (i, load) in loaded_later {
        let val = match load {
            LoadType::Function(d) => Variable::Function(Function::Standard(file_no, d)),
            LoadType::Class(d) => classes[d as usize].clone(),
            LoadType::Option(d) => {
                Variable::Option(LangOption::new(Option::Some(constants[d as usize].clone())))
            }
            LoadType::Tuple(v) => Variable::Tuple(LangTuple::new(
                v.into_iter()
                    .map(|i| constants[i as usize].clone())
                    .collect(),
            )),
            LoadType::OptionType(d) => {
                let t = match constants[d as usize] {
                    Variable::Type(t) => Box::leak(Box::new(t)),
                    _ => panic!(),
                };
                Variable::Type(Type::Option(t))
            }
        };
        constants[i] = val;
    }

    files[file_no] = FileInfo::new(name, constants, functions, exports, jump_tables);
    file_no
}
