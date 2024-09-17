use crate::cpu::AddressingMode;
use std::collections::HashMap;

pub struct OpCode { // store all information associated with an OpCode
    pub code: u8,
    pub mnemonic: &'static str, // static lifetime: lives for entire duration of prgrm.
    // always used for string literals.
    pub len: u8,
    pub cycles: u8,
    pub mode: AddressingMode,
}

impl OpCode {
    fn new(code: u8, mnemonic: &'static str, len: u8, cycles: u8, mode: AddressingMode) -> Self {
        OpCode {
            code: code,
            mnemonic: mnemonic,
            len: len,
            cycles: cycles,
            mode: mode,
        }
    }
}


lazy_static! {
// We need statics (values which are true for the entire program duration) to allow for these values to
// be (hash)-mapped to the CPU.

// This must occur at runtime, which is what the lazy_static! marcro provides:
// " ... it is possible to have statics that require code to be executed at runtime in order to be initialized. 
// This includes anything requiring heap allocations, like vectors or hash maps, as well as anything that requires 
// function calls to be computed."

    pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![
        // A ref is like matching but with a borrow, still retaining the thing which had to be
        // compared to a pattern for further use.
        OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
        OpCode::new(0xaa, "TAX", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xe8, "INX", 1, 2, AddressingMode::NoneAddressing),

        OpCode::new(0xa9, "LDA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa5, "LDA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xad, "LDA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbd, "LDA", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_X),
        OpCode::new(0xb9, "LDA", 3, 4/*+1 if page crossed*/, AddressingMode::Absolute_Y),
        OpCode::new(0xa1, "LDA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xb1, "LDA", 2, 5/*+1 if page crossed*/, AddressingMode::Indirect_Y),

        OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8d, "STA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x9d, "STA", 3, 5, AddressingMode::Absolute_X),
        OpCode::new(0x99, "STA", 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x81, "STA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x91, "STA", 2, 6, AddressingMode::Indirect_Y),

    ];


    pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {
        // Returns hashmap (key w/ value) from type u8 (instruction from memory, like a9 or c0 or aa etc.) with a borrow to its
        // corresponding OpCode data, which is available for the entire program.
        let mut map = HashMap::new();
        for cpuop in &*CPU_OPS_CODES { // Dereference the value from the vec! CPU_OP_CODES, and then borrow it.
            // The deref is important as CPU_OP_CODES is a ref (somewhat like &) and thus won't give the values,
            // instead returning the pointers.
            map.insert(cpuop.code, cpuop); // Map each u8 opcode (value) with the full description of the OpCode (type)
        }
        map
    };
}