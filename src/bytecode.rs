#[derive(Debug, Clone, Copy, FromPrimitive)]
pub enum Bytecode {
    Nop = 0x0,
    LoadNull = 0x1,
    LoadConst = 0x2,
    LoadValue = 0x3,
    LoadDot = 0x4,
    LoadSubscript = 0x5,
    LoadOp = 0x6,
    PopTop = 0x7,
    DupTop = 0x8,
    Swap2 = 0x9,
    Swap3 = 0xA,
    SwapN = 0xB,
    Store = 0xC,
    StoreSubscript = 0xD,
    StoreAttr = 0xE,
    // Binary operators
    Plus = 0x10,
    Minus = 0x11,
    Times = 0x12,
    Divide = 0x13,
    FloorDiv = 0x14,
    Mod = 0x15,
    Subscript = 0x16,
    Power = 0x17,
    LBitshift = 0x18,
    RBitshift = 0x19,
    BitwiseAnd = 0x1A,
    BitwiseOr = 0x1B,
    BitwiseXor = 0x1C,
    Compare = 0x1D,
    DelSubscript = 0x1E,
    UMinus = 0x1F,
    BitwiseNot = 0x20,
    BoolAnd = 0x21,
    BoolOr = 0x22,
    BoolNot = 0x23,
    BoolXor = 0x24,
    Identical = 0x25,
    Instanceof = 0x26,
    CallOp = 0x27,
    PackTuple = 0x28,
    UnpackTuple = 0x29,
    Equal = 0x2A,
    LessThan = 0x2B,
    GreaterThan = 0x2C,
    LessEqual = 0x2D,
    GreaterEqual = 0x2E,
    Contains = 0x2F,
    // Jumps, etc.
    Jump = 0x30,
    JumpFalse = 0x31,
    JumpTrue = 0x32,
    JumpNN = 0x33,
    JumpNull = 0x34,
    CallMethod = 0x35,
    CallTos = 0x36,
    CallFunction = 0x37,
    TailMethod = 0x38,
    TailTos = 0x39,
    TailFunction = 0x3A,
    Return = 0x3B,
    // Exception stuff
    Throw = 0x40,
    ThrowQuick = 0x41,
    EnterTry = 0x42,
    ExceptN = 0x43,
    Finally = 0x44,
    EndTry = 0x45,
    // Markers
    FuncDef = 0x48,
    ClassDef = 0x49,
    EndClass = 0x4A,
    // Loop stuff
    ForIter = 0x50,
    ListCreate = 0x51,
    SetCreate = 0x52,
    DictCreate = 0x53,
    ListAdd = 0x54,
    SetAdd = 0x55,
    DictAdd = 0x56,
    Dotimes = 0x57,
    // Misc.
    LoadFunction = 0x60,
}

pub fn bytecode_size(b: Bytecode) -> usize {
    return match b {
        Bytecode::Nop | Bytecode::LoadNull => 0,
        Bytecode::LoadConst
        | Bytecode::LoadValue
        | Bytecode::LoadDot
        | Bytecode::LoadSubscript
        | Bytecode::LoadOp => 2,
        Bytecode::PopTop | Bytecode::DupTop | Bytecode::Swap2 | Bytecode::Swap3 => 0,
        Bytecode::SwapN => 4,
        Bytecode::Store | Bytecode::StoreSubscript | Bytecode::StoreAttr => 2,
        Bytecode::Plus
        | Bytecode::Minus
        | Bytecode::Times
        | Bytecode::Divide
        | Bytecode::FloorDiv
        | Bytecode::Mod
        | Bytecode::Subscript
        | Bytecode::Power
        | Bytecode::LBitshift
        | Bytecode::RBitshift
        | Bytecode::BitwiseAnd
        | Bytecode::BitwiseOr
        | Bytecode::BitwiseXor
        | Bytecode::Compare
        | Bytecode::DelSubscript
        | Bytecode::UMinus
        | Bytecode::BitwiseNot
        | Bytecode::BoolAnd
        | Bytecode::BoolOr
        | Bytecode::BoolNot
        | Bytecode::BoolXor
        | Bytecode::Identical
        | Bytecode::Instanceof => 0,
        Bytecode::CallOp => 4,
        Bytecode::PackTuple
        | Bytecode::UnpackTuple
        | Bytecode::Equal
        | Bytecode::LessThan
        | Bytecode::GreaterThan
        | Bytecode::LessEqual
        | Bytecode::GreaterEqual
        | Bytecode::Contains => 0,
        Bytecode::Jump
        | Bytecode::JumpFalse
        | Bytecode::JumpTrue
        | Bytecode::JumpNN
        | Bytecode::JumpNull => 4,
        Bytecode::CallMethod => 4,
        Bytecode::CallTos => 2,
        Bytecode::CallFunction => 2 + 2,
        Bytecode::TailMethod => 4,
        Bytecode::TailTos => 2,
        Bytecode::TailFunction => 2 + 2,
        Bytecode::Return => 2,
        Bytecode::Throw => 0,
        Bytecode::ThrowQuick => 2,
        Bytecode::EnterTry => 4,
        Bytecode::ExceptN => 2,
        Bytecode::Finally
        | Bytecode::EndTry
        | Bytecode::FuncDef
        | Bytecode::ClassDef
        | Bytecode::EndClass => 0,
        Bytecode::ForIter => 4,
        Bytecode::ListCreate | Bytecode::SetCreate | Bytecode::DictCreate => 2,
        Bytecode::ListAdd | Bytecode::SetAdd | Bytecode::DictAdd => 0,
        Bytecode::Dotimes => 4,
        Bytecode::LoadFunction => 2,
    };
}
