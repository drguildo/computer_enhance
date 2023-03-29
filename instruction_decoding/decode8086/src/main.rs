#[derive(Debug)]
enum InstructionType {
    MovRegisterMemoryToFromRegister,
    MovImmediateToRegister,
}

#[derive(Debug)]
enum Mode {
    MemoryModeNoDisplacement,
    MemoryMode8BitDisplacement,
    MemoryMode16BitDisplacement,
    RegisterMode,
}

#[derive(Debug)]
enum Register {
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

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Register::AL => "al",
            Register::CL => "cl",
            Register::DL => "dl",
            Register::BL => "bl",
            Register::AH => "ah",
            Register::CH => "ch",
            Register::DH => "dh",
            Register::BH => "bh",
            Register::AX => "ax",
            Register::CX => "cx",
            Register::DX => "dx",
            Register::BX => "bx",
            Register::SP => "sp",
            Register::BP => "bp",
            Register::SI => "si",
            Register::DI => "di",
        };
        write!(f, "{}", s)
    }
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
                InstructionType::MovRegisterMemoryToFromRegister => {
                    decode_register_memory_to_from_register(remaining_bytes)
                }
                InstructionType::MovImmediateToRegister => {
                    decode_immediate_to_register(remaining_bytes)
                }
            };
            instruction_index += instruction_size;
        } else {
            panic!("unsupported instruction: {byte:b}");
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
    Err(DecodeError::InvalidInstruction)
}

fn decode_mode(byte: u8) -> Result<Mode, DecodeError> {
    match byte {
        0x0 => Ok(Mode::MemoryModeNoDisplacement),
        0x1 => Ok(Mode::MemoryMode8BitDisplacement),
        0x2 => Ok(Mode::MemoryMode16BitDisplacement),
        0x3 => Ok(Mode::RegisterMode),
        _ => Err(DecodeError::InvalidMode),
    }
}

fn decode_register(byte: u8, word_operation: bool) -> Result<Register, DecodeError> {
    match (byte, word_operation) {
        (0x0, false) => Ok(Register::AL),
        (0x1, false) => Ok(Register::CL),
        (0x2, false) => Ok(Register::DL),
        (0x3, false) => Ok(Register::BL),
        (0x4, false) => Ok(Register::AH),
        (0x5, false) => Ok(Register::CH),
        (0x6, false) => Ok(Register::DH),
        (0x7, false) => Ok(Register::BH),
        (0x0, true) => Ok(Register::AX),
        (0x1, true) => Ok(Register::CX),
        (0x2, true) => Ok(Register::DX),
        (0x3, true) => Ok(Register::BX),
        (0x4, true) => Ok(Register::SP),
        (0x5, true) => Ok(Register::BP),
        (0x6, true) => Ok(Register::SI),
        (0x7, true) => Ok(Register::DI),
        _ => Err(DecodeError::InvalidRegister),
    }
}

fn decode_effective_address(rm: u8) -> String {
    match rm {
        0x0 => "bx + si".to_string(),
        0x1 => "bx + di".to_string(),
        0x2 => "bp + si".to_string(),
        0x3 => "bp + di".to_string(),
        0x4 => "si".to_string(),
        0x5 => "di".to_string(),
        0x6 => "bp".to_string(),
        0x7 => "bx".to_string(),
        _ => panic!("invalid rm value"),
    }
}

fn bytes_to_wide(a: u8, b: u8) -> u16 {
    let mut n = a as u16;
    n = ((b as u16) << 8) | n;
    n
}

fn decode_register_memory_to_from_register(bytes: &[u8]) -> usize {
    let word_operation = (bytes[0] & 0x1) != 0;
    let reg_is_destination = (bytes[0] & 0x2) != 0;

    let mode_field = decode_mode(bytes[1] >> 6).expect("failed to decode mode field");
    let register_field = decode_register((bytes[1] & 0x38) >> 3, word_operation)
        .expect("failed to decode register field");

    match mode_field {
        Mode::MemoryModeNoDisplacement => {
            println!("MemoryModeNoDisplacement: {:b} {:b}", bytes[0], bytes[1]);
            if reg_is_destination {
                println!(
                    "mov {},[{}]",
                    register_field.to_string(),
                    decode_effective_address(bytes[1] & 0x7)
                );
            } else {
                println!(
                    "mov [{}],{}",
                    decode_effective_address(bytes[1] & 0x7),
                    register_field.to_string()
                );
            }
            return 2;
        }
        Mode::MemoryMode8BitDisplacement => {
            println!(
                "MemoryMode8BitDisplacement: {:b} {:b} {:b}",
                bytes[0], bytes[1], bytes[2]
            );
            let displacement = bytes[2];
            return 3;
        }
        Mode::MemoryMode16BitDisplacement => {
            println!(
                "MemoryMode16BitDisplacement: {:b} {:b} {:b} {:b}",
                bytes[0], bytes[1], bytes[2], bytes[3]
            );
            let displacement = u16::from_be_bytes([bytes[2], bytes[3]]);
            return 4;
        }
        Mode::RegisterMode => {
            let rm_field = decode_register(bytes[1] & 0x7, word_operation)
                .expect("failed to decode r/m field");

            if reg_is_destination {
                println!(
                    "mov {},{}",
                    register_field.to_string(),
                    rm_field.to_string()
                );
                return 2;
            } else {
                println!(
                    "mov {},{}",
                    rm_field.to_string(),
                    register_field.to_string()
                );
                return 2;
            }
        }
    }
}

fn decode_immediate_to_register(bytes: &[u8]) -> usize {
    let wide_field = (bytes[0] & 0x8) != 0;
    let register_field =
        decode_register(bytes[0] & 0x7, wide_field).expect("failed to decode register field");

    if wide_field {
        println!(
            "mov {},{}",
            register_field.to_string(),
            bytes_to_wide(bytes[1], bytes[2])
        );
        return 3;
    } else {
        println!("mov {},{}", register_field.to_string(), bytes[1]);
        return 2;
    }
}
