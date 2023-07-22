#[derive(Debug, PartialEq)]
enum Mode {
    MemoryModeNoDisplacement,
    MemoryMode8BitDisplacement,
    MemoryMode16BitDisplacement,
    RegisterMode,
}

#[derive(Debug, PartialEq)]
pub enum RegisterName {
    AL,
    BL,
    CL,
    DL,
    AH,
    BH,
    CH,
    DH,
    AX,
    BX,
    CX,
    DX,
    BP,
    SP,
    DI,
    SI,
}

impl std::fmt::Display for RegisterName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            RegisterName::AL => "al",
            RegisterName::CL => "cl",
            RegisterName::DL => "dl",
            RegisterName::BL => "bl",
            RegisterName::AH => "ah",
            RegisterName::CH => "ch",
            RegisterName::DH => "dh",
            RegisterName::BH => "bh",
            RegisterName::AX => "ax",
            RegisterName::CX => "cx",
            RegisterName::DX => "dx",
            RegisterName::BX => "bx",
            RegisterName::SP => "sp",
            RegisterName::BP => "bp",
            RegisterName::SI => "si",
            RegisterName::DI => "di",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Mnemonic {
    ADD,
    CMP,
    JA,
    JC,
    JCXZ,
    JG,
    JL,
    JNA,
    JNC,
    JNG,
    JNL,
    JNO,
    JNS,
    JNZ,
    JO,
    JPE,
    JPO,
    JS,
    JZ,
    LOOP,
    LOOPE,
    LOOPNE,
    MOV,
    SUB,
}

impl std::fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Mnemonic::ADD => "add",
            Mnemonic::CMP => "cmp",
            Mnemonic::JA => "ja",
            Mnemonic::JC => "jc",
            Mnemonic::JCXZ => "jcxz",
            Mnemonic::JG => "jg",
            Mnemonic::JL => "jl",
            Mnemonic::JNA => "jna",
            Mnemonic::JNC => "jnc",
            Mnemonic::JNG => "jng",
            Mnemonic::JNL => "jnl",
            Mnemonic::JNO => "jno",
            Mnemonic::JNS => "jns",
            Mnemonic::JNZ => "jnz",
            Mnemonic::JO => "jo",
            Mnemonic::JPE => "jpe",
            Mnemonic::JPO => "jpo",
            Mnemonic::JS => "js",
            Mnemonic::JZ => "jz",
            Mnemonic::LOOP => "loop",
            Mnemonic::LOOPE => "loope",
            Mnemonic::LOOPNE => "loopne",
            Mnemonic::MOV => "mov",
            Mnemonic::SUB => "sub",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum RegisterMemory {
    Register(RegisterName),
    RegisterAddress(RegisterName),
    RegisterAddressDisplacement(RegisterName, u16),
    RegisterAddressOffset(RegisterName, RegisterName),
    RegisterAddressOffsetDisplacement(RegisterName, RegisterName, u16),
    DirectAddress(u16),
}

impl std::fmt::Display for RegisterMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RegisterMemory::Register(register) => register.to_string(),
            RegisterMemory::RegisterAddress(register) => format!("[{}]", register),
            RegisterMemory::RegisterAddressDisplacement(register, displacement) => {
                if *displacement == 0 {
                    format!("[{}]", register)
                } else {
                    format!("[{} + {}]", register, displacement)
                }
            }
            RegisterMemory::RegisterAddressOffset(register, offset_register) => {
                format!("[{} + {}]", register, offset_register)
            }
            RegisterMemory::RegisterAddressOffsetDisplacement(
                register,
                offset_register,
                displacement,
            ) => {
                if *displacement == 0 {
                    format!("[{} + {}]", register, offset_register)
                } else {
                    format!("[{} + {} + {}]", register, offset_register, displacement)
                }
            }
            RegisterMemory::DirectAddress(address) => format!("[{}]", address),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum InstructionCategory {
    RegisterMemoryAndRegister(Mnemonic, RegisterMemory, RegisterMemory),
    ImmediateToRegister(Mnemonic, u16, RegisterName),
    ImmediateToRegisterMemory(Mnemonic, u16, RegisterMemory, bool),
    ImmediateToAccumulator(Mnemonic, u16, RegisterName),
    Jump(Mnemonic, i8),
}

