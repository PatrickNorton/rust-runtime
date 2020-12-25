use crate::base_fn::BaseFunction;
use crate::constant_loaders::{
    class_index, function_index, load_ascii, load_bigint, load_bool, load_builtin, load_bytes,
    load_char, load_class, load_decimal, load_int, load_range, load_std_str, load_str,
    option_index, tuple_indices,
};
use crate::file_info::FileInfo;
use crate::function::Function;
use crate::int_tools::bytes_index;
use crate::jump_table::JumpTable;
use crate::std_type::Type;
use crate::tuple::LangTuple;
use crate::variable::{InnerVar, Variable};
use std::collections::HashMap;
use std::fs::read;
use std::path::Path;

const FILE_EXTENSION: &str = ".nbyte";

#[derive(Debug, Clone)]
enum Constant {
    Current(Variable),
    Later(LoadType),
}

#[derive(Debug, Clone)]
enum LoadType {
    Function(u32),
    Class(u32),
    Option(u16),
    Tuple(Vec<u16>),
    OptionType(u32),
}

fn load_constant(data: &[u8], index: &mut usize, imports: &[Variable]) -> Constant {
    *index += 1;
    match data[*index - 1] {
        0 => Variable::default().into(),
        1 => load_str(data, index).into(),
        2 => load_int(data, index).into(),
        3 => load_bigint(data, index).into(),
        4 => load_decimal(data, index).into(),
        5 => imports[bytes_index::<u32>(data, index) as usize]
            .clone()
            .into(),
        6 => load_builtin(data, index).into(),
        7 => LoadType::Function(function_index(data, index)).into(),
        8 => load_bool(data, index).into(),
        9 => LoadType::Class(class_index(data, index)).into(),
        10 => LoadType::Option(option_index(data, index)).into(),
        11 => load_bytes(data, index).into(),
        12 => load_range(data, index).into(),
        13 => LoadType::Tuple(tuple_indices(data, index)).into(),
        14 => LoadType::OptionType(class_index(data, index)).into(),
        15 => load_char(data, index).into(),
        16 => load_ascii(data, index).into(),
        x => panic!("Invalid value for constant: {}", x),
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
    let mut constants = Vec::with_capacity(constant_count as usize);
    for _ in 0..constant_count {
        constants.push(load_constant(&data, &mut index, &imports));
    }

    let fn_count = bytes_index::<u32>(&data, &mut index);
    let mut functions: Vec<BaseFunction> = Vec::with_capacity(fn_count as usize);
    for _ in 0..fn_count {
        functions.push(BaseFunction::parse(&data, &mut index))
    }

    let class_count = bytes_index::<u32>(&data, &mut index);
    let mut classes: Vec<Type> = Vec::with_capacity(fn_count as usize);
    for _ in 0..class_count {
        classes.push(load_class(file_no, &data, &mut index, &mut functions));
    }

    let table_count = bytes_index::<u32>(&data, &mut index);
    let mut jump_tables: Vec<JumpTable> = Vec::with_capacity(table_count as usize);
    for _ in 0..table_count {
        jump_tables.push(JumpTable::parse(&data, &mut index));
    }

    debug_assert_eq!(data.len(), index);

    let mut new_constants = Vec::with_capacity(constants.len());
    for c in constants {
        match c {
            Constant::Current(x) => new_constants.push(x),
            Constant::Later(x) => new_constants.push(match x {
                LoadType::Function(d) => Function::Standard(file_no, d).into(),
                LoadType::Class(d) => classes[d as usize].into(),
                LoadType::Option(d) => Option::Some(new_constants[d as usize].clone()).into(),
                LoadType::Tuple(v) => LangTuple::new(
                    v.into_iter()
                        .map(|i| new_constants[i as usize].clone())
                        .collect(),
                )
                .into(),
                LoadType::OptionType(d) => {
                    let t = match new_constants[d as usize] {
                        Variable::Normal(InnerVar::Type(t)) => Box::leak(Box::new(t)),
                        _ => panic!(),
                    };
                    t.make_option().into()
                }
            }),
        }
    }

    files[file_no] = FileInfo::new(name, new_constants, functions, exports, jump_tables);
    file_no
}

impl From<Variable> for Constant {
    fn from(x: Variable) -> Self {
        Constant::Current(x)
    }
}

impl From<LoadType> for Constant {
    fn from(x: LoadType) -> Self {
        Constant::Later(x)
    }
}
