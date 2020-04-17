use crate::base_fn::BaseFunction;
use crate::constant_loaders::{
    function_index, load_bigint, load_bool, load_builtin, load_class, load_decimal, load_int,
    load_str,
};
use crate::file_info::FileInfo;
use crate::int_tools::{bytes_index, bytes_to};
use crate::variable::{Function, Variable};
use std::collections::HashMap;
use std::fs::read;
use std::path::Path;
use std::rc::Rc;

fn load_constant(
    data: &Vec<u8>,
    index: &mut usize,
    functions: &mut Vec<u32>,
    classes: &mut Vec<u32>,
) -> Variable {
    *index += 1;
    match data[*index - 1] {
        0 => load_str(data, index),
        1 => load_int(data, index),
        2 => load_bigint(data, index),
        3 => load_decimal(data, index),
        4 => unimplemented!(), // import
        5 => load_builtin(data, index),
        6 => {
            functions.push(function_index(data, index));
            Variable::Null()
        }
        7 => load_bool(data, index),
        8 => {
            classes.push(function_index(data, index));
            Variable::Custom()
        }
        _ => unimplemented!(),
    }
}

pub fn parse_file(name: String, files: &mut Vec<Rc<FileInfo>>) -> usize {
    let data = read(Path::new(&name)).expect("File not found");
    let file_no = files.len();
    files.push(Rc::new(FileInfo::temp()));
    let mut index: usize = 0;

    let magic_number = bytes_index::<u32>(&data, &mut index);
    if magic_number != bytes_to::<u32>(&vec![0x0A, 0xBA, 0xDE, 0x66]) {
        panic!("File does not start with the magic number")
    }

    // static mut FILES: HashMap<String, &'static FileInfo> = HashMap::new();
    let import_count = bytes_index::<u32>(&data, &mut index);
    let _imports: Vec<Variable> = Vec::with_capacity(import_count as usize);
    if import_count != 0 {
        panic!("Imports not implemented yet")
    }

    let export_count = bytes_index::<u32>(&data, &mut index);
    let _exports: Vec<Variable> = Vec::with_capacity(export_count as usize);
    if export_count != 0 {
        panic!("Exports not implemented yet")
    }

    let constant_count = bytes_index::<u32>(&data, &mut index);
    let mut constants: Vec<Variable> = Vec::with_capacity(constant_count as usize);
    let mut fn_indices: Vec<u32> = Vec::new();
    let mut class_indices: Vec<u32> = Vec::new();
    for _ in 0..constant_count {
        constants.push(load_constant(
            &data,
            &mut index,
            &mut fn_indices,
            &mut class_indices,
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

    let mut fn_count: usize = 0;
    let mut cls_count: usize = 0;
    for c in &mut constants {
        if let Variable::Null() = c {
            *c = Variable::Function(Function::Standard(file_no, fn_indices[fn_count]));
            fn_count += 1;
        } else if let Variable::Custom() = c {
            *c = classes[cls_count].clone();
            cls_count += 1;
        }
    }

    files[file_no] = Rc::new(FileInfo::new(name, constants, functions, HashMap::new()));
    return file_no;
}
