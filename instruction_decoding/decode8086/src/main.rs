#[derive(Debug, PartialEq)]
enum InstructionType {
    MovRegisterMemoryToFromRegister,
    MovImmediateToRegister,
    MovImmediateToRegisterMemory,
    AddRegisterMemoryToFromRegister,
    AddImmediateToRegisterMemory,
}

impl std::fmt::Display for InstructionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            InstructionType::MovRegisterMemoryToFromRegister => "mov",
            InstructionType::MovImmediateToRegister => "mov",
            InstructionType::MovImmediateToRegisterMemory => "mov",
            InstructionType::AddRegisterMemoryToFromRegister => "add",
            InstructionType::AddImmediateToRegisterMemory => "add",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug)]
enum Mode {
    MemoryModeNoDisplacement,
    MemoryMode8BitDisplacement,
    MemoryMode16BitDisplacement,
    RegisterMode,
}

#[derive(Debug)]
enum RegisterName {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
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

#[derive(Debug)]
enum RegisterMemory {
    Register(RegisterName),
    RegisterAddress(RegisterName),
    RegisterAddressDisplacement(RegisterName, u16),
    RegisterAddressOffset(RegisterName, RegisterName),
    RegisterAddressOffsetDisplacement(RegisterName, RegisterName, u16),
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
        };
        write!(f, "{}", s)
    }
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
enum DecodeError {
    InvalidInstruction,
    InvalidMode,
    InvalidRegister,
    InvalidImmediateToRegisterInstruction,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = args.first().expect("failed to get path to binary");

    let bytes = std::fs::read(path).expect("failed to read file");

    if let Some(filename) = std::path::Path::new(path).file_name() {
        if let Some(filename_string) = filename.to_str() {
            println!("; {} disassembly:", filename_string);
        }
    }

    println!("bits 16");

    let mut instruction_index = 0;
    while instruction_index < bytes.len() {
        if let Ok(instruction_type) = identify_instruction(&bytes[instruction_index..]) {
            let remaining_bytes = &bytes[instruction_index..];
            let instruction_size = match instruction_type {
                InstructionType::MovRegisterMemoryToFromRegister
                | InstructionType::AddRegisterMemoryToFromRegister => {
                    decode_reg_memory_and_register_to_either(instruction_type, remaining_bytes)
                }
                InstructionType::MovImmediateToRegisterMemory => {
                    decode_immediate_to_register_memory(instruction_type, remaining_bytes)
                }
                InstructionType::MovImmediateToRegister => {
                    decode_immediate_to_register(instruction_type, remaining_bytes)
                }
                InstructionType::AddImmediateToRegisterMemory => {
                    decode_immediate_to_register_memory(instruction_type, remaining_bytes)
                }
            };
            instruction_index += instruction_size;
        } else {
            panic!("unsupported instruction: {:#010b}", bytes[0]);
        }
    }
}

fn identify_instruction(bytes: &[u8]) -> Result<InstructionType, DecodeError> {
    let instruction = bytes[0];
    if (instruction & 0b11111100) == 0b10000000 {
        return identify_immediate_to_register_instruction(bytes[1]);
    }
    if (instruction & 0b11111100) == 0b10001000 {
        return Ok(InstructionType::MovRegisterMemoryToFromRegister);
    }
    if (instruction & 0b11110000) == 0b10110000 {
        return Ok(InstructionType::MovImmediateToRegister);
    }
    if (instruction & 0b11111100) == 0b00000000 {
        return Ok(InstructionType::AddRegisterMemoryToFromRegister);
    }
    Err(DecodeError::InvalidInstruction)
}

fn identify_immediate_to_register_instruction(byte: u8) -> Result<InstructionType, DecodeError> {
    let instruction = (byte & 0b00111000) >> 2;
    match instruction {
        0x0 => Ok(InstructionType::AddImmediateToRegisterMemory),
        _ => Err(DecodeError::InvalidImmediateToRegisterInstruction),
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
            displacement = u16::from_be_bytes([instruction_stream[3], instruction_stream[2]]);
        }
    }

    let register_memory =
        decode_register_memory(operands_byte & 0x7, mode, displacement, word_operation)?;

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

    let mode = decode_mod(operands_byte >> 6)?;

    let instruction_length: usize;
    let mut displacement: u16 = 0;
    let immediate: u16;
    match mode {
        Mode::MemoryModeNoDisplacement | Mode::RegisterMode => {
            if word_immediate {
                instruction_length = 4;
                immediate = u16::from_be_bytes([instruction_stream[3], instruction_stream[2]]);
            } else {
                instruction_length = 3;
                immediate = instruction_stream[2] as u16;
            }
        }
        Mode::MemoryMode8BitDisplacement => {
            displacement = instruction_stream[2] as u16;
            if word_immediate {
                instruction_length = 5;
                immediate = u16::from_be_bytes([instruction_stream[4], instruction_stream[3]]);
            } else {
                instruction_length = 4;
                immediate = instruction_stream[3] as u16;
            }
        }
        Mode::MemoryMode16BitDisplacement => {
            displacement = u16::from_be_bytes([instruction_stream[3], instruction_stream[2]]);
            if word_immediate {
                instruction_length = 6;
                immediate = u16::from_be_bytes([instruction_stream[5], instruction_stream[4]]);
            } else {
                instruction_length = 5;
                immediate = instruction_stream[4] as u16;
            }
        }
    }

    let register_memory =
        decode_register_memory(operands_byte & 0x7, mode, displacement, word_operation)?;

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
    mode: Mode,
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
        (0x2, Mode::RegisterMode) => RegisterMemory::Register(RegisterName::DL),

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
        (0x6, Mode::MemoryModeNoDisplacement) => todo!(),
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
        (0x7, Mode::MemoryModeNoDisplacement) => RegisterMemory::Register(RegisterName::BX),
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

fn decode_reg_memory_and_register_to_either(
    instruction_type: InstructionType,
    bytes: &[u8],
) -> usize {
    let word_operation = (bytes[0] & 0x1) != 0;
    let reg_is_destination = (bytes[0] & 0x2) != 0;

    let operands = decode_reg_memory_with_register_to_either_operands(bytes, word_operation)
        .expect("failed to decode operands");

    if reg_is_destination {
        println!(
            "{} {}, {}",
            instruction_type, operands.register, operands.register_memory
        );
    } else {
        println!(
            "{} {}, {}",
            instruction_type, operands.register_memory, operands.register
        );
    }

    return operands.instruction_length;
}

fn decode_immediate_to_register_memory(instruction_type: InstructionType, bytes: &[u8]) -> usize {
    let word_operation = (bytes[0] & 0x1) != 0;
    let sign_extension = (bytes[0] & 0x2) != 0;

    let operands =
        decode_immediate_to_register_memory_operands(bytes, sign_extension, word_operation)
            .expect("failed to decode operands");

    println!(
        "{} {}, {}",
        instruction_type, operands.register_memory, operands.immediate
    );

    operands.instruction_length
}

fn decode_immediate_to_register(instruction_type: InstructionType, bytes: &[u8]) -> usize {
    let word_operation = (bytes[0] & 0x8) != 0;

    let register =
        decode_register(bytes[0] & 7, word_operation).expect("failed to decode register");
    let data = if word_operation {
        u16::from_be_bytes([bytes[2], bytes[1]])
    } else {
        bytes[1] as u16
    };

    println!("{} {}, {}", instruction_type, register, data);

    if word_operation {
        3
    } else {
        2
    }
}