impl std::fmt::Display for InstructionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            InstructionCategory::RegisterMemoryAndRegister(mnemonic, src, dest) => {
                format!("{} {}, {}", mnemonic, dest, src)
            }
            InstructionCategory::ImmediateToRegister(mnemonic, immediate, register) => {
                format!("{} {}, {}", mnemonic, register, immediate)
            }
            InstructionCategory::ImmediateToRegisterMemory(
                mnemonic,
                immediate,
                dest,
                word_operation,
            ) => {
                format!(
                    "{} {} {}, {}",
                    mnemonic,
                    if *word_operation { "word" } else { "byte" },
                    dest,
                    immediate
                )
            }
            InstructionCategory::ImmediateToAccumulator(mnemonic, immediate, dest) => {
                format!("{} {}, {}", mnemonic, dest, immediate)
            }
            InstructionCategory::Jump(mnemonic, increment) => format!("{} {}", mnemonic, increment),
        };
        write!(f, "{}", s)
    }
}

pub struct Instruction {
    pub(crate) length: usize,
    pub(crate) instruction_category: InstructionCategory,
}

#[derive(Debug)]
struct RegMemoryWithRegisterToEitherOperands {
    instruction_length: usize,
    register: RegisterName,
    register_memory: RegisterMemory,
}

#[derive(Debug)]
struct ImmediateToRegisterMemoryOperands {
    instruction_length: usize,
    register_memory: RegisterMemory,
    immediate: u16,
}

#[derive(Debug)]
pub(crate) enum DecodeError {
    InvalidInstruction,
    InvalidMode,
    InvalidRegister,
}

