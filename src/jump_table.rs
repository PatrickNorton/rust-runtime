use crate::constant_loaders::{load_bigint, load_std_str};
use crate::int_tools::bytes_index;
use crate::int_var::IntVar;
use crate::string_var::StringVar;
use num::ToPrimitive;
use std::collections::HashMap;
use std::ops::Index;

#[derive(Debug)]
pub enum JumpTable {
    Compact(CompactJumpTbl),
    Big(BigJumpTbl),
    String(StrJumpTbl),
}

#[derive(Debug)]
pub struct CompactJumpTbl {
    values: Vec<usize>,
    default: usize,
}

#[derive(Debug)]
pub struct BigJumpTbl {
    values: HashMap<IntVar, usize>,
    default: usize,
}

#[derive(Debug)]
pub struct StrJumpTbl {
    values: HashMap<&'static str, usize>,
    default: usize,
}

impl JumpTable {
    pub fn parse(data: &[u8], index: &mut usize) -> JumpTable {
        let tbl_type = data[*index];
        *index += 1;
        match tbl_type {
            0 => JumpTable::Compact(CompactJumpTbl::parse(data, index)),
            1 => JumpTable::Big(BigJumpTbl::parse(data, index)),
            2 => JumpTable::String(StrJumpTbl::parse(data, index)),
            _ => panic!("{} is an invalid table-type number", tbl_type),
        }
    }
}

impl CompactJumpTbl {
    pub fn parse(data: &[u8], index: &mut usize) -> CompactJumpTbl {
        let size = bytes_index::<u32>(data, index);
        let values = (0..size)
            .map(|_| bytes_index::<u32>(data, index) as usize)
            .collect();
        let default = bytes_index::<u32>(data, index) as usize;
        CompactJumpTbl { values, default }
    }
}

impl BigJumpTbl {
    pub fn parse(data: &[u8], index: &mut usize) -> BigJumpTbl {
        let size = bytes_index::<u32>(data, index);
        let values = (0..size)
            .map(|_| {
                (
                    load_bigint(data, index).into(),
                    bytes_index::<u32>(data, index) as usize,
                )
            })
            .collect();
        let default = bytes_index::<u32>(data, index) as usize;
        BigJumpTbl { values, default }
    }
}

impl StrJumpTbl {
    pub fn parse(data: &[u8], index: &mut usize) -> StrJumpTbl {
        let size = bytes_index::<u32>(data, index);
        let values = (0..size)
            .map(|_| {
                (
                    Box::leak(load_std_str(data, index).into_boxed_str()) as &'static str,
                    bytes_index::<u32>(data, index) as usize,
                )
            })
            .collect();
        let default = bytes_index::<u32>(data, index) as usize;
        StrJumpTbl { values, default }
    }
}

impl Index<usize> for CompactJumpTbl {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        self.values.get(index).unwrap_or_else(|| &self.default)
    }
}

impl Index<IntVar> for CompactJumpTbl {
    type Output = usize;

    fn index(&self, index: IntVar) -> &Self::Output {
        index.to_usize().map_or_else(|| &self.default, |i| &self[i])
    }
}

impl Index<IntVar> for BigJumpTbl {
    type Output = usize;

    fn index(&self, index: IntVar) -> &Self::Output {
        self.values.get(&index).unwrap_or_else(|| &self.default)
    }
}

impl Index<StringVar> for StrJumpTbl {
    type Output = usize;

    fn index(&self, index: StringVar) -> &Self::Output {
        self.values.get(&*index).unwrap_or_else(|| &self.default)
    }
}