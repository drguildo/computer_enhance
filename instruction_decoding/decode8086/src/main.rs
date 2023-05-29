#[derive(Debug)]
enum InstructionType {
    MovRegisterMemoryToFromRegister,
    MovImmediateToRegister,
    AddRegisterMemoryWithRegisterToEither,
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
struct Operands {
    instruction_length: usize,
    register: RegisterName,
    register_memory: RegisterMemory,
}

#[derive(Debug)]
enum DecodeError {
    InvalidInstruction,
    InvalidMode,
    InvalidRegister,
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
        let byte = bytes[instruction_index];
        if let Ok(instruction_type) = identify_instruction(byte) {
            let remaining_bytes = &bytes[instruction_index..];
            let instruction_size = match instruction_type {
                InstructionType::MovRegisterMemoryToFromRegister => decode_mov(remaining_bytes),
                InstructionType::MovImmediateToRegister => {
                    decode_mov_immediate_to_register(remaining_bytes)
                }
                InstructionType::AddRegisterMemoryWithRegisterToEither => todo!(),
            };
            instruction_index += instruction_size;
        } else {
            panic!("unsupported instruction: {byte:#010b}");
        }
    }
}

fn identify_instruction(byte: u8) -> Result<InstructionType, DecodeError> {
    if (byte & 0b11111100) == 0b10001000 {
        return Ok(InstructionType::MovRegisterMemoryToFromRegister);
    }
    if (byte & 0b11110000) == 0b10110000 {
        return Ok(InstructionType::MovImmediateToRegister);
    }
    if (byte & 0b11111100) == 0b00000000 {
        return Ok(InstructionType::AddRegisterMemoryWithRegisterToEither);
    }
    Err(DecodeError::InvalidInstruction)
}

fn decode_operands(
    instruction_stream: &[u8],
    word_operation: bool,
) -> Result<Operands, DecodeError> {
    let operands_byte = instruction_stream[1];

    let register = decode_register((operands_byte & 0x38) >> 3, word_operation)?;

    let instruction_length: usize;
    let mut displacement: u16 = 0;
    let mode = match operands_byte >> 6 {
        0x0 => {
            instruction_length = 2;
            Mode::MemoryModeNoDisplacement
        }
        0x1 => {
            instruction_length = 3;
            displacement = instruction_stream[2] as u16;
            Mode::MemoryMode8BitDisplacement
        }
        0x2 => {
            instruction_length = 4;
            displacement = u16::from_be_bytes([instruction_stream[3], instruction_stream[2]]);
            Mode::MemoryMode16BitDisplacement
        }
        0x3 => {
            instruction_length = 2;
            Mode::RegisterMode
        }
        _ => return Err(DecodeError::InvalidMode),
    };

    let register_memory = match (operands_byte & 0x7, mode) {
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

    Ok(Operands {
        instruction_length,
        register,
        register_memory,
    })
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

fn decode_mov(bytes: &[u8]) -> usize {
    let word_operation = (bytes[0] & 0x1) != 0;
    let reg_is_destination = (bytes[0] & 0x2) != 0;

    let operands = decode_operands(bytes, word_operation).expect("failed to decode operands");

    if reg_is_destination {
        println!("mov {}, {}", operands.register, operands.register_memory);
        return operands.instruction_length;
    } else {
        println!("mov {}, {}", operands.register_memory, operands.register);
        return operands.instruction_length;
    }
}

fn decode_mov_immediate_to_register(bytes: &[u8]) -> usize {
    let word_operation = (bytes[0] & 0x8) != 0;

    let register =
        decode_register(bytes[0] & 7, word_operation).expect("failed to decode register");
    let displacement = if word_operation {
        u16::from_be_bytes([bytes[2], bytes[1]])
    } else {
        bytes[1] as u16
    };

    println!("mov {}, {}", register, displacement);

    if word_operation {
        3
    } else {
        2
    }
}