pub(crate) fn decode_instruction(remaining_bytes: &[u8]) -> Result<Instruction, DecodeError> {
    let instruction = remaining_bytes[0];

    if (instruction & 0b11111100) == 0b10000000 {
        let mnemonic = match (remaining_bytes[1] & 0b00111000) >> 3 {
            0x0 => Mnemonic::ADD,
            0x5 => Mnemonic::SUB,
            0x7 => Mnemonic::CMP,
            _ => return Err(DecodeError::InvalidInstruction),
        };
        return Ok(decode_immediate_to_register_memory(
            mnemonic,
            remaining_bytes,
        ));
    }

    if (instruction & 0b11111100) == 0b10001000 {
        return Ok(decode_reg_memory_and_register_to_either(
            Mnemonic::MOV,
            remaining_bytes,
        ));
    }
    if (instruction & 0b11110000) == 0b10110000 {
        return Ok(decode_immediate_to_register(Mnemonic::MOV, remaining_bytes));
    }

    if (instruction & 0b11111100) == 0b00000000 {
        return Ok(decode_reg_memory_and_register_to_either(
            Mnemonic::ADD,
            remaining_bytes,
        ));
    }
    if (instruction & 0b11111110) == 0b00000100 {
        return Ok(decode_immediate_to_accumulator(
            Mnemonic::ADD,
            remaining_bytes,
        ));
    }

    if (instruction & 0b11111100) == 0b00101000 {
        return Ok(decode_reg_memory_and_register_to_either(
            Mnemonic::SUB,
            remaining_bytes,
        ));
    }
    if (instruction & 0b11111110) == 0b00101100 {
        return Ok(decode_immediate_to_accumulator(
            Mnemonic::SUB,
            remaining_bytes,
        ));
    }

    if (instruction & 0b11111100) == 0b00111000 {
        return Ok(decode_reg_memory_and_register_to_either(
            Mnemonic::CMP,
            remaining_bytes,
        ));
    }
    if (instruction & 0b11111110) == 0b00111100 {
        return Ok(decode_immediate_to_accumulator(
            Mnemonic::CMP,
            remaining_bytes,
        ));
    }

    match instruction {
        0b01110100 => return Ok(decode_jump(Mnemonic::JZ, remaining_bytes)),
        0b01111100 => return Ok(decode_jump(Mnemonic::JL, remaining_bytes)),
        0b01111110 => return Ok(decode_jump(Mnemonic::JNG, remaining_bytes)),
        0b01110010 => return Ok(decode_jump(Mnemonic::JC, remaining_bytes)),
        0b01110110 => return Ok(decode_jump(Mnemonic::JNA, remaining_bytes)),
        0b01111010 => return Ok(decode_jump(Mnemonic::JPE, remaining_bytes)),
        0b01110000 => return Ok(decode_jump(Mnemonic::JO, remaining_bytes)),
        0b01111000 => return Ok(decode_jump(Mnemonic::JS, remaining_bytes)),
        0b01110101 => return Ok(decode_jump(Mnemonic::JNZ, remaining_bytes)),
        0b01111101 => return Ok(decode_jump(Mnemonic::JNL, remaining_bytes)),
        0b01111111 => return Ok(decode_jump(Mnemonic::JG, remaining_bytes)),
        0b01110011 => return Ok(decode_jump(Mnemonic::JNC, remaining_bytes)),
        0b01110111 => return Ok(decode_jump(Mnemonic::JA, remaining_bytes)),
        0b01111011 => return Ok(decode_jump(Mnemonic::JPO, remaining_bytes)),
        0b01110001 => return Ok(decode_jump(Mnemonic::JNO, remaining_bytes)),
        0b01111001 => return Ok(decode_jump(Mnemonic::JNS, remaining_bytes)),
        0b11100010 => return Ok(decode_jump(Mnemonic::LOOP, remaining_bytes)),
        0b11100001 => return Ok(decode_jump(Mnemonic::LOOPE, remaining_bytes)),
        0b11100000 => return Ok(decode_jump(Mnemonic::LOOPNE, remaining_bytes)),
        0b11100011 => return Ok(decode_jump(Mnemonic::JCXZ, remaining_bytes)),
        _ => Err(DecodeError::InvalidInstruction),
    }
}

fn decode_reg_memory_with_register_to_either_operands(
    instruction_stream: &[u8],
    word_operation: bool,
) -> Result<RegMemoryWithRegisterToEitherOperands, DecodeError> {
    let operands_byte = instruction_stream[1];

    let register = decode_register((operands_byte & 0x38) >> 3, word_operation)?;

    let mode = decode_mod(operands_byte >> 6)?;

    let instruction_length: usize;
    let mut displacement: u16 = 0;
    match mode {
        Mode::MemoryModeNoDisplacement | Mode::RegisterMode => instruction_length = 2,
        Mode::MemoryMode8BitDisplacement => {
            instruction_length = 3;
            displacement = instruction_stream[2] as u16;
        }
        Mode::MemoryMode16BitDisplacement => {
            instruction_length = 4;
            displacement = u16::from_le_bytes([instruction_stream[2], instruction_stream[3]]);
        }
    }

    let register_memory =
        decode_register_memory(operands_byte & 0x7, &mode, displacement, word_operation)?;

    Ok(RegMemoryWithRegisterToEitherOperands {
        instruction_length,
        register,
        register_memory,
    })
}

fn decode_immediate_to_register_memory_operands(
    instruction_stream: &[u8],
    sign_extension: bool,
    word_operation: bool,
) -> Result<ImmediateToRegisterMemoryOperands, DecodeError> {
    let operands_byte = instruction_stream[1];
    let word_immediate = !sign_extension && word_operation;

    let register_memory_byte = operands_byte & 0x7;

    let mode = decode_mod(operands_byte >> 6)?;

    let instruction_length: usize;
    let mut displacement: u16 = 0;
    let immediate: u16;
    match mode {
        Mode::RegisterMode => {
            if word_immediate {
                instruction_length = 4;
                immediate = u16::from_le_bytes([instruction_stream[2], instruction_stream[3]]);
            } else {
                instruction_length = 3;
                immediate = instruction_stream[2] as u16;
            }
        }
        Mode::MemoryModeNoDisplacement => {
            if register_memory_byte == 0x6 {
                // Shitty hack because we need the displacement to construct a
                // DirectAddress, but we  also need to check the register/memory
                // type is a direct address to know to calculate the
                // displacement.
                displacement = u16::from_le_bytes([instruction_stream[2], instruction_stream[3]]);
                if word_immediate {
                    instruction_length = 6;
                    immediate = u16::from_le_bytes([instruction_stream[4], instruction_stream[5]]);
                } else {
                    instruction_length = 5;
                    immediate = instruction_stream[4] as u16;
                }
            } else {
                if word_immediate {
                    instruction_length = 4;
                    immediate = u16::from_le_bytes([instruction_stream[2], instruction_stream[3]]);
                } else {
                    instruction_length = 3;
                    immediate = instruction_stream[2] as u16;
                }
            }
        }
        Mode::MemoryMode8BitDisplacement => {
            displacement = instruction_stream[2] as u16;
            if word_immediate {
                instruction_length = 5;
                immediate = u16::from_le_bytes([instruction_stream[3], instruction_stream[4]]);
            } else {
                instruction_length = 4;
                immediate = instruction_stream[3] as u16;
            }
        }
        Mode::MemoryMode16BitDisplacement => {
            displacement = u16::from_le_bytes([instruction_stream[2], instruction_stream[3]]);
            if word_immediate {
                instruction_length = 6;
                immediate = u16::from_le_bytes([instruction_stream[4], instruction_stream[5]]);
            } else {
                instruction_length = 5;
                immediate = instruction_stream[4] as u16;
            }
        }
    }

    let register_memory =
        decode_register_memory(register_memory_byte, &mode, displacement, word_operation)?;

    Ok(ImmediateToRegisterMemoryOperands {
        instruction_length,
        register_memory,
        immediate,
    })
}

fn decode_mod(mod_byte: u8) -> Result<Mode, DecodeError> {
    let mode = match mod_byte {
        0x0 => Mode::MemoryModeNoDisplacement,
        0x1 => Mode::MemoryMode8BitDisplacement,
        0x2 => Mode::MemoryMode16BitDisplacement,
        0x3 => Mode::RegisterMode,
        _ => return Err(DecodeError::InvalidMode),
    };
    Ok(mode)
}

fn decode_register_memory(
    register_memory_byte: u8,
    mode: &Mode,
    displacement: u16,
    word_operation: bool,
) -> Result<RegisterMemory, DecodeError> {
    let register_memory = match (register_memory_byte, mode) {
        (0x0, Mode::MemoryMode8BitDisplacement) | (0x0, Mode::MemoryMode16BitDisplacement) => {
            RegisterMemory::RegisterAddressOffsetDisplacement(
                RegisterName::BX,
                RegisterName::SI,
                displacement,
            )
        }
        (0x0, Mode::MemoryModeNoDisplacement) => {
            RegisterMemory::RegisterAddressOffset(RegisterName::BX, RegisterName::SI)
        }
        (0x0, Mode::RegisterMode) => {
            if word_operation {
                RegisterMemory::Register(RegisterName::AX)
            } else {
                RegisterMemory::Register(RegisterName::AL)
            }
        }

        (0x1, Mode::MemoryMode8BitDisplacement) | (0x1, Mode::MemoryMode16BitDisplacement) => {
            RegisterMemory::RegisterAddressOffsetDisplacement(
                RegisterName::BX,
                RegisterName::DI,
                displacement,
            )
        }
        (0x1, Mode::MemoryModeNoDisplacement) => {
            RegisterMemory::RegisterAddressOffset(RegisterName::BX, RegisterName::DI)
        }
        (0x1, Mode::RegisterMode) => {
            if word_operation {
                RegisterMemory::Register(RegisterName::CX)
            } else {
                RegisterMemory::Register(RegisterName::CL)
            }
        }

        (0x2, Mode::MemoryMode8BitDisplacement) | (0x2, Mode::MemoryMode16BitDisplacement) => {
            RegisterMemory::RegisterAddressOffsetDisplacement(
                RegisterName::BP,
                RegisterName::SI,
                displacement,
            )
        }
        (0x2, Mode::MemoryModeNoDisplacement) => {
            RegisterMemory::RegisterAddressOffset(RegisterName::BP, RegisterName::SI)
        }
        (0x2, Mode::RegisterMode) => {
            if word_operation {
                RegisterMemory::Register(RegisterName::DX)
            } else {
                RegisterMemory::Register(RegisterName::DL)
            }
        }

        (0x3, Mode::MemoryMode8BitDisplacement) | (0x3, Mode::MemoryMode16BitDisplacement) => {
            RegisterMemory::RegisterAddressOffsetDisplacement(
                RegisterName::BP,
                RegisterName::DI,
                displacement,
            )
        }
        (0x3, Mode::MemoryModeNoDisplacement) => {
            RegisterMemory::RegisterAddressOffset(RegisterName::BP, RegisterName::DI)
        }
        (0x3, Mode::RegisterMode) => {
            if word_operation {
                RegisterMemory::Register(RegisterName::BX)
            } else {
                RegisterMemory::Register(RegisterName::BL)
            }
        }

        (0x4, Mode::MemoryMode8BitDisplacement) | (0x4, Mode::MemoryMode16BitDisplacement) => {
            RegisterMemory::RegisterAddressDisplacement(RegisterName::SI, displacement)
        }
        (0x4, Mode::MemoryModeNoDisplacement) => RegisterMemory::RegisterAddress(RegisterName::SI),
        (0x4, Mode::RegisterMode) => {
            if word_operation {
                RegisterMemory::Register(RegisterName::SP)
            } else {
                RegisterMemory::Register(RegisterName::AH)
            }
        }

        (0x5, Mode::MemoryMode8BitDisplacement) | (0x5, Mode::MemoryMode16BitDisplacement) => {
            RegisterMemory::RegisterAddressDisplacement(RegisterName::DI, displacement)
        }
        (0x5, Mode::MemoryModeNoDisplacement) => RegisterMemory::RegisterAddress(RegisterName::DI),
        (0x5, Mode::RegisterMode) => {
            if word_operation {
                RegisterMemory::Register(RegisterName::BP)
            } else {
                RegisterMemory::Register(RegisterName::CH)
            }
        }

        (0x6, Mode::MemoryMode8BitDisplacement) | (0x6, Mode::MemoryMode16BitDisplacement) => {
            RegisterMemory::RegisterAddressDisplacement(RegisterName::BP, displacement)
        }
        (0x6, Mode::MemoryModeNoDisplacement) => RegisterMemory::DirectAddress(displacement),
        (0x6, Mode::RegisterMode) => {
            if word_operation {
                RegisterMemory::Register(RegisterName::SI)
            } else {
                RegisterMemory::Register(RegisterName::DH)
            }
        }

        (0x7, Mode::MemoryMode8BitDisplacement) | (0x7, Mode::MemoryMode16BitDisplacement) => {
            RegisterMemory::RegisterAddressDisplacement(RegisterName::BX, displacement)
        }
        (0x7, Mode::MemoryModeNoDisplacement) => RegisterMemory::RegisterAddress(RegisterName::BX),
        (0x7, Mode::RegisterMode) => {
            if word_operation {
                RegisterMemory::Register(RegisterName::DI)
            } else {
                RegisterMemory::Register(RegisterName::BH)
            }
        }

        (_, _) => return Err(DecodeError::InvalidInstruction),
    };
    Ok(register_memory)
}

fn decode_register(register_byte: u8, word_operation: bool) -> Result<RegisterName, DecodeError> {
    match (register_byte, word_operation) {
        (0x0, false) => Ok(RegisterName::AL),
        (0x1, false) => Ok(RegisterName::CL),
        (0x2, false) => Ok(RegisterName::DL),
        (0x3, false) => Ok(RegisterName::BL),
        (0x4, false) => Ok(RegisterName::AH),
        (0x5, false) => Ok(RegisterName::CH),
        (0x6, false) => Ok(RegisterName::DH),
        (0x7, false) => Ok(RegisterName::BH),
        (0x0, true) => Ok(RegisterName::AX),
        (0x1, true) => Ok(RegisterName::CX),
        (0x2, true) => Ok(RegisterName::DX),
        (0x3, true) => Ok(RegisterName::BX),
        (0x4, true) => Ok(RegisterName::SP),
        (0x5, true) => Ok(RegisterName::BP),
        (0x6, true) => Ok(RegisterName::SI),
        (0x7, true) => Ok(RegisterName::DI),
        _ => return Err(DecodeError::InvalidRegister),
    }
}

fn decode_reg_memory_and_register_to_either(mnemonic: Mnemonic, bytes: &[u8]) -> Instruction {
    let word_operation = (bytes[0] & 0x1) != 0;
    let reg_is_destination = (bytes[0] & 0x2) != 0;

    let operands = decode_reg_memory_with_register_to_either_operands(bytes, word_operation)
        .expect("failed to decode operands");

    if reg_is_destination {
        Instruction {
            length: operands.instruction_length,
            instruction_category: InstructionCategory::RegisterMemoryAndRegister(
                mnemonic,
                operands.register_memory,
                RegisterMemory::Register(operands.register),
            ),
        }
    } else {
        Instruction {
            length: operands.instruction_length,
            instruction_category: InstructionCategory::RegisterMemoryAndRegister(
                mnemonic,
                RegisterMemory::Register(operands.register),
                operands.register_memory,
            ),
        }
    }
}

fn decode_immediate_to_register_memory(mnemonic: Mnemonic, bytes: &[u8]) -> Instruction {
    let word_operation = (bytes[0] & 0x1) != 0;
    let sign_extension = (bytes[0] & 0x2) != 0;

    let operands =
        decode_immediate_to_register_memory_operands(bytes, sign_extension, word_operation)
            .expect("failed to decode operands");

    Instruction {
        length: operands.instruction_length,
        instruction_category: InstructionCategory::ImmediateToRegisterMemory(
            mnemonic,
            operands.immediate,
            operands.register_memory,
            word_operation,
        ),
    }
}

fn decode_immediate_to_register(mnemonic: Mnemonic, bytes: &[u8]) -> Instruction {
    let word_operation = (bytes[0] & 0x8) != 0;

    let register =
        decode_register(bytes[0] & 7, word_operation).expect("failed to decode register");
    let immediate = if word_operation {
        u16::from_le_bytes([bytes[1], bytes[2]])
    } else {
        bytes[1] as u16
    };

    let length = if word_operation { 3 } else { 2 };
    Instruction {
        length,
        instruction_category: InstructionCategory::ImmediateToRegister(
            mnemonic, immediate, register,
        ),
    }
}

fn decode_immediate_to_accumulator(mnemonic: Mnemonic, bytes: &[u8]) -> Instruction {
    let word_operation = (bytes[0] & 0x1) != 0;

    let length = if word_operation { 3 } else { 2 };

    let data = if word_operation {
        u16::from_le_bytes([bytes[1], bytes[2]])
    } else {
        bytes[1] as u16
    };

    let register_name = if word_operation {
        RegisterName::AX
    } else {
        RegisterName::AL
    };

    Instruction {
        length,
        instruction_category: InstructionCategory::ImmediateToAccumulator(
            mnemonic,
            data,
            register_name,
        ),
    }
}

fn decode_jump(mnemonic: Mnemonic, remaining_bytes: &[u8]) -> Instruction {
    let increment = remaining_bytes[1] as i8;

    Instruction {
        length: 2,
        instruction_category: InstructionCategory::Jump(mnemonic, increment),
    }
}
